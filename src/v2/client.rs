/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::errors::SmugMugError;
use bytes::Bytes;
use chrono::{DateTime, Duration, TimeZone, Utc};
use const_format::formatcp;
use num_enum::TryFromPrimitive;
use reqwest::Response as ReqwestResponse;
use reqwest::header::HeaderMap;
use reqwest_oauth1::{OAuthClientProvider, SecretsProvider};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::sync::{Arc, RwLock};

// Root SmugMug API
pub(crate) const API_ORIGIN: &str = "https://api.smugmug.com";

// When retrieving pages this is the default records to retrieve
pub(crate) const NUM_TO_GET: usize = 25;

// String representation of default number of records to retrieve
pub(crate) const NUM_TO_GET_STRING: &str = formatcp!("{}", NUM_TO_GET);

/// Handles the lower level communication with the SmugMug REST API.
#[derive(Default, Clone)]
pub struct Client {
    inner: Arc<ClientRef>,
}

impl Client {
    /// Creates a new SmugMug client instance from the provided credentials
    pub fn new(creds: Creds) -> Self {
        Self {
            inner: Arc::new(ClientRef::new(creds)),
        }
    }

    /// Performs a GET request to the SmugMug API
    pub async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Option<&ApiParams<'_>>,
    ) -> Result<Response<T>, SmugMugError> {
        self.inner.get::<T>(url, params).await
    }

    /// Performs a GET request for binary data to the SmugMug API
    pub async fn get_binary_data(
        &self,
        url: &str,
        params: Option<&ApiParams<'_>>,
    ) -> Result<Response<Bytes>, SmugMugError> {
        self.inner.get_binary_data(url, params).await
    }

    /// Performs a PATCH request to the SmugMug API
    pub async fn patch<T: DeserializeOwned>(
        &self,
        url: &str,
        data: Vec<u8>,
        params: Option<&ApiParams<'_>>,
    ) -> Result<Response<T>, SmugMugError> {
        self.inner.patch::<T>(url, data, params).await
    }

    /// Performs a POST request to the SmugMug API
    pub async fn post<T: DeserializeOwned>(
        &self,
        url: &str,
        data: Vec<u8>,
        params: Option<&ApiParams<'_>>,
    ) -> Result<Response<T>, SmugMugError> {
        self.inner.post::<T>(url, data, params).await
    }

    /// Retrieves the last update for the API rate limit information.  This will return none if
    /// a get/post/patch API call hasn't been made yet.
    ///
    /// *NOTE: This isn't updated after [`Self::get_binary_data`] calls used in retrieving images/videos*
    pub fn get_last_rate_limit_window_update(&self) -> Option<Arc<RateLimitWindow>> {
        let rate_window = self
            .inner
            .last_rate_window
            .read()
            .expect("Failed read locking for last rate window update")
            .clone();
        if rate_window.is_valid() {
            Some(rate_window)
        } else {
            None
        }
    }
}

// Internal representation of the client
#[derive(Default)]
struct ClientRef {
    creds: Creds,
    https_client: reqwest::Client,
    last_rate_window: RwLock<Arc<RateLimitWindow>>,
}

impl ClientRef {
    // Creates a new SmugMug client instance from the provided credentials
    fn new(creds: Creds) -> Self {
        Self {
            creds,
            https_client: reqwest::Client::new(),
            last_rate_window: RwLock::new(Arc::new(RateLimitWindow {
                ..Default::default()
            })),
        }
    }

