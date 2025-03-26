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
//! use smugmug::v2::{Album, Client, Creds, NodeTypeFilters, SortDirection, SortMethod, User, SmugMugError};
//! use futures::{pin_mut, StreamExt};
//!
//!async fn iterate_albums<Fut>(
//!    api_key: &str,
//!    api_secret: &str,
//!    access_token: &str,
//!    access_token_secret: &str,
//!    album_op: impl Fn(Album) -> Fut,
//!) -> anyhow::Result<()>
//!where
//!    Fut: Future<Output=anyhow::Result<bool>>,
//!{
//!    // The API key/secret is obtained from your SmugMug account
//!    // The Access Token/Secret is obtained via Oauth1 process external to this
//!    let client = Client::new(Creds::from_tokens(
//!         api_key,
//!         api_secret,
//!         Some(access_token),
//!         Some(access_token_secret),
//!     ));
//!
//!    // Get information for the authenticated user
//!    let user_info = User::authenticated_user_info(client.clone()).await?;
//!
//!    // Get information on the root node for this user
//!    let node_info = user_info.node().await?;
//!
//!    // Retrieve the Albums under the root node
//!    let node_children = node_info.children(
//!        NodeTypeFilters::Album,
//!        SortDirection::Descending,
//!        SortMethod::Organizer,
//!    );
//!    // Iterate over the node children
//!    pin_mut!(node_children);
//!    while let Some(Ok(child_album_node)) = node_children.next().await {
//!        // Retrieve album specific information about this child node
//!        let album_info = child_album_node.album().await?;
//!
//!        // Do operation on album and stop stream if returns false
//!        if !album_op(album_info).await? {
//!            break;
//!        }
//!    }
//!    Ok(())
//!}
//! ```
//!
pub mod v2;
