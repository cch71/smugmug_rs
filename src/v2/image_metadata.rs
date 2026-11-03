/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::Image;
use crate::v2::parsers::{from_lenient_f64, from_lenient_string, from_lenient_u64};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

/// EXIF/IPTC metadata SmugMug read from an image's file when it was uploaded.
///
/// SmugMug keeps this even when the downloadable file has had its EXIF stripped.  The common
/// fields are typed; empty values are `None`.  Every field the API returned (about 78) is kept
/// in [`Self::fields`], and serializing writes that map back out unchanged.
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/image-metadata.html) for
/// more details on the individual fields.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImageMetadata {
    pub uri: Option<String>,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub keywords: Option<String>,
    pub author: Option<String>,
    pub copyright: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens: Option<String>,
    pub serial_number: Option<String>,
    pub software: Option<String>,
    pub date_time_created: Option<String>,
    pub date_digitized: Option<String>,
    pub date_time_modified: Option<String>,
    pub micro_date_time_created: Option<String>,
    pub exposure: Option<String>,
    pub aperture: Option<String>,
    pub focal_length: Option<String>,
    pub focal_length_35mm: Option<String>,
    pub flash: Option<String>,
    pub white_balance: Option<String>,
    pub exposure_mode: Option<String>,
    pub exposure_program: Option<String>,
    pub metering: Option<String>,
    pub color_space: Option<String>,
    pub altitude: Option<String>,
    pub duration: Option<String>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub iso: Option<u64>,

    /// Every field returned by the API, keyed by the SmugMug field name
    pub fields: Map<String, Value>,
}

impl ImageMetadata {
    /// Returns the raw value of the given SmugMug field (e.g. `"SceneCaptureType"`)
    pub fn get(&self, field: &str) -> Option<&Value> {
        self.fields.get(field)
    }

    /// Returns true if SmugMug has a location for this image.  SmugMug reports a 0/0
    /// latitude/longitude when there isn't one.
    pub fn has_location(&self) -> bool {
        matches!((self.latitude, self.longitude), (Some(lat), Some(lon)) if lat != 0.0 || lon != 0.0)
    }
}

impl<'de> Deserialize<'de> for ImageMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let fields = Map::<String, Value>::deserialize(deserializer)?;
        let typed: TypedImageMetadata = serde_json::from_value(Value::Object(fields.clone()))
            .map_err(serde::de::Error::custom)?;
        Ok(Self {
            uri: typed.uri,
            title: typed.title,
            caption: typed.caption,
            keywords: typed.keywords,
            author: typed.author,
            copyright: typed.copyright,
            make: typed.make,
            model: typed.model,
            lens: typed.lens,
            serial_number: typed.serial_number,
            software: typed.software,
            date_time_created: typed.date_time_created,
            date_digitized: typed.date_digitized,
            date_time_modified: typed.date_time_modified,
            micro_date_time_created: typed.micro_date_time_created,
            exposure: typed.exposure,
            aperture: typed.aperture,
            focal_length: typed.focal_length,
            focal_length_35mm: typed.focal_length_35mm,
            flash: typed.flash,
            white_balance: typed.white_balance,
            exposure_mode: typed.exposure_mode,
            exposure_program: typed.exposure_program,
            metering: typed.metering,
            color_space: typed.color_space,
            altitude: typed.altitude,
            duration: typed.duration,
            video_codec: typed.video_codec,
            audio_codec: typed.audio_codec,
            city: typed.city,
            state: typed.state,
            country: typed.country,
            latitude: typed.latitude,
            longitude: typed.longitude,
            iso: typed.iso,
            fields,
        })
    }
}

impl Serialize for ImageMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.fields.serialize(serializer)
    }
}

