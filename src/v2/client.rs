/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */


use crate::errors::SmugMugError;
use crate::v2::user::User;
use crate::v2::{ApiClient, API_ORIGIN};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum_macros::{EnumString, IntoStaticStr};


/// Example
// /// ```rust
// ///     use smugmug::SmugMug;
// ///     let api_key = "".to_string();
// ///     let api_secret = "".to_string();
// ///     let token = "".to_string();
// ///     let token_secret = "".to_string();
// ///     let smugmug_client = SmugMug::new(&api_key, &api_secret, &token, &token_secret);
// ///     smugmug_client.authenticated_user_info().await.unwrap();
// /// ```
#[derive(Debug, Clone)]
pub struct Client {
    api_client: Arc<ApiClient>,
}

impl Client {
    pub fn new(
        consumer_key: &str,
        consumer_secret: &str,
        access_token: &str,
        token_secret: &str,
    ) -> Self {
        Self {
            api_client: Arc::new(ApiClient::new(
                consumer_key,
                consumer_secret,
                access_token,
                token_secret,
            )),
        }
    }

    /// Returns information for the user at the provided full url
    pub async fn user_from_url(&self, url: &str) -> Result<User, SmugMugError> {
        let params = vec![("_verbosity", "1")];
        self
            .api_client
            .get::<UserResponse>(url, Some(&params))
            .await?
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.user.api_client = self.api_client.clone();
                v.user
            })
    }

    /// Returns information for the specified user id
    pub async fn user_from_id(&self, user_id: &str) -> Result<User, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join("/api/v2/user")?
            .join(user_id)?;
        self.user_from_url(req_url.as_str()).await
    }

    /// Returns information for the authenticated user
    pub async fn authenticated_user_info(&self) -> Result<User, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?.join("/api/v2!authuser")?;
        self.user_from_url(req_url.as_str()).await
    }
}


#[derive(Debug, EnumString, IntoStaticStr)]
pub enum SortMethod {
    Organizer,
    SortIndex,
    Name,
    DateAdded,
    DateModified,
}

#[derive(Debug, EnumString, IntoStaticStr)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Serialize, EnumString, IntoStaticStr)]
pub enum PrivacyLevel {
    Unknown,
    Public,
    Unlisted,
    Private,
}

#[derive(Debug, EnumString, IntoStaticStr)]
pub enum NodeTypeFilters {
    Any,
    Album,
    Folder,
    Page,
    #[strum(to_string = "System Album")]
    SystemAlbum,
    #[strum(to_string = "Folder Album Page")]
    FolderAlbumPage,
}

#[derive(Debug, EnumString, IntoStaticStr)]
pub enum NodeType {
    Unknown,
    Album,
    Folder,
    Page,
    #[strum(to_string = "System Folder")]
    SystemFolder,
    #[strum(to_string = "System Page")]
    SystemPage,
}

// Expected response from a User request
#[derive(Deserialize, Debug)]
struct UserResponse {
    #[serde(rename = "User")]
    user: User,
}



