/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */

//! # SmugMug
//!
//! This SmugMug library was created for working with the SmugMug APIv2 interface.
//!
//! For further details on the Rest API refer to the [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/index.html)
//!
//! ## Features
//!
//! - Basic user information (Read only)
//! - Node information
//!     - Can create an Album
//!     - List children of a Node
//! - Album information
//!     - Can set upload key
//!     - Can list the images contained in an Album
//! - Image information
//! - Lower level interface for handling the raw communication
//!
//! *The SmugMug API uses OAuth1. This library handles the request signing.
//! Getting the Access Token/Secret is left up to the consumer of this library*
//!
//! *If you want to use this library for more that is currently implemented, the
//! [`v2::Client`] is a way to make request/responses in a more direct way*
//!
//! ## Installation
//!
//! ```toml
//! [dependencies]
//! smugmug = "0.1.0"
//! ```
//!
//! ## Usage
//!
//! **You will need to acquire an API key/secret from SmugMug prior to using the API**
//!
//! ```rust
//! use smugmug::v2::{Client, NodeTypeFilters, SortDirection, SortMethod, User, SmugMugError};
//! use futures::{pin_mut, StreamExt};
//!
//! async fn iterate_albums(api_key: &str, api_secret: &str, access_token:&str, access_token_secret: &str )
//! -> Result<(), SmugMugError>
//! {
//!     // The API key/secret is obtained from your SmugMug account
//!     // The Access Token/Secret is obtained via Oauth1 process external to this library
//!     let client = Client::new(api_key, api_secret, access_token, access_token_secret);
//!
//!     // Get information for the authenticated user
//!     let user_info = User::authenticated_user_info(client.clone()).await?;
//!     println!("User info: {:?}", user_info);
//!
//!     // Get information on the root node for this user
//!     let node_info = user_info.node().await?;
//!     println!("{:?}", node_info);
//!
//!     // Retrieve the Albums under the root node
//!     let node_children = node_info.children(
//!         NodeTypeFilters::Album,
//!         SortDirection::Descending,
//!         SortMethod::Organizer,
//!     );
//!     // Iterate over the node children
//!     pin_mut!(node_children);
//!     while let Some(Ok(album_node)) = node_children.next().await {
//!         println!("Child Node: {:?}", album_node);
//!
//!         // Get Album infomation about this node
//!         let album_info = album_node.album().await?;
//!         println!("Child Album: {:?}", album_info);
//!     }
//!     Ok(())
//! }
//! ```
//!
pub mod v2;

#[cfg(test)]
mod tests {
    use crate::v2::{Client, NodeTypeFilters, SortDirection, SortMethod, User};
    use anyhow::Result;
    use chrono::{Duration, Utc};
    use dotenvy::dotenv;
    use futures::{pin_mut, StreamExt};
    use serde::Deserialize;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;

    #[derive(Deserialize, Debug)]
    struct SmugMugOauth1Token {
        token: String,
        secret: String,
    }

    fn get_smugmug_tokens(path: PathBuf) -> Result<SmugMugOauth1Token> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    #[tokio::test]
    async fn it_works() {
        dotenv().ok();
        let api_key = std::env::var("SMUGMUG_API_KEY").unwrap();
        let api_secret = std::env::var("SMUGMUG_API_SECRET").unwrap();
        let token_cache = std::env::var("SMUGMUG_AUTH_CACHE").unwrap();
        let tokens = get_smugmug_tokens(token_cache.into()).unwrap();
        let client = Client::new(&api_key, &api_secret, &tokens.token, &tokens.secret);
        let user_info = User::authenticated_user_info(client.clone()).await.unwrap();
        println!("User info: {:?}", user_info);
        let node_info = user_info.node().await.unwrap();
        println!("{:?}", node_info);
        let node_children = node_info.children(
            NodeTypeFilters::Album,
            SortDirection::Descending,
            SortMethod::Organizer,
        );
        pin_mut!(node_children);

        // Date to cutoff no matter what just in case some spam/maliciousness is happening
        let cutoff_from_date_created_dt = Utc::now() - Duration::days(60);

        // Date we use to give leeway from last change time.
        let last_updated_cutoff_dt = Utc::now() - Duration::days(45);

        while let Some(result_album_node) = node_children.next().await {
            let album_node = result_album_node.unwrap();
            // println!("Child Node: {:?}", album_node);
            let album_info = album_node.album().await.unwrap();
            //println!("Child Album: {:?}", album_info);

            if album_info.upload_key.is_none() {
                break;
            }
            if cutoff_from_date_created_dt > album_info.date_created
                || last_updated_cutoff_dt > album_info.last_updated
            {
                println!(
                    "Album to remove upload key Name: {} Image Count: {} {}",
                    &album_info.name,
                    &album_info.image_count,
                    album_info
                        .upload_key
                        .as_ref()
                        .map_or("".to_string(), |v| format!("Upload Key: {}", v)),
                );
                let album_info = album_info.clear_upload_key().await.unwrap();
                println!("Removed Upload Key From: {:?}", album_info);
            }
        }
    }
}
