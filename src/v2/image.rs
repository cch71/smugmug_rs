/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::errors::SmugMugError;
use crate::v2::macros::{
    obj_from_url, obj_update_from_uri, obj_update_from_url, objs_from_id_slice,
};
use crate::v2::parsers::{from_lenient_datetime, from_lenient_u64};
use crate::v2::{API_ORIGIN, Client, ImageMetadata, LargestVideo};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// Holds information returned from the AlbumImage/Image API.
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/image.html) for more
/// details on the individual fields.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Image {
    #[serde(skip)]
    pub(crate) client: Option<Client>,

    #[serde(rename = "Uri")]
    pub uri: String,

    #[serde(rename = "Title")]
    pub name: String,

    #[serde(rename = "Caption", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "Altitude")]
    pub altitude: u64,

    #[serde(rename = "Latitude", skip_serializing_if = "Option::is_none")]
    pub latitude: Option<String>,

    #[serde(rename = "Longitude", skip_serializing_if = "Option::is_none")]
    pub longitude: Option<String>,

    #[serde(rename = "Format")]
    pub format: String,

    #[serde(rename = "FileName")]
    pub file_name: String,

    #[serde(rename = "ImageKey")]
    pub image_key: String,

    #[serde(rename = "KeywordArray")]
    pub keywords: Vec<String>,

    #[serde(rename = "ArchivedUri", skip_serializing_if = "Option::is_none")]
    pub archived_uri: Option<String>,

    #[serde(rename = "ArchivedMD5", skip_serializing_if = "Option::is_none")]
    pub archived_md5: Option<String>,

    #[serde(rename = "ArchivedSize", skip_serializing_if = "Option::is_none")]
    pub archived_size: Option<u64>,

    #[serde(rename = "Processing")]
    pub is_processing: bool,

    #[serde(rename = "IsVideo")]
    pub is_video: bool,

    #[serde(rename = "Hidden")]
    pub is_hidden: bool,

    #[serde(default, rename = "Watermarked")]
    pub is_watermarked: bool,

    // Album specific fields
    #[serde(rename = "DateTimeUploaded")]
    pub date_created: DateTime<Utc>,

    #[serde(rename = "LastUpdated")]
    pub last_updated: DateTime<Utc>,

    /// When the photo or video was taken, as SmugMug read it from the file
    #[serde(
        default,
        rename = "DateTimeOriginal",
        deserialize_with = "from_lenient_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_time_original: Option<DateTime<Utc>>,

    /// Revision of the image, bumped when the image is replaced.  Together with the image key
    /// (`KEY-SERIAL`) it identifies the image for the per image endpoints.
    #[serde(
        default,
        rename = "Serial",
        deserialize_with = "from_lenient_u64",
        skip_serializing_if = "Option::is_none"
    )]
    pub serial: Option<u64>,

    #[serde(
        default,
        rename = "OriginalWidth",
        deserialize_with = "from_lenient_u64",
        skip_serializing_if = "Option::is_none"
    )]
    pub original_width: Option<u64>,

    #[serde(
        default,
        rename = "OriginalHeight",
        deserialize_with = "from_lenient_u64",
        skip_serializing_if = "Option::is_none"
    )]
    pub original_height: Option<u64>,

    /// Size in bytes of the original upload
    #[serde(
        default,
        rename = "OriginalSize",
        deserialize_with = "from_lenient_u64",
        skip_serializing_if = "Option::is_none"
    )]
    pub original_size: Option<u64>,
}

impl Image {
    const BASE_URI: &'static str = "/api/v2/image/";

    /// Returns information for the image at the provided full url
    pub async fn from_url(client: Client, url: &str) -> Result<Self, SmugMugError> {
        obj_from_url!(client, url, ImageResponse, image)
    }

