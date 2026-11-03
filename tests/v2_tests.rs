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
    use crate::helpers::{get_full_client, get_read_only_client};
    use chrono::Utc;
    use futures::{StreamExt, pin_mut};
    use rand::RngExt;
    use rand::distr::Alphanumeric;
    use serde_json::json;
    use smugmug::v2::{
        Album, CreateAlbumProps, Image, Node, NodeType, NodeTypeFilters, PrivacyLevel,
        SortDirection, SortMethod, User,
    };

    #[tokio::test]
    async fn user_from_id() {
        let client = get_read_only_client();
        let user_info = User::from_id(client.clone(), "apidemo").await.unwrap();
        let rate_limit = client.get_last_rate_limit_window_update().unwrap();
        assert!(Utc::now() < rate_limit.window_reset_datetime().unwrap());
        assert!(0 != rate_limit.num_remaining_requests().unwrap());

        log::debug!("User info: {:?}", user_info);
    }

    // Disabling for ci/cd builds since I would need to get an access token/secret
    #[ignore]
    #[tokio::test]
    async fn authenticated_user_info() {
        let client = get_full_client();
        let user_info = User::authenticated_user_info(client.clone()).await.unwrap();
        assert!(client.get_last_rate_limit_window_update().is_some());
        log::debug!("User info: {:?}", user_info);
    }

    #[ignore]
    #[tokio::test]
    async fn getting_info_using_full_creds() {
        let client = get_full_client();
        let user_info = User::authenticated_user_info(client.clone()).await.unwrap();
        assert!(client.get_last_rate_limit_window_update().is_some());
        println!("User info: {:?}", user_info);

        // Uses Troop 27 Album
        let album_info = Album::from_id(client.clone(), "rxLLKT").await.unwrap();
        println!("Album info: {:?}", album_info);

        // Uses Troop 27 Image
        // The album key is StG5THQ but to get the information you need the -0 from the URI or Image Uri
        let image_info = Image::from_id(client.clone(), "StG5THQ-0").await.unwrap();
        println!("Image info: {:?}", image_info);
    }

    #[ignore]
    #[tokio::test]
    async fn creating_album_using_full_creds() {
        let client = get_full_client();
        let user_info = User::authenticated_user_info(client.clone()).await.unwrap();
        assert!(client.get_last_rate_limit_window_update().is_some());
        println!("User info: {:?}", user_info);

        // Creates this off of Troop 27's node
        let root_node = user_info.node().await.unwrap();
        let album = root_node
            .create_album(CreateAlbumProps {
                name: "Test Album".to_string(),
                privacy: Some(PrivacyLevel::Public),
                upload_key: Some("Stuffings".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        let upload_key: String = rand::rng()
            .sample_iter(Alphanumeric)
            .take(32) // Generates a 32-character long nonce
            .map(char::from)
            .collect();

        let album = album.set_upload_key(&upload_key).await.unwrap();

        let album_update_props = serde_json::to_vec(&json!({
            "AllowDownloads": true,
            "UploadKey": upload_key,
        }))
        .unwrap();
        let album = album
            .update_album_data_fields(album_update_props)
            .await
            .unwrap();

        println!("Album info: {album:?}");
    }

    #[tokio::test]
    async fn node_from_id_and_children() {
        let client = get_read_only_client();
        // Using API Demo root node id
        let node_info = Node::from_id(client.clone(), "2StTX5").await.unwrap();
        log::debug!("Node info: {:?}", node_info);
        let node_children = node_info
            .children(
                NodeTypeFilters::Any,
                SortDirection::Ascending,
                SortMethod::DateAdded,
            )
            .unwrap();

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
    async fn node_from_id_and_children_with_multi_pages() {
        let client = get_read_only_client();
        // Using Troop 27 root node
        let node_info = Node::from_id(client.clone(), "67t8MV").await.unwrap();
        log::debug!("Node info: {:?}", node_info);
        let node_children = node_info
            .children(
                NodeTypeFilters::Any,
                SortDirection::Ascending,
                SortMethod::DateAdded,
            )
            .unwrap();

        // Iterate over the node children
        let mut node_count: u64 = 0;
        pin_mut!(node_children);
        while let Some(node_result) = node_children.next().await {
            let _ = node_result.unwrap();
            node_count += 1;
        }
        assert!(node_count > 150);
        assert!(node_info.has_children && node_count > 0);
    }

    #[tokio::test]
    async fn get_album_id_from_node_id() {
        let client = get_read_only_client();
        // Using cmac album/node id /api/v2/album/SJT3DX
        let node_info = Node::from_id(client.clone(), "ZsfFs").await.unwrap();
        assert_eq!("SJT3DX", node_info.album_id().unwrap())
    }

    #[tokio::test]
    async fn get_multiple_nodes() {
        let client = get_read_only_client();

        // Using API Demo and cmac root node id
        let nodes = Node::from_id_slice(client, &["2StTX5", "XWx8t"])
            .await
            .unwrap();

        assert_eq!(nodes.len(), 2);
    }

    #[tokio::test]
    async fn album_from_id_and_images() {
        let client = get_read_only_client();

        // Using API Demo album id
        let album_info = Album::from_id(client.clone(), "pPJnZx").await.unwrap();
        log::debug!("Album info: {:?}", album_info);

        let images = album_info.images().unwrap();
        let mut image_count: u64 = 0;
        pin_mut!(images);
        while let Some(image_result) = images.next().await {
            let _ = image_result.unwrap();
            image_count += 1;
        }
        assert_eq!(album_info.image_count, image_count);
    }

    #[tokio::test]
    async fn album_from_id_and_images_with_multi_pages() {
        let client = get_read_only_client();

        // Using Troop 27 2024-11 Mini High Adventure album id
        let album_info = Album::from_id(client.clone(), "rxLLKT").await.unwrap();
        log::debug!("Album info: {:?}", album_info);

        let images = album_info.images().unwrap();
        let mut image_count: u64 = 0;
        pin_mut!(images);
        while let Some(image_result) = images.next().await {
            let _ = image_result.unwrap();
            image_count += 1;
        }
        assert!(image_count > 150);
        assert_eq!(album_info.image_count, image_count);
    }

    #[tokio::test]
    async fn get_multiple_albums() {
        let client = get_read_only_client();

        // Using cmac demo account to get 2 albums
        let objs = Album::from_id_slice(client, &["RJHXVN", "TrBCmb"])
            .await
            .unwrap();

        assert_eq!(objs.len(), 2);
    }

    #[tokio::test]
    async fn image_from_id() {
        let client = get_read_only_client();

        // Using CMAC example image id
        let image_info = Image::from_id(client.clone(), "jPPKD2c").await.unwrap();
        assert!(client.get_last_rate_limit_window_update().is_some());
        log::debug!("Image info: {:?}", image_info);

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
        let client = get_read_only_client();

        // Using cmac demo account to get 2 images
        let objs = Image::from_id_slice(client, &["jPPKD2c", "F9sMpg5"])
            .await
            .unwrap();

        assert_eq!(objs.len(), 2);
    }

    // The type filter used to be dropped from the request, so every child type came back
    #[tokio::test]
    async fn node_children_type_filter_is_applied() {
        let client = get_read_only_client();
        // Using Troop 27 root node
        let node_info = Node::from_id(client.clone(), "67t8MV").await.unwrap();
        let node_children = node_info
            .children(
                NodeTypeFilters::Album,
                SortDirection::Ascending,
                SortMethod::DateAdded,
            )
            .unwrap();

        let mut node_count: u64 = 0;
        pin_mut!(node_children);
        while let Some(node_result) = node_children.next().await {
            assert!(matches!(node_result.unwrap().node_type, NodeType::Album));
            node_count += 1;
        }
        assert!(node_count > 0);
    }

    #[ignore]
    #[tokio::test]
    async fn images_with_metadata_using_full_creds() {
        let client = get_full_client();
        // Uses a Troop 27 Album
        let album = Album::from_id(client.clone(), "pJk78v").await.unwrap();
        let images = album.images_with_metadata().unwrap();

        let mut count: u64 = 0;
        pin_mut!(images);
        while let Some(result) = images.next().await {
            let item = result.unwrap();
            assert!(item.image.serial.is_some());
            assert!(item.metadata.unwrap().fields().len() > 50);
            count += 1;
        }
        assert_eq!(count, album.image_count);
    }

    #[ignore]
    #[tokio::test]
    async fn image_metadata_and_largest_video_using_full_creds() {
        let client = get_full_client();

        // Uses a Troop 27 video
        let video = Image::from_id(client.clone(), "xVhzRzQ-0").await.unwrap();
        assert!(video.is_video);
        let metadata = video.metadata().await.unwrap();
        assert!(metadata.video_codec().is_some());

        let largest = video.largest_video().await.unwrap().unwrap();
        let data = largest.download().await.unwrap();
        assert_eq!(Some(data.len() as u64), largest.size);
        assert_eq!(
            Some(format!("{:x}", md5::compute(data))),
            largest.md5
        );

        // Uses a Troop 27 photo, which has no video
        let photo = Image::from_id(client.clone(), "Mzf2k5x-0").await.unwrap();
        assert!(photo.largest_video().await.unwrap().is_none());
    }

    #[ignore]
    #[tokio::test]
    async fn deleted_albums_and_folders_using_full_creds() {
        let client = get_full_client();
        let user_info = User::authenticated_user_info(client.clone()).await.unwrap();

        let albums = user_info.deleted_albums().unwrap();
        pin_mut!(albums);
        while let Some(album) = albums.next().await {
            let album = album.unwrap();
            assert!(!album.album_key().is_empty() && album.date_deleted.is_some());
        }

        let folders = user_info.deleted_folders().unwrap();
        pin_mut!(folders);
        while let Some(folder) = folders.next().await {
            assert!(!folder.unwrap().folder_key().is_empty());
        }
    }

    // Step 1 of the OAuth1 login, which needs no user input
    #[ignore]
    #[tokio::test]
    async fn oauth1_request_token_from_smugmug() {
        dotenvy::dotenv().ok();
        let token = smugmug::v2::oauth1_request_token(
            &std::env::var("SMUGMUG_API_KEY").unwrap(),
            &std::env::var("SMUGMUG_API_SECRET").unwrap(),
            "oob",
        )
        .await
        .unwrap();
        assert!(!token.token.is_empty() && !token.secret.is_empty());
    }
}
