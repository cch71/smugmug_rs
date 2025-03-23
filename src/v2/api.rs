/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::errors::SmugMugError;
use num_enum::TryFromPrimitive;
use reqwest_oauth1::{OAuthClientProvider, SecretsProvider};
use serde::de::DeserializeOwned;
use serde::Deserialize;

// Root SmugMug API
pub const API_ORIGIN: &str = "https://api.smugmug.com";

/// Directly communicates with the API.
#[derive(Default, Clone)]
pub struct ApiClient {
    creds: Creds,
    https_client: reqwest::Client,
}

impl ApiClient {
    /// Creates a new SmugMug client instance from provided tokens
    pub fn new(
        consumer_api_key: &str,
        consumer_api_secret: &str,
        access_token: &str,
        token_secret: &str,
    ) -> Self {
        Self {
            creds: Creds {
                consumer_api_key: consumer_api_key.into(),
                consumer_api_secret: consumer_api_secret.into(),
                access_token: access_token.into(),
                token_secret: token_secret.into(),
            },
            https_client: reqwest::Client::new(),
        }
    }

    /// Performs a get request to the SmugMug API
    pub async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Option<&ApiParams<'_>>,
    ) -> Result<Option<T>, SmugMugError> {
        let req_url = params.map_or(reqwest::Url::parse(&url), |v| {
            reqwest::Url::parse_with_params(&url, v)
        })?;
        let resp = self
            .https_client
            .clone()
            .oauth1(self.creds.clone())
            .get(req_url)
            .header("Accept", "application/json")
            .send()
            .await?;
        match resp.json::<ResponseBody<T>>().await {
            Ok(body) => {
                if !body.is_code_an_error()? {
                    return Err(SmugMugError::ApiResponse(body.code, body.message));
                }
                Ok(body.response)
            }
            Err(err) => {
                println!("Api Malformed Err {:?}", err);
                Err(SmugMugError::ApiResponseMalformed())
            }
        }
        // println!("{}", serde_json::to_string_pretty(&resp)?);
        // let resp = serde_json::from_value::<T>(resp)?;
    }
}

impl std::fmt::Debug for ApiClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiClient")
            .finish()
    }
}


/// This can be filter types as well as other parameters the specific API expects
pub type ApiParams<'a> = [(&'a str, &'a str)];

/// Error codes per the SmugMug API site
#[derive(Debug, TryFromPrimitive)]
#[repr(u32)]
pub enum ApiErrorCodes {
    // Good Codes
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

#[derive(Default, Clone)]
struct Creds {
    consumer_api_key: String,
    consumer_api_secret: String,
    access_token: String,
    token_secret: String,
}

impl std::fmt::Debug for Creds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Creds")
            .field("consumer_api_key", &"xxx")
            .field("consumer_api_secret", &"xxx")
            .field("access_token", &"xxx")
            .field("token_secret", &"xxx")
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
        Some((self.access_token.as_str(), &self.token_secret))
    }

    fn get_token_option_pair<'a>(&'a self) -> (Option<&'a str>, Option<&'a str>) {
        (Some(self.access_token.as_str()), Some(&self.token_secret))
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

