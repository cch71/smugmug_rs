/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */

pub mod errors;
pub mod v2;

// #![feature(async_stream)]

#[cfg(test)]
mod tests {
    use crate::v2::{Client, NodeTypeFilters, SortDirection, SortMethod};
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
        let smugmug = Client::new(&api_key, &api_secret, &tokens.token, &tokens.secret);
        let user_info = smugmug.authenticated_user_info().await.unwrap();
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
                || last_updated_cutoff_dt > album_info.images_last_updated
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
