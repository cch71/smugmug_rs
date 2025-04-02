/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */

extern crate smugmug;

use anyhow::Result;
use dotenvy::dotenv;
use futures::{pin_mut, StreamExt};
use serde::Deserialize;
use smugmug::v2::{Album, Client, Creds, Node, NodeTypeFilters, SortDirection, SortMethod, User};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

// Iterates over album nodes and retrieves album information.
// NOTE: This assumes there are albums at the provided node.
async fn for_each_album_off_of_node<Fut>(node: Node, album_op: impl Fn(Album) -> Fut) -> Result<()>
where
    Fut: Future<Output=Result<bool>>,
{
    // Retrieve the Albums under the root node
    let node_children = node.children(
        NodeTypeFilters::Album,
        SortDirection::Descending,
        SortMethod::Organizer,
    )?;

    // Iterate over the node children
    pin_mut!(node_children);
    while let Some(Ok(child_album_node)) = node_children.next().await {
        // Retrieve album specific information about this child node
        let album = child_album_node.album().await?;

        println!("Found album: {} web path:{}", album.name, album.web_uri);

        // Do operation on album and stop stream if returns false
        if !album_op(album).await? {
            break;
        }
    }

    Ok(())
}

// Retrieve the list of images in an album and download the first one found
// NOTE: the authenticated user should have an album that allows downloads for this to work.
async fn retrieve_first_image_from_album(album: Album) -> Result<bool> {
    let images = album.images()?;

    // No images so keep looking
    if album.image_count == 0 {
        return Ok(true);
    }

    pin_mut!(images);
    while let Some(Ok(image)) = images.next().await {
        println!("Found possible image: {} {:?}", &image.name, &image);
        // NOTE: archived image may not be there if is processing but not completely sure.
        if image.is_processing {
            continue;
        }
        println!("Downloading image: {}", &image.name);
        let image_data = image.get_archive().await.expect("expected image data");

        // Download image and verify data is good
        let image_md5sum = image.archived_md5.as_ref().expect("expected md5 sum");
        let image_size = image.archived_size.expect("expected image size");

        assert_eq!(image_data.len(), image_size as usize);

        let digest = md5::compute(image_data);
        assert_eq!(&format!("{:x}", digest), image_md5sum);
        println!(
            "Successfully Downloaded format: {},\tsize: {},\tfilename:{}",
            image.format, image_size, image.file_name
        );
        // We found our image so stop.
        return Ok(false);
    }

    // tell iterator to keep looking
    Ok(true)
}

// Find the root node of the authenticated user.
async fn find_authenticated_users_root_node(client: Client) -> Result<Node> {
    // Get information for the authenticated user
    let user = User::authenticated_user_info(client.clone()).await?;

    // Get information on the root node for this user
    let node = user.node().await?;
    Ok(node)
}

// main
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // The API key/secret is obtained from your SmugMug account.
    // The API key is the only required field for accessing public accounts.
    // The Access Token/Secret is obtained via the OAuth1 authentication process.
    let client = Client::new(get_full_auth_tokens()?);

    // Get node to look for albums off of.
    // NOTE: The authenticated user is the API Key owner.
    let root_node = find_authenticated_users_root_node(client).await?;
    println!(
        "Found name: {} web path:{}",
        root_node.name, root_node.web_uri
    );

    // Iterate over the albums.  This should be a stream as well however since this is used as an
    // example it is not.
    for_each_album_off_of_node(root_node, retrieve_first_image_from_album).await?;
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

fn get_full_auth_tokens() -> anyhow::Result<Creds> {
    let api_key = std::env::var("SMUGMUG_API_KEY")?;
    let api_secret = std::env::var("SMUGMUG_API_SECRET")?;
    let token_cache = std::env::var("SMUGMUG_AUTH_CACHE")?;
    let tokens = get_smugmug_tokens(token_cache.into())?;

    Ok(Creds::from_tokens(
        &api_key,
        Some(&api_secret),
        Some(&tokens.token),
        Some(&tokens.secret),
    ))
}