    /// Returns information for the specified image id
    pub async fn from_id(client: Client, id: &str) -> Result<Self, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join(Self::BASE_URI)?
            .join(id)?;
        Self::from_url(client, req_url.as_str()).await
    }

    /// Returns information for the list of image id
    pub async fn from_id_slice(
        client: Client,
        id_list: &[&str],
    ) -> Result<Vec<Self>, SmugMugError> {
        objs_from_id_slice!(client, id_list, Self::BASE_URI, ImagesResponse, images)
    }

    /// Updates this Image data fields
    pub async fn update_image_data_with_client(
        &self,
        client: Client,
        data: Vec<u8>,
    ) -> Result<Image, SmugMugError> {
        obj_update_from_uri!(client, self.uri.as_str(), data, ImageResponse, image)
    }

    /// Updates data for the provided image id using the given client
    pub async fn update_image_data_with_client_from_id(
        client: Client,
        data: Vec<u8>,
        id: &str,
    ) -> Result<Image, SmugMugError> {
        let req_url = url::Url::parse(API_ORIGIN)?
            .join(Self::BASE_URI)?
            .join(id)?;
        obj_update_from_url!(client, req_url.as_str(), data, ImageResponse, image)
    }

    /// Retrieves the image data found at the archive uri using the provided client.
    /// See [`Self::get_archive`].
    pub async fn get_archive_with_client(&self, client: Client) -> Result<Bytes, SmugMugError> {
        match self.archived_uri.as_ref() {
            Some(archived_uri) => client
                .get_binary_data(archived_uri, None)
                .await?
                .payload
                .ok_or(SmugMugError::ResponseMissing()),
            None => Err(SmugMugError::ImageArchiveNotFound(
                self.file_name.clone(),
                self.image_key.clone(),
            )),
        }
    }

    /// Retrieves the image data found at the archive uri using the internal client.
    ///
    /// For photos this is the original upload.  For videos it is only a JPEG poster frame; use
    /// [`Self::largest_video`] to get the video.
    pub async fn get_archive(&self) -> Result<Bytes, SmugMugError> {
        self.get_archive_with_client(self.client()?).await
    }

    /// Returns the image key with its serial (e.g. `KEY-0`), which the per image endpoints such
    /// as `!metadata` and `!largestvideo` require.  Without the serial SmugMug redirects, and the
    /// redirected request fails the OAuth1 signature check.
    pub fn key_with_serial(&self) -> String {
        // The uri ends with the key and serial (e.g. /api/v2/album/ALBUM/image/KEY-0)
        match self.uri.rsplit('/').next() {
            Some(last) if last.starts_with(&format!("{}-", self.image_key)) => last.to_string(),
            _ => match self.serial {
                Some(serial) => format!("{}-{}", self.image_key, serial),
                None => format!("{}-0", self.image_key),
            },
        }
    }

    // Url for one of the image's `!` endpoints
    fn endpoint_url(&self, endpoint: &str) -> Result<url::Url, SmugMugError> {
        Ok(url::Url::parse(API_ORIGIN)?
            .join(Self::BASE_URI)?
            .join(&format!("{}!{}", self.key_with_serial(), endpoint))?)
    }

    // The internal client
    fn client(&self) -> Result<Client, SmugMugError> {
        self.client.clone().ok_or(SmugMugError::ClientNotFound())
    }

    /// Retrieves the EXIF/IPTC metadata SmugMug holds for this image using the provided client.
    ///
    /// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/image-metadata.html)
    pub async fn metadata_with_client(
        &self,
        client: Client,
    ) -> Result<ImageMetadata, SmugMugError> {
        let req_url = self.endpoint_url("metadata")?;
        client
            .get::<ImageMetadataResponse>(req_url.as_str(), None)
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
            .map(|v| v.metadata)
    }

    /// Retrieves the EXIF/IPTC metadata SmugMug holds for this image
    pub async fn metadata(&self) -> Result<ImageMetadata, SmugMugError> {
        self.metadata_with_client(self.client()?).await
    }

    /// Retrieves information about the largest video rendition of this image using the provided
    /// client.  Returns None if there is no video for this image.
    ///
    /// For videos [`Self::archived_uri`] only points at a JPEG poster frame; use
    /// [`LargestVideo::download`] to get the video itself.
    pub async fn largest_video_with_client(
        &self,
        client: Client,
    ) -> Result<Option<LargestVideo>, SmugMugError> {
        let req_url = self.endpoint_url("largestvideo")?;
        match client
            .get::<LargestVideoResponse>(req_url.as_str(), None)
            .await
        {
            Ok(resp) => Ok(resp.payload.and_then(|v| v.largest_video).map(|mut v| {
                v.client = Some(client.clone());
                v
            })),
            // Images that aren't videos return a 404.  For a video a 404 is a real error, such
            // as a deleted image or a stale serial, so it is returned.
            Err(SmugMugError::Request(err))
                if !self.is_video && err.status() == Some(reqwest::StatusCode::NOT_FOUND) =>
            {
                Ok(None)
            }
            Err(err) => Err(err),
        }
    }

    /// Retrieves information about the largest video rendition of this image.
    /// Returns None if there is no video for this image.
    pub async fn largest_video(&self) -> Result<Option<LargestVideo>, SmugMugError> {
        self.largest_video_with_client(self.client()?).await
    }
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.image_key == other.image_key
    }
}
impl Eq for Image {}