// The typed subset of the metadata fields
#[derive(Deserialize)]
struct TypedImageMetadata {
    /// Uri of the metadata endpoint for this image
    #[serde(default, rename = "Uri", deserialize_with = "from_lenient_string")]
    pub uri: Option<String>,
    #[serde(default, rename = "Title", deserialize_with = "from_lenient_string")]
    pub title: Option<String>,
    #[serde(default, rename = "Caption", deserialize_with = "from_lenient_string")]
    pub caption: Option<String>,
    #[serde(default, rename = "Keywords", deserialize_with = "from_lenient_string")]
    pub keywords: Option<String>,
    #[serde(default, rename = "Author", deserialize_with = "from_lenient_string")]
    pub author: Option<String>,
    #[serde(
        default,
        rename = "Copyright",
        deserialize_with = "from_lenient_string"
    )]
    pub copyright: Option<String>,
    /// Camera maker
    #[serde(default, rename = "Make", deserialize_with = "from_lenient_string")]
    pub make: Option<String>,
    /// Camera model
    #[serde(default, rename = "Model", deserialize_with = "from_lenient_string")]
    pub model: Option<String>,
    #[serde(default, rename = "Lens", deserialize_with = "from_lenient_string")]
    pub lens: Option<String>,
    /// Camera serial number
    #[serde(
        default,
        rename = "SerialNumber",
        deserialize_with = "from_lenient_string"
    )]
    pub serial_number: Option<String>,
    /// Software or firmware that wrote the file
    #[serde(default, rename = "Software", deserialize_with = "from_lenient_string")]
    pub software: Option<String>,
    /// Capture time as written in the file. Usually local time without an offset, e.g. `2026-08-08T08:31:55`
    #[serde(
        default,
        rename = "DateTimeCreated",
        deserialize_with = "from_lenient_string"
    )]
    pub date_time_created: Option<String>,
    /// Digitized time as written in the file, usually without an offset
    #[serde(
        default,
        rename = "DateDigitized",
        deserialize_with = "from_lenient_string"
    )]
    pub date_digitized: Option<String>,
    #[serde(
        default,
        rename = "DateTimeModified",
        deserialize_with = "from_lenient_string"
    )]
    pub date_time_modified: Option<String>,
    /// Capture time with sub-second precision and, when the file has one, its offset
    #[serde(
        default,
        rename = "MicroDateTimeCreated",
        deserialize_with = "from_lenient_string"
    )]
    pub micro_date_time_created: Option<String>,
    /// Exposure time, e.g. `1/167`
    #[serde(default, rename = "Exposure", deserialize_with = "from_lenient_string")]
    pub exposure: Option<String>,
    /// F-number, e.g. `1.8`
    #[serde(default, rename = "Aperture", deserialize_with = "from_lenient_string")]
    pub aperture: Option<String>,
    /// e.g. `5.4 mm`
    #[serde(
        default,
        rename = "FocalLength",
        deserialize_with = "from_lenient_string"
    )]
    pub focal_length: Option<String>,
    #[serde(
        default,
        rename = "FocalLength35mm",
        deserialize_with = "from_lenient_string"
    )]
    pub focal_length_35mm: Option<String>,
    /// e.g. `No Flash`
    #[serde(default, rename = "Flash", deserialize_with = "from_lenient_string")]
    pub flash: Option<String>,
    #[serde(
        default,
        rename = "WhiteBalance",
        deserialize_with = "from_lenient_string"
    )]
    pub white_balance: Option<String>,
    #[serde(
        default,
        rename = "ExposureMode",
        deserialize_with = "from_lenient_string"
    )]
    pub exposure_mode: Option<String>,
    #[serde(
        default,
        rename = "ExposureProgram",
        deserialize_with = "from_lenient_string"
    )]
    pub exposure_program: Option<String>,
    #[serde(default, rename = "Metering", deserialize_with = "from_lenient_string")]
    pub metering: Option<String>,
    #[serde(
        default,
        rename = "ColorSpace",
        deserialize_with = "from_lenient_string"
    )]
    pub color_space: Option<String>,
    /// Altitude with its unit, e.g. `273.303 m`
    #[serde(default, rename = "Altitude", deserialize_with = "from_lenient_string")]
    pub altitude: Option<String>,
    /// Video duration with its unit, e.g. `11.40 s`
    #[serde(default, rename = "Duration", deserialize_with = "from_lenient_string")]
    pub duration: Option<String>,
    #[serde(
        default,
        rename = "VideoCodec",
        deserialize_with = "from_lenient_string"
    )]
    pub video_codec: Option<String>,
    #[serde(
        default,
        rename = "AudioCodec",
        deserialize_with = "from_lenient_string"
    )]
    pub audio_codec: Option<String>,
    #[serde(default, rename = "City", deserialize_with = "from_lenient_string")]
    pub city: Option<String>,
    #[serde(default, rename = "State", deserialize_with = "from_lenient_string")]
    pub state: Option<String>,
    #[serde(default, rename = "Country", deserialize_with = "from_lenient_string")]
    pub country: Option<String>,
    /// Decimal degrees. SmugMug reports 0 when the file has no location
    #[serde(default, rename = "Latitude", deserialize_with = "from_lenient_f64")]
    pub latitude: Option<f64>,
    /// Decimal degrees. SmugMug reports 0 when the file has no location
    #[serde(default, rename = "Longitude", deserialize_with = "from_lenient_f64")]
    pub longitude: Option<f64>,
    #[serde(default, rename = "ISO", deserialize_with = "from_lenient_u64")]
    pub iso: Option<u64>,
}

/// An image together with its metadata, as returned by [`crate::v2::Album::images_with_metadata`]
#[derive(Clone, Debug)]
pub struct ImageWithMetadata {
    pub image: Image,

    /// None if SmugMug didn't return metadata for the image
    pub metadata: Option<ImageMetadata>,
}

// An AlbumImage requested with `_expand=ImageMetadata&_expandmethod=inline`.  The expansion is
// placed at `Uris.ImageMetadata.ImageMetadata`.
#[derive(Deserialize, Debug)]
pub(crate) struct ExpandedAlbumImage {
    #[serde(flatten)]
    pub(crate) image: Image,

    #[serde(default, rename = "Uris")]
    pub(crate) uris: ExpandedImageUris,
}

