pub mod errors;
pub mod v2;

#[cfg(test)]
mod tests {
    use crate::v2::{Client, NodeTypeFilters, SortDirection, SortMethod};
    use anyhow::Result;
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
        let node_children = node_info
            .children(
                NodeTypeFilters::Album,
                SortDirection::Descending,
                SortMethod::Organizer,
            );
        pin_mut!(node_children);
        while let Some(result_album_node) = node_children.next().await {
            println!("Child Node: {:?}", result_album_node.unwrap());
        }
    }
}
