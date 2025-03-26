SmugMug APIv2 Client Library
Parser for Rust source code
===========================

[![GitHub Repo](https://img.shields.io/badge/github-cch71%2Fsmugmug-green?logo=github)](https://github.com/cch71/smugmug_rs.git)
[![docs.rs](https://img.shields.io/docsrs/smugmug?logo=docsdotrs)]()
[![crates.io](https://img.shields.io/crates/dv/smugmug?logo=rust)]()
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/cch71/smugmug_rs/build?logo=github)]()
![License](https://img.shields.io/crates/l/smugmug)

This SmugMug library was created for working with the SmugMug APIv2 interface.
For further details on the Rest API refer to: [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/index.html)

## Features

- Basic user information (Read only)
- Node information
    - Can create an Album
    - List children of a Node
- Album information
    - Can set upload key
    - Can list the images contained in an Album
- Image information
- Lower level interface for handling the raw communication

*The SmugMug API uses OAuth1. This library handles the request signing.
Getting the Access Token/Secret is left up to the consumer of this library*

*In future versions I will provide more functionality. If you want
to use this library and the information/capability is not yet there then the
[`smugmug::v2::Client`] is a way to make request/responses in a more
direct way*

## Installation

```toml
[dependencies]
smugmug = "0.1.0"
```

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
    // The Access Token/Secret is obtained via Oauth1 process external to this
    let client = Client::new(Creds::from_tokens(
        api_key,
        api_secret,
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

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.

## Contributions

Contributions are welcome.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