#[derive(Deserialize, Debug, Default)]
pub(crate) struct ExpandedImageUris {
    #[serde(default, rename = "ImageMetadata")]
    pub(crate) image_metadata: Option<InlineImageMetadata>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct InlineImageMetadata {
    #[serde(default, rename = "ImageMetadata")]
    pub(crate) image_metadata: Option<ImageMetadata>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Trimmed from a live `!metadata` response
    fn photo_metadata() -> Value {
        json!({
            "Altitude": 0, "Aperture": "1.8", "AudioCodec": "", "Caption": "",
            "DateDigitized": "2026-08-08T08:31:55", "DateTimeCreated": "2026-08-08T08:31:55",
            "Exposure": "1/167", "Flash": "No Flash", "FocalLength": "5.4 mm", "ISO": 40,
            "Latitude": 0, "Longitude": 0, "Make": "samsung", "Model": "SM-G991U",
            "MicroDateTimeCreated": "2026-08-08T08:31:55.884-05:00",
            "SceneCaptureType": "Standard", "Software": "G991USQSJHZB1",
            "Uri": "/api/v2/image/Mzf2k5x-0!metadata"
        })
    }

    #[test]
    fn parses_photo_metadata() {
        let metadata: ImageMetadata = serde_json::from_value(photo_metadata()).unwrap();
        assert_eq!(metadata.make.as_deref(), Some("samsung"));
        assert_eq!(metadata.model.as_deref(), Some("SM-G991U"));
        assert_eq!(metadata.iso, Some(40));
        assert_eq!(metadata.aperture.as_deref(), Some("1.8"));
        assert_eq!(metadata.date_digitized.as_deref(), Some("2026-08-08T08:31:55"));
        assert_eq!(metadata.altitude.as_deref(), Some("0"));
        assert_eq!(metadata.caption, None);
        assert_eq!(metadata.audio_codec, None);
        assert!(!metadata.has_location());
        assert_eq!(metadata.get("SceneCaptureType"), Some(&json!("Standard")));
    }

    #[test]
    fn parses_video_metadata_with_string_numbers() {
        let metadata: ImageMetadata = serde_json::from_value(json!({
            "Altitude": "273.303 m", "Duration": "11.40 s", "ISO": "",
            "Latitude": 30.5568, "Longitude": "-97.7326", "VideoCodec": "H.264"
        }))
        .unwrap();
        assert_eq!(metadata.altitude.as_deref(), Some("273.303 m"));
        assert_eq!(metadata.duration.as_deref(), Some("11.40 s"));
        assert_eq!(metadata.iso, None);
        assert_eq!(metadata.latitude, Some(30.5568));
        assert_eq!(metadata.longitude, Some(-97.7326));
        assert!(metadata.has_location());
    }

    #[test]
    fn serializes_all_fields_unchanged() {
        let metadata: ImageMetadata = serde_json::from_value(photo_metadata()).unwrap();
        assert_eq!(serde_json::to_value(&metadata).unwrap(), photo_metadata());
        let round_trip: ImageMetadata =
            serde_json::from_str(&serde_json::to_string(&metadata).unwrap()).unwrap();
        assert_eq!(round_trip, metadata);
    }

    // Trimmed from a live `!images?_expand=ImageMetadata&_expandmethod=inline` response
    #[test]
    fn parses_inline_expanded_album_image() {
        let item: ExpandedAlbumImage = serde_json::from_value(json!({
            "Altitude": 0, "ArchivedMD5": "89288f2a2f85c804fa524e017b7e9b17",
            "ArchivedSize": 6023161, "Caption": "",
            "DateTimeOriginal": "2026-08-08T15:31:55+00:00",
            "DateTimeUploaded": "2026-08-08T21:49:34+00:00", "FileName": "20260808_083155.jpg",
            "Format": "JPG", "Hidden": false, "ImageKey": "Mzf2k5x", "IsVideo": false,
            "KeywordArray": [], "LastUpdated": "2026-08-08T21:49:35+00:00", "Latitude": "0",
            "Longitude": "0", "OriginalHeight": 3024, "OriginalSize": 6023161,
            "OriginalWidth": 4032, "Processing": false, "Serial": 0, "Title": "",
            "Uri": "/api/v2/album/pJk78v/image/Mzf2k5x-0",
            "Uris": {
                "Album": {"Uri": "/api/v2/album/pJk78v", "Locator": "Album"},
                "ImageMetadata": {
                    "Locator": "ImageMetadata", "LocatorType": "Object",
                    "Uri": "/api/v2/image/Mzf2k5x-0!metadata",
                    "ImageMetadata": photo_metadata()
                }
            }
        }))
        .unwrap();
        assert_eq!(item.image.image_key, "Mzf2k5x");
        assert_eq!(item.image.serial, Some(0));
        assert_eq!(item.image.original_width, Some(4032));
        assert_eq!(item.image.original_height, Some(3024));
        assert_eq!(item.image.original_size, Some(6023161));
        assert_eq!(
            item.image.date_time_original.unwrap().to_rfc3339(),
            "2026-08-08T15:31:55+00:00"
        );
        assert_eq!(item.image.key_with_serial(), "Mzf2k5x-0");
        let metadata = item.uris.image_metadata.unwrap().image_metadata.unwrap();
        assert_eq!(metadata.make.as_deref(), Some("samsung"));
    }
}
