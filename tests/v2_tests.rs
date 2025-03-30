/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
mod helpers;

#[cfg(test)]
mod test {
    use crate::helpers;
    use chrono::Utc;
    use dotenvy::dotenv;
    use futures::{StreamExt, pin_mut};
    use smugmug::v2::{
        Album, Client, Image, Node, NodeTypeFilters, SortDirection, SortMethod, User,
    };

    #[tokio::test]
    async fn user_from_id() {
        dotenv().ok();
        let creds = helpers::get_read_only_auth_tokens().unwrap();
        let client = Client::new(creds);
        assert!(client.get_last_rate_limit_window_update().is_none());
        let user_info = User::from_id(client.clone(), "apidemo").await.unwrap();
        let rate_limit = client.get_last_rate_limit_window_update().unwrap();
        assert!(Utc::now() < rate_limit.window_reset_datetime().unwrap());
        assert!(0 != rate_limit.num_remaining_requests().unwrap());

        println!("User info: {:?}", user_info);
    }

    // Disabling for ci/cd builds since I would need to get an access token/secret
    #[ignore]
    #[tokio::test]
    async fn authenticated_user_info() {
        dotenv().ok();
        let creds = helpers::get_full_auth_tokens().unwrap();
        let client = Client::new(creds);
        assert!(client.get_last_rate_limit_window_update().is_none());
        let user_info = User::authenticated_user_info(client.clone()).await.unwrap();
        assert!(client.get_last_rate_limit_window_update().is_some());
        println!("User info: {:?}", user_info);
    }

    #[tokio::test]
    async fn node_from_id_and_children() {
        dotenv().ok();
        let creds = helpers::get_read_only_auth_tokens().unwrap();
        let client = Client::new(creds);
        // Using API Demo root node id
        let node_info = Node::from_id(client.clone(), "2StTX5").await.unwrap();
        println!("Node info: {:?}", node_info);
        let node_children = node_info.children(
            NodeTypeFilters::Any,
            SortDirection::Ascending,
            SortMethod::DateAdded,
        );

        // Iterate over the node children
        let mut node_count: u64 = 0;
        pin_mut!(node_children);
        while let Some(node_result) = node_children.next().await {
            let _ = node_result.unwrap();
            node_count += 1;
        }
        assert!(node_info.has_children && node_count > 0);
    }

    #[tokio::test]
    async fn get_multiple_nodes() {
        dotenv().ok();
        let creds = helpers::get_read_only_auth_tokens().unwrap();
        let client = Client::new(creds);

        // Using API Demo and cmac root node id
        let nodes = Node::from_id_slice(client, &["2StTX5", "XWx8t"])
            .await
            .unwrap();

        assert_eq!(nodes.len(), 2);
    }

    #[tokio::test]
    async fn album_from_id_and_images() {
        dotenv().ok();
        let creds = helpers::get_read_only_auth_tokens().unwrap();
        let client = Client::new(creds);
        // Using API Demo album id
        let album_info = Album::from_id(client.clone(), "pPJnZx").await.unwrap();
        println!("Album info: {:?}", album_info);

        let images = album_info.images();
        let mut image_count: u64 = 0;
        pin_mut!(images);
        while let Some(image_result) = images.next().await {
            let _ = image_result.unwrap();
            image_count += 1;
        }
        assert_eq!(album_info.image_count, image_count);
    }

    #[tokio::test]
    async fn get_multiple_albums() {
        dotenv().ok();
        let creds = helpers::get_read_only_auth_tokens().unwrap();
        let client = Client::new(creds);

        // Using cmac demo account to get 2 albums
        let objs = Album::from_id_slice(client, &["RJHXVN", "TrBCmb"])
            .await
            .unwrap();

        assert_eq!(objs.len(), 2);
    }

    #[tokio::test]
    async fn image_from_id() {
        dotenv().ok();
        let creds = helpers::get_read_only_auth_tokens().unwrap();
        let client = Client::new(creds);
        assert!(client.get_last_rate_limit_window_update().is_none());
        // Using CMAC example image id
        let image_info = Image::from_id(client.clone(), "jPPKD2c").await.unwrap();
        assert!(client.get_last_rate_limit_window_update().is_some());
        println!("Image info: {:?}", image_info);

        // Download image and verify data is good
        let image_md5sum = image_info.archived_md5.as_ref().unwrap();
        let image_size = image_info.archived_size.unwrap();
        let image_data = image_info.get_archive().await.unwrap();

        assert_eq!(image_data.len(), image_size as usize);

        let digest = md5::compute(image_data);
        assert_eq!(&format!("{:x}", digest), image_md5sum);
    }

    #[tokio::test]
    async fn get_multiple_images() {
        dotenv().ok();
        let creds = helpers::get_read_only_auth_tokens().unwrap();
        let client = Client::new(creds);

        // Using cmac demo account to get 2 images
        let objs = Image::from_id_slice(client, &["jPPKD2c", "F9sMpg5"])
            .await
            .unwrap();

        assert_eq!(objs.len(), 2);
    }
}