impl Hash for Image {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(self.image_key.as_bytes());
        let _ = state.finish();
    }
}

impl PartialOrd for Image {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Image {
    fn cmp(&self, other: &Self) -> Ordering {
        self.image_key.cmp(&other.image_key)
    }
}

impl std::fmt::Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "name: {}, filename: {} id: {}",
            self.name, self.file_name, self.image_key
        )
    }
}
// Expected response for a request to get an Image
#[derive(Deserialize, Debug)]
struct ImageResponse {
    #[serde(rename = "Image")]
    image: Image,
}
// Expected response for a request to get Images
#[derive(Deserialize, Debug)]
struct ImagesResponse {
    #[serde(rename = "Image")]
    images: Vec<Image>,
}

// Expected response for a request to get an Image's metadata
#[derive(Deserialize, Debug)]
struct ImageMetadataResponse {
    #[serde(rename = "ImageMetadata")]
    metadata: ImageMetadata,
}

// Expected response for a request to get an Image's largest video
#[derive(Deserialize, Debug)]
struct LargestVideoResponse {
    #[serde(rename = "LargestVideo")]
    largest_video: Option<LargestVideo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Image JSON as saved by older versions, without the newer fields
    fn old_image_json() -> serde_json::Value {
        json!({
            "Uri": "/api/v2/album/pJk78v/image/Mzf2k5x-0", "Title": "", "Altitude": 0,
            "Latitude": "0", "Longitude": "0", "Format": "JPG",
            "FileName": "20260808_083155.jpg", "ImageKey": "Mzf2k5x", "KeywordArray": [],
            "ArchivedSize": 6023161, "Processing": false, "IsVideo": false, "Hidden": false,
            "Watermarked": false, "DateTimeUploaded": "2026-08-08T21:49:34Z",
            "LastUpdated": "2026-08-08T21:49:35Z"
        })
    }

    #[test]
    fn deserializes_older_saved_json() {
        let image: Image = serde_json::from_value(old_image_json()).unwrap();
        assert_eq!(image.date_time_original, None);
        assert_eq!(image.serial, None);
        assert_eq!(image.original_size, None);
        // Saving it again doesn't add the missing fields
        assert_eq!(serde_json::to_value(&image).unwrap(), old_image_json());
    }

    #[test]
    fn bad_date_time_original_is_none() {
        let mut v = old_image_json();
        v["DateTimeOriginal"] = json!("");
        let image: Image = serde_json::from_value(v).unwrap();
        assert_eq!(image.date_time_original, None);
    }

    #[test]
    fn key_with_serial() {
        let mut image: Image = serde_json::from_value(old_image_json()).unwrap();
        assert_eq!(image.key_with_serial(), "Mzf2k5x-0");
        assert_eq!(
            image.endpoint_url("largestvideo").unwrap().as_str(),
            "https://api.smugmug.com/api/v2/image/Mzf2k5x-0!largestvideo"
        );

        image.uri = "/api/v2/image/Mzf2k5x-2".to_string();
        assert_eq!(image.key_with_serial(), "Mzf2k5x-2");

        // Falls back to the serial when the uri doesn't end with the key
        image.uri = "/api/v2/image/other".to_string();
        image.serial = Some(3);
        assert_eq!(image.key_with_serial(), "Mzf2k5x-3");
    }
}
