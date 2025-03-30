# SmugMug REST API Client Library

[![GitHub Repo](https://img.shields.io/badge/github-cch71%2Fsmugmug-green?logo=github)](https://github.com/cch71/smugmug_rs.git)
[![Static Badge](https://img.shields.io/badge/crate.io-smugmug-blue?logo=rust)](https://crates.io/crates/smugmug)
[![docs.rs](https://img.shields.io/docsrs/smugmug?logo=docsdotrs)](https://docs.rs/smugmug/latest/smugmug)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/cch71/smugmug_rs/.github/workflows/build.yml?branch=main&logo=github)](https://github.com/cch71/smugmug_rs/actions/workflows/build.yml)
![License](https://img.shields.io/crates/l/smugmug)

This SmugMug library was created for working with the SmugMug API interface.
For further details on the Rest API refer to: [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/index.html)

## Features

- Retrieve Basic user information (Read only).
- Retrieve Node information.
  - Can create an Album.
  - List children of a Node.
- Retrieve Album information.
  - Can set the upload key.
  - Can list the images contained in an Album.
- Retrieve Image information.
  - Download of the archive image supported.
- Lower level interface for handling more direct communication.

_The SmugMug API uses OAuth1. This library handles the request signing.
Getting the Access Token/Secret is left up to the consumer of this library_

The [`v2::Client`] currently provides direct GET/PATCH/POST functionality to allow library usage
for features that may not be implemented yet in the higher level interfaces\*

## Usage

**You will need to acquire an API key/secret from SmugMug prior to using the API**

```rust
use futures::{pin_mut, StreamExt};
use smugmug::v2::{Album, Client, NodeTypeFilters, SortDirection, SortMethod, User};

async fn iterate_albums<Fut>(
    api_key: &str,
    api_secret: &str,
    access_token: &str,
    access_token_secret: &str,
    album_op: impl Fn(Album) -> Fut,
) -> Result<()>
where
    Fut: Future<Output=Result<bool>>,
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
```

## License

Licensed under either of <a href="LICENSE-APACHE">Apache License Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.

## Contributions

Contributions are welcome.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
