/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */

extern crate smugmug;

use anyhow::Result;
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use futures::{StreamExt, pin_mut};
use serde::Deserialize;
use smugmug::v2::{Album, Client, Creds, NodeTypeFilters, SortDirection, SortMethod, User};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

async fn iterate_albums<Fut>(
    api_key: &str,
    api_secret: &str,
    access_token: &str,
    access_token_secret: &str,
    album_op: impl Fn(Album) -> Fut,
) -> Result<()>
where
    Fut: Future<Output = Result<bool>>,
{
    // The API key/secret is obtained from your SmugMug account
    // The API key is the only required field for accessing public accounts
    // The Access Token/Secret is obtained via the OAuth1 authentication process
    let client = Client::new(Creds::from_tokens(
        api_key,
        Some(api_secret),
        Some(access_token),
        Some(access_token_secret),
    ));

    // Get information for the authenticated user
    let user_info = User::authenticated_user_info(client.clone()).await?;

    // Get information on the root node for this user
    let node_info = user_info.node().await?;

    // Retrieve the Albums under the root node
    let node_children = node_info.children(
        NodeTypeFilters::Album,
        SortDirection::Descending,
        SortMethod::Organizer,
    );

    // Iterate over the node children
    pin_mut!(node_children);
    while let Some(Ok(child_album_node)) = node_children.next().await {
        // Retrieve album specific information about this child node
        let album_info = child_album_node.album().await?;

        // Do operation on album and stop stream if returns false
        if !album_op(album_info).await? {
            break;
        }
    }

    Ok(())
}

// Oauth tokens stored in cache json file
#[derive(Deserialize, Debug)]
struct SmugMugOauth1Token {
    token: String,
    secret: String,
}

// Retrieves the auth tokens.
fn get_smugmug_tokens(path: PathBuf) -> Result<SmugMugOauth1Token> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let api_key = std::env::var("SMUGMUG_API_KEY")?;
    let api_secret = std::env::var("SMUGMUG_API_SECRET")?;
    let token_cache = std::env::var("SMUGMUG_AUTH_CACHE")?;
    let tokens = get_smugmug_tokens(token_cache.into())?;

    // Date to cutoff no matter what just in case some spam/maliciousness is happening
    let cutoff_from_date_created_dt = Utc::now() - Duration::days(60);
    // Date we use to give leeway from last change time.
    let last_updated_cutoff_dt = Utc::now() - Duration::days(45);

    let cleaner = async |album_info: Album| {
        // Because of query sort order we can stop as soon as we get an album that doesn't have
        // a key
        if album_info.upload_key.is_none() {
            return Ok(false);
        }

        // Check cutoffs
        if cutoff_from_date_created_dt > album_info.date_created.unwrap()
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
        Ok(true)
    };

    // Iterate over the albums.  This should be a stream as well however since this is used as an
    // example it is not.
    iterate_albums(
        &api_key,
        &api_secret,
        &tokens.token,
        &tokens.secret,
        cleaner,
    )
    .await?;
    Ok(())
}
