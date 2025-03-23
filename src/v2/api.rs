use crate::errors::SmugMugError;
use reqwest_oauth1::{OAuthClientProvider, SecretsProvider};
use serde::de::DeserializeOwned;


pub const API_ORIGIN: &str = "https://api.smugmug.com";

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

    /// Returns the full raw json information for the authenticated user
    pub async fn authenticated_user_info<T: DeserializeOwned>(
        &self,
        params: Option<&ApiParams<'_>>,
    ) -> Result<T, SmugMugError> {
        let req_url = {
            let base_url = format!("{API_ORIGIN}/api/v2!authuser");
            params.map_or(reqwest::Url::parse(&base_url), |v| {
                reqwest::Url::parse_with_params(&base_url, v)
            })?
        };
        let resp = self
            .https_client
            .clone()
            .oauth1(self.creds.clone())
            .get(req_url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<T>()
            .await?;
        // println!("{}", serde_json::to_string_pretty(&resp)?);
        Ok(resp)
    }

    /// Returns the full raw json information for a node using the Node specific URI
    pub async fn node_info<T: DeserializeOwned>(
        &self,
        node_uri: &str,
        params: Option<&ApiParams<'_>>,
    ) -> Result<T, SmugMugError> {
        let req_url = params.map_or(reqwest::Url::parse(&node_uri), |v| {
                reqwest::Url::parse_with_params(&node_uri, v)
            })?;
        let resp = self
            .https_client
            .clone()
            .oauth1(self.creds.clone())
            .get(req_url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<T>()
            //.json::<serde_json::Value>()
            .await?;

        //println!("{}", serde_json::to_string_pretty(&resp)?);
        //let resp = serde_json::from_value::<T>(resp)?;
        Ok(resp)
    }

    /// Returns the full raw json information for an album using the provided Album specific URI
    pub async fn album_info<T: DeserializeOwned>(
        &self,
        album_uri: &str,
        params: Option<&ApiParams<'_>>,
    ) -> Result<T, SmugMugError> {
        // Same operation for a raw operation.  The only difference is the uri and response
        self.node_info::<T>(album_uri, params).await
    }

    /// Returns the full raw json information for an album using the provided Album specific URI
    pub async fn node_children<T: DeserializeOwned>(
        &self,
        node_uri: &str,
        params: Option<&ApiParams<'_>>,
    ) -> Result<T, SmugMugError> {
        // params = {
        //     │   "_verbosity": "1",
        //     │   "Type": "Album",
        //     │   "SortMethod": "Organizer",
        //     │   "SortDirection": "Descending",
        // }
        let req_url = params.map_or(reqwest::Url::parse(&node_uri), |v| {
            reqwest::Url::parse_with_params(&node_uri, v)
        })?;

        let resp = self
            .https_client
            .clone()
            .oauth1(self.creds.clone())
            .get(req_url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<T>()
            //.json::<serde_json::Value>()
            .await?;
        // println!("{}", serde_json::to_string_pretty(&resp)?);
        // let resp = serde_json::from_value::<T>(resp)?;
        Ok(resp)
    }
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

/// This can be filter types as well as other parameters the specific API expects
pub type ApiParams<'a> = [(&'a str, &'a str)];

#[derive(Default, Clone)]
struct Creds {
    consumer_api_key: String,
    consumer_api_secret: String,
    access_token: String,
    token_secret: String,
}

impl std::fmt::Debug for ApiClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiClient")
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

