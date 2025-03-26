/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::errors::SmugMugError;
use const_format::formatcp;
use num_enum::TryFromPrimitive;
use reqwest::Response;
use reqwest_oauth1::{OAuthClientProvider, SecretsProvider};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::sync::Arc;

// Root SmugMug API
pub(crate) const API_ORIGIN: &str = "https://api.smugmug.com";

// When retrieving pages this is the default records to retrieve
pub(crate) const NUM_TO_GET: usize = 25;

// String representation of default number of records to retrieve
pub(crate) const NUM_TO_GET_STRING: &'static str = formatcp!("{}", NUM_TO_GET);

/// Handles the lower level communication with the SmugMug REST API.
#[derive(Default, Clone)]
pub struct Client {
    creds: Creds,
    https_client: reqwest::Client,
}

impl Client {
    /// Creates a new SmugMug client instance from the provided credentials
    pub fn new(creds: Creds) -> Arc<Self> {
        Arc::new(Self {
            creds,
            https_client: reqwest::Client::new(),
        })
    }

    fn create_req(&self, url: &str, params: Option<&ApiParams<'_>>) -> Result<reqwest::Url, SmugMugError> {
        let mut req_url = params.map_or(reqwest::Url::parse(&url), |v| {
            reqwest::Url::parse_with_params(&url, v)
        })?;

        if self.creds.access_token.is_none() || self.creds.token_secret.is_none() {
            req_url = reqwest::Url::parse_with_params(req_url.as_str(), [("APIKey", &self.creds.consumer_api_key)])?;
        }

        Ok(req_url)
    }

    /// Performs a GET request to the SmugMug API
    pub async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Option<&ApiParams<'_>>,
    ) -> Result<Option<T>, SmugMugError> {
        let req_url = self.create_req(url, params)?;

        // If we are in read-only mode we have to do this a little different.  Since other functions
        // require Oauth1 singing, this is only needed for get.
        let resp = if self.creds.get_token_pair_option().is_some() {
            self.https_client.clone().oauth1(self.creds.clone())
                .get(req_url)
                .header("Accept", "application/json")
                .send()
                .await?
        } else {
            self.https_client.clone()
                .get(req_url)
                .header("Accept", "application/json")
                .send()
                .await?
        };
        self.handle_response(resp).await
    }

    /// Performs a PATCH request to the SmugMug API
    pub async fn patch<T: DeserializeOwned>(
        &self,
        url: &str,
        data: Vec<u8>,
        params: Option<&ApiParams<'_>>,
    ) -> Result<Option<T>, SmugMugError> {
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
        self.handle_response(resp).await
    }

    /// Performs a POST request to the SmugMug API
    pub async fn post<T: DeserializeOwned>(
        &self,
        url: &str,
        data: Vec<u8>,
        params: Option<&ApiParams<'_>>,
    ) -> Result<Option<T>, SmugMugError> {
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
        self.handle_response(resp).await
    }

    // Response handling logic
    async fn handle_response<T: DeserializeOwned>(&self, resp: Response) -> Result<Option<T>, SmugMugError> {
        match resp.json::<ResponseBody<T>>().await {
            Ok(body) => {
                if !body.is_code_an_error()? {
                    return Err(SmugMugError::ApiResponse(body.code, body.message));
                }
                Ok(body.response)
            }
            Err(err) => {
                Err(SmugMugError::ApiResponseMalformed(err))
            }
        }
        // println!("{}", serde_json::to_string_pretty(&resp)?);
        // let resp = serde_json::from_value::<T>(resp)?;
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiClient")
            .finish()
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

/// Holds credentials used for accessing/signing REST requests
#[derive(Default, Clone)]
pub struct Creds {
    consumer_api_key: String,
    consumer_api_secret: String,
    access_token: Option<String>,
    token_secret: Option<String>,
}

impl Creds {
    /// Creates credentials from the tokens
    pub fn from_tokens(consumer_api_key: &str,
                       consumer_api_secret: &str,
                       access_token: Option<&str>,
                       token_secret: Option<&str>) -> Self
    {
        Self {
            consumer_api_key: consumer_api_key.into(),
            consumer_api_secret: consumer_api_secret.into(),
            access_token: access_token.map(|v| v.into()),
            token_secret: token_secret.map(|v| v.into()),
        }
    }
}

impl std::fmt::Debug for Creds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Creds")
            .field("consumer_api_key", &"xxx")
            .field("consumer_api_secret", &"xxx")
            .field("access_token", &self.access_token.as_ref().map_or("", |_| "xxx"))
            .field("token_secret", &self.access_token.as_ref().map_or("", |_| "xxx"))
            .finish()
    }
}

// Internally this makes it easier to pass into reqwest for signing
impl SecretsProvider for Creds {
    fn get_consumer_key_pair<'a>(&'a self) -> (&'a str, &'a str) {
        (
            self.consumer_api_key.as_str(),
            self.consumer_api_secret.as_str(),
        )
    }

    fn get_token_pair_option<'a>(&'a self) -> Option<(&'a str, &'a str)> {
        self.access_token.as_deref().zip(self.token_secret.as_deref())
    }

    fn get_token_option_pair<'a>(&'a self) -> (Option<&'a str>, Option<&'a str>) {
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
            E::Accepted | E::Ok | E::CreatedSuccessfully | E::MovedPermanently | E::MovedTemporarily => Ok(true),
            _ => Ok(false),
        }
    }
}