    // Performs a GET request to the SmugMug API
    async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Option<&ApiParams<'_>>,
    ) -> Result<Response<T>, SmugMugError> {
        let req_url = self.create_req(url, params)?;

        // If we are in read-only mode we have to do this a little different.  Since other functions
        // require Oauth1 singing, this is only needed for get.
        let resp = if self.creds.get_token_pair_option().is_some() {
            self.https_client
                .clone()
                .oauth1(self.creds.clone())
                .get(req_url)
                .header("Accept", "application/json")
                .send()
                .await?
        } else {
            self.https_client
                .clone()
                .get(req_url)
                .header("Accept", "application/json")
                .send()
                .await?
        };
        self.handle_json_response(resp).await
    }

    // Performs a GET request for binary data to the SmugMug API
    async fn get_binary_data(
        &self,
        url: &str,
        params: Option<&ApiParams<'_>>,
    ) -> Result<Response<Bytes>, SmugMugError> {
        let req_url = self.create_req(url, params)?;

        // If we are in read-only mode we have to do this a little different.  Since other functions
        // require Oauth1 singing, this is only needed for get.
        let resp = if self.creds.get_token_pair_option().is_some() {
            self.https_client
                .clone()
                .oauth1(self.creds.clone())
                .get(req_url)
                .send()
                .await?
        } else {
            self.https_client.clone().get(req_url).send().await?
        };

        // Rate Limits aren't returned for this kind of call

        // Check if the http error code returned was an error
        self.error_on_http_status(&resp, None)?;

        // Pull out the payload
        match resp.bytes().await {
            Ok(body) => Ok(Response {
                payload: Some(body),
                rate_limit: None,
            }),
            Err(err) => Err(SmugMugError::ApiResponseMalformed(err)),
        }
    }

    // Performs a PATCH request to the SmugMug API
    async fn patch<T: DeserializeOwned>(
        &self,
        url: &str,
        data: Vec<u8>,
        params: Option<&ApiParams<'_>>,
    ) -> Result<Response<T>, SmugMugError> {
        let req_url = self.create_req(url, params)?;
        let resp = self
            .https_client
            .clone()
            .oauth1(self.creds.clone())
            .patch(req_url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .body(data)
            .send()
            .await?;
        self.handle_json_response(resp).await
    }

    // Performs a POST request to the SmugMug API
    async fn post<T: DeserializeOwned>(
        &self,
        url: &str,
        data: Vec<u8>,
        params: Option<&ApiParams<'_>>,
    ) -> Result<Response<T>, SmugMugError> {
        let req_url = self.create_req(url, params)?;
        let resp = self
            .https_client
            .clone()
            .oauth1(self.creds.clone())
            .post(req_url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .body(data)
            .send()
            .await?;
        self.handle_json_response(resp).await
    }

    // Parse the rate limit headers that are returned.
    fn extract_rate_limits_from_response(&self, resp: &ReqwestResponse) -> Arc<RateLimitWindow> {
        // Extract the rate limits
        let rate_limit = Arc::new(RateLimitWindow::from_reqwest_headers(
            Utc::now(),
            resp.headers(),
        ));

        // Write the latest value to the clients stores last rate window update area
        self.last_rate_window
            .write()
            .map(|mut v| {
                *v = rate_limit.clone();
            })
            .unwrap();
        rate_limit
    }

    // Returns an error on an http error
    fn error_on_http_status(
        &self,
        resp: &ReqwestResponse,
        rate_limit: Option<&RateLimitWindow>,
    ) -> Result<(), SmugMugError> {
        let _ = resp.error_for_status_ref().map_err(|v| {
            match rate_limit.and_then(|v| v.retry_after_seconds()) {
                Some(retry_after) if resp.status().as_u16() == 429 => {
                    SmugMugError::ApiResponseTooManyRequests(retry_after)
                }
                _ => SmugMugError::from(v),
            }
        })?;
        Ok(())
    }

    // Response handling logic
    async fn handle_json_response<T: DeserializeOwned>(
        &self,
        resp: ReqwestResponse,
    ) -> Result<Response<T>, SmugMugError> {
        // Get current rate limit values
        let rate_limit = self.extract_rate_limits_from_response(&resp);

        // Check if the http error code returned was an error
        self.error_on_http_status(&resp, Some(&rate_limit))?;

        // Pull out the payload
        match resp.json::<ResponseBody<T>>().await {
            Ok(body) => {
                if !body.is_code_an_error()? {
                    return Err(SmugMugError::ApiResponse(body.code, body.message));
                }
                Ok(Response {
                    payload: body.response,
                    rate_limit: Some(rate_limit),
                })
            }
            Err(err) => Err(SmugMugError::ApiResponseMalformed(err)),
        }
    }

    // Creates the request with the given params
    fn create_req(
        &self,
        url: &str,
        params: Option<&ApiParams<'_>>,
    ) -> Result<reqwest::Url, SmugMugError> {
        let mut req_url = params.map_or(reqwest::Url::parse(url), |v| {
            reqwest::Url::parse_with_params(url, v)
        })?;

        if self.creds.access_token.is_none() || self.creds.token_secret.is_none() {
            req_url = reqwest::Url::parse_with_params(
                req_url.as_str(),
                [("APIKey", &self.creds.consumer_api_key)],
            )?;
        }

        Ok(req_url)
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiClient").finish()
    }
}

/// This can be filter types as well as other parameters the specific API expects
pub type ApiParams<'a> = [(&'a str, &'a str)];

/// API Error codes per the SmugMug API site
#[derive(Debug, TryFromPrimitive)]
#[repr(u32)]
pub enum ApiErrorCodes {
    // Successful Codes
    Ok = 200,
    CreatedSuccessfully = 201,
    Accepted = 202,
    MovedPermanently = 301,
    MovedTemporarily = 302,

    // Failing Codes
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    BadAccept = 406,
    Conflict = 407,
    TooManyRequests = 429,
    InternalServerError = 500,
    ServiceUnavailable = 503,
}

/// The call rate limits returned from the REST API call.
#[derive(Default, Clone)]
pub struct RateLimitWindow {
    num_remaining_requests: Option<u64>,
    current_window_reset_datetime: Option<DateTime<Utc>>,
    retry_after_seconds: Option<u64>,
    timestamp: DateTime<Utc>,
}

impl RateLimitWindow {
    // Pull out the rate limit headers
    fn from_reqwest_headers(timestamp: DateTime<Utc>, headers: &HeaderMap) -> Self {
        // Parse the headers fully per call since we are rate limited and taking time to parse
        // is potentially better than storing raw and performance of parsing on call.
        let retry_after_seconds = headers
            .get("retry-after")
            .and_then(|v| v.to_str().map_or(None, |v| v.parse().ok()));

        let num_remaining_requests = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().map_or(None, |v| v.parse().ok()));

        let current_window_reset_datetime = headers.get("x-ratelimit-reset").and_then(|v| {
            v.to_str().map_or(None, |v| {
                v.parse::<i64>()
                    .ok()
                    .and_then(|v| Utc.timestamp_opt(v, 0).latest())
            })
        });

        RateLimitWindow {
            num_remaining_requests,
            current_window_reset_datetime,
            retry_after_seconds,
            timestamp,
        }
    }

    /// Number of remaining requests until a [`SmugMugError::ApiResponseTooManyRequests`] error
    pub fn num_remaining_requests(&self) -> Option<u64> {
        self.num_remaining_requests
    }

    /// UTC Date/Time until current rate window ends
    pub fn window_reset_datetime(&self) -> Option<DateTime<Utc>> {
        self.current_window_reset_datetime
    }

    /// If the [`SmugMugError::ApiResponseTooManyRequests`] error is returned this will be the
    /// number of seconds until API calls can be resumed
    pub fn retry_after_seconds(&self) -> Option<u64> {
        self.retry_after_seconds
    }

    /// Returns true if either [`Self::num_remaining_requests`] or [`Self::retry_after_seconds`] are not None
    pub fn is_valid(&self) -> bool {
        self.num_remaining_requests.is_some() || self.retry_after_seconds.is_some()
    }

    /// If the caller has used up the number of request allocated, then the time returned indicates
    /// when requests can be resumed
    pub fn resume_after(&self) -> Option<DateTime<Utc>> {
        self.retry_after_seconds
            .map(|v| self.timestamp + Duration::seconds(v as i64))
    }

    /// Timestamp this was recorded
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Response returned by the [`Client`]` get/get_binary_data/post/patch/requests
pub struct Response<T> {
    /// Request data being returned.
    pub payload: Option<T>,
    /// Returns rate limit for the last call.  This will be None for binary data retrieval.
    pub rate_limit: Option<Arc<RateLimitWindow>>,
}

/// Holds credentials used for accessing/signing REST requests
#[derive(Default, Clone)]
pub struct Creds {
    consumer_api_key: String,
    consumer_api_secret: Option<String>,
    access_token: Option<String>,
    token_secret: Option<String>,
}

impl Creds {
    /// Creates credentials from the tokens
    /// Only the consumer_api_key is required for working with public SmugMug Accounts
    pub fn from_tokens(
        consumer_api_key: &str,
        consumer_api_secret: Option<&str>,
        access_token: Option<&str>,
        token_secret: Option<&str>,
    ) -> Self {
        Self {
            consumer_api_key: consumer_api_key.into(),
            consumer_api_secret: consumer_api_secret.map(|v| v.into()),
            access_token: access_token.map(|v| v.into()),
            token_secret: token_secret.map(|v| v.into()),
        }
    }
}

impl std::fmt::Debug for Creds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Creds")
            .field("consumer_api_key", &"xxx")
            .field(
                "consumer_api_secret",
                &self.consumer_api_secret.as_ref().map_or("", |_| "xxx"),
            )
            .field(
                "access_token",
                &self.access_token.as_ref().map_or("", |_| "xxx"),
            )
            .field(
                "token_secret",
                &self.access_token.as_ref().map_or("", |_| "xxx"),
            )
            .finish()
    }
}

// Internally this makes it easier to pass into reqwest for signing
impl SecretsProvider for Creds {
    fn get_consumer_key_pair(&self) -> (&str, &str) {
        (
            self.consumer_api_key.as_str(),
            self.consumer_api_secret
                .as_deref()
                .expect("`consumer_api_secret` is required"),
        )
    }

    fn get_token_pair_option(&self) -> Option<(&str, &str)> {
        self.access_token
            .as_deref()
            .zip(self.token_secret.as_deref())
    }

    fn get_token_option_pair(&self) -> (Option<&str>, Option<&str>) {
        (self.access_token.as_deref(), self.token_secret.as_deref())
    }
}

// Base expected response body to be returned from the API
#[derive(Deserialize, Debug)]
struct ResponseBody<ResponseType> {
    #[serde(rename = "Code")]
    code: u32,

    #[serde(rename = "Message")]
    message: String,

    #[serde(rename = "Response")]
    response: Option<ResponseType>,
}

impl<ResponseType> ResponseBody<ResponseType> {
    /// Determine if the code returned in the response body is an error based on SmugMug API Docs
    fn is_code_an_error(&self) -> Result<bool, SmugMugError> {
        use ApiErrorCodes as E;
        match ApiErrorCodes::try_from(self.code)? {
            E::Accepted
            | E::Ok
            | E::CreatedSuccessfully
            | E::MovedPermanently
            | E::MovedTemporarily => Ok(true),
            _ => Ok(false),
        }
    }
}
