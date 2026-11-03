/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::errors::SmugMugError;
use crate::v2::parsers::{from_lenient_string, from_lenient_u64};
use crate::v2::Client;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// The largest video rendition SmugMug has for an image, from the image's `!largestvideo`
/// endpoint.  Retrieve it with [`crate::v2::Image::largest_video`].
///
/// For videos an Image's `ArchivedUri`, `ArchivedMD5` and `ArchivedSize` describe a JPEG poster
/// frame, not the video.  This is where the playable file comes from.
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/image.html) for more
/// details.
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct LargestVideo {
    #[serde(skip)]
    pub(crate) client: Option<Client>,

    /// Download link for the video
    #[serde(default, rename = "Url")]
    pub url: String,

    /// Size of the video in bytes
    #[serde(rename = "Size")]
    pub size: u64,

    #[serde(default, rename = "MD5", skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    /// File extension, e.g. `mp4`
    #[serde(default, rename = "Ext")]
    pub ext: String,

    #[serde(
        default,
        rename = "Width",
        deserialize_with = "from_lenient_u64",
        skip_serializing_if = "Option::is_none"
    )]
    pub width: Option<u64>,

    #[serde(
        default,
        rename = "Height",
        deserialize_with = "from_lenient_u64",
        skip_serializing_if = "Option::is_none"
    )]
    pub height: Option<u64>,

    /// Duration in seconds as SmugMug returns it, e.g. `11.405`.  See [`Self::duration_seconds`].
    #[serde(
        default,
        rename = "Duration",
        deserialize_with = "from_lenient_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub duration: Option<String>,

    /// Bits per second as SmugMug returns it, e.g. `7681362.9110039`
    #[serde(
        default,
        rename = "Bitrate",
        deserialize_with = "from_lenient_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub bitrate: Option<String>,

    #[serde(default, rename = "Usable", skip_serializing_if = "Option::is_none")]
    pub is_usable: Option<bool>,

    #[serde(default, rename = "Watermarked", skip_serializing_if = "Option::is_none")]
    pub is_watermarked: Option<bool>,
}

impl LargestVideo {
    /// Duration in seconds
    pub fn duration_seconds(&self) -> Option<f64> {
        self.duration.as_ref().and_then(|v| v.trim().parse().ok())
    }

    /// Downloads the video data using the provided client
    pub async fn download_with_client(&self, client: Client) -> Result<Bytes, SmugMugError> {
        client
            .get_binary_data(&self.url, None)
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
    }

    /// Downloads the video data
    pub async fn download(&self) -> Result<Bytes, SmugMugError> {
        self.download_with_client(self.client.clone().ok_or(SmugMugError::ClientNotFound())?)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Trimmed from a live `!largestvideo` response
    #[test]
    fn parses_largest_video() {
        let video: LargestVideo = serde_json::from_value(json!({
            "Bitrate": "7681362.9110039", "Duration": "11.405", "Ext": "mp4", "Height": 1080,
            "MD5": "d0474a16e45ea087cd5eead0bbd94edb", "Size": 10950743,
            "Uri": "/api/v2/image/xVhzRzQ-0!largestvideo",
            "Uris": {"VideoSize1920": {"Uri": "/api/v2/video/xVhzRzQ-0!size1920"}},
            "Url": "https://photos.smugmug.com/video.mp4", "Usable": true,
            "Watermarked": false, "Width": 1920
        }))
        .unwrap();
        assert_eq!(video.url, "https://photos.smugmug.com/video.mp4");
        assert_eq!(video.size, 10950743);
        assert_eq!(video.md5.as_deref(), Some("d0474a16e45ea087cd5eead0bbd94edb"));
        assert_eq!(video.ext, "mp4");
        assert_eq!((video.width, video.height), (Some(1920), Some(1080)));
        assert_eq!(video.duration_seconds(), Some(11.405));
        assert_eq!(video.bitrate.as_deref(), Some("7681362.9110039"));
        assert_eq!(video.is_usable, Some(true));
    }
}
