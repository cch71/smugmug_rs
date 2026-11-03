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
/// fields have typed accessors such as [`Self::make`] and [`Self::iso`], which return `None`
/// for empty values.  Every field the API returned (about 78) is in [`Self::fields`], and
/// serializing writes that map back out unchanged.
///
/// The data is read-only, so the typed accessors and [`Self::fields`] always agree.  To change
/// it, take the map with [`Self::into_fields`] and deserialize a new value from it.
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/image-metadata.html) for
/// more details on the individual fields.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImageMetadata {
    // Parsed from `fields` when deserializing
    typed: TypedImageMetadata,

    fields: Map<String, Value>,
}

impl ImageMetadata {
    /// Every field returned by the API, keyed by the SmugMug field name
    pub fn fields(&self) -> &Map<String, Value> {
        &self.fields
    }

    /// Takes every field returned by the API, keyed by the SmugMug field name
    pub fn into_fields(self) -> Map<String, Value> {
        self.fields
    }

    /// Returns the raw value of the given SmugMug field (e.g. `"SceneCaptureType"`)
    pub fn get(&self, field: &str) -> Option<&Value> {
        self.fields.get(field)
    }

    /// Returns true if SmugMug has a location for this image.  SmugMug reports a 0/0
    /// latitude/longitude when there isn't one.
    pub fn has_location(&self) -> bool {
        matches!((self.latitude(), self.longitude()), (Some(lat), Some(lon)) if lat != 0.0 || lon != 0.0)
    }

    /// The `Uri` field. Uri of the metadata endpoint for this image
    pub fn uri(&self) -> Option<&str> {
        self.typed.uri.as_deref()
    }

    /// The `Title` field
    pub fn title(&self) -> Option<&str> {
        self.typed.title.as_deref()
    }

    /// The `Caption` field
    pub fn caption(&self) -> Option<&str> {
        self.typed.caption.as_deref()
    }

    /// The `Keywords` field
    pub fn keywords(&self) -> Option<&str> {
        self.typed.keywords.as_deref()
    }

    /// The `Author` field
    pub fn author(&self) -> Option<&str> {
        self.typed.author.as_deref()
    }

    /// The `Copyright` field
    pub fn copyright(&self) -> Option<&str> {
        self.typed.copyright.as_deref()
    }

    /// The `Make` field. Camera maker
    pub fn make(&self) -> Option<&str> {
        self.typed.make.as_deref()
    }

    /// The `Model` field. Camera model
    pub fn model(&self) -> Option<&str> {
        self.typed.model.as_deref()
    }

    /// The `Lens` field
    pub fn lens(&self) -> Option<&str> {
        self.typed.lens.as_deref()
    }

    /// The `SerialNumber` field. Camera serial number
    pub fn serial_number(&self) -> Option<&str> {
        self.typed.serial_number.as_deref()
    }

    /// The `Software` field. Software or firmware that wrote the file
    pub fn software(&self) -> Option<&str> {
        self.typed.software.as_deref()
    }

    /// The `DateTimeCreated` field. Capture time as written in the file. Usually local time without an offset, e.g. `2026-08-08T08:31:55`
    pub fn date_time_created(&self) -> Option<&str> {
        self.typed.date_time_created.as_deref()
    }

    /// The `DateDigitized` field. Digitized time as written in the file, usually without an offset
    pub fn date_digitized(&self) -> Option<&str> {
        self.typed.date_digitized.as_deref()
    }

    /// The `DateTimeModified` field
    pub fn date_time_modified(&self) -> Option<&str> {
        self.typed.date_time_modified.as_deref()
    }

    /// The `MicroDateTimeCreated` field. Capture time with sub-second precision and, when the file has one, its offset
    pub fn micro_date_time_created(&self) -> Option<&str> {
        self.typed.micro_date_time_created.as_deref()
    }

    /// The `Exposure` field. Exposure time, e.g. `1/167`
    pub fn exposure(&self) -> Option<&str> {
        self.typed.exposure.as_deref()
    }

    /// The `Aperture` field. F-number, e.g. `1.8`
    pub fn aperture(&self) -> Option<&str> {
        self.typed.aperture.as_deref()
    }

    /// The `FocalLength` field. e.g. `5.4 mm`
    pub fn focal_length(&self) -> Option<&str> {
        self.typed.focal_length.as_deref()
    }

    /// The `FocalLength35mm` field
    pub fn focal_length_35mm(&self) -> Option<&str> {
        self.typed.focal_length_35mm.as_deref()
    }

    /// The `Flash` field. e.g. `No Flash`
    pub fn flash(&self) -> Option<&str> {
        self.typed.flash.as_deref()
    }

    /// The `WhiteBalance` field
    pub fn white_balance(&self) -> Option<&str> {
        self.typed.white_balance.as_deref()
    }

    /// The `ExposureMode` field
    pub fn exposure_mode(&self) -> Option<&str> {
        self.typed.exposure_mode.as_deref()
    }

    /// The `ExposureProgram` field
    pub fn exposure_program(&self) -> Option<&str> {
        self.typed.exposure_program.as_deref()
    }

    /// The `Metering` field
    pub fn metering(&self) -> Option<&str> {
        self.typed.metering.as_deref()
    }

    /// The `ColorSpace` field
    pub fn color_space(&self) -> Option<&str> {
        self.typed.color_space.as_deref()
    }

    /// The `Altitude` field. Altitude with its unit, e.g. `273.303 m`
    pub fn altitude(&self) -> Option<&str> {
        self.typed.altitude.as_deref()
    }

    /// The `Duration` field. Video duration with its unit, e.g. `11.40 s`
    pub fn duration(&self) -> Option<&str> {
        self.typed.duration.as_deref()
    }

    /// The `VideoCodec` field
    pub fn video_codec(&self) -> Option<&str> {
        self.typed.video_codec.as_deref()
    }

    /// The `AudioCodec` field
    pub fn audio_codec(&self) -> Option<&str> {
        self.typed.audio_codec.as_deref()
    }

    /// The `City` field
    pub fn city(&self) -> Option<&str> {
        self.typed.city.as_deref()
    }

    /// The `State` field
    pub fn state(&self) -> Option<&str> {
        self.typed.state.as_deref()
    }

    /// The `Country` field
    pub fn country(&self) -> Option<&str> {
        self.typed.country.as_deref()
    }

    /// The `Latitude` field. Decimal degrees. SmugMug reports 0 when the file has no location
    pub fn latitude(&self) -> Option<f64> {
        self.typed.latitude
    }

    /// The `Longitude` field. Decimal degrees. SmugMug reports 0 when the file has no location
    pub fn longitude(&self) -> Option<f64> {
        self.typed.longitude
    }

    /// The `ISO` field
    pub fn iso(&self) -> Option<u64> {
        self.typed.iso
    }
}

impl<'de> Deserialize<'de> for ImageMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let fields = Map::<String, Value>::deserialize(deserializer)?;
        let typed = TypedImageMetadata::deserialize(Value::Object(fields.clone()))
            .map_err(serde::de::Error::custom)?;
        Ok(Self { typed, fields })
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
#[derive(Deserialize, Clone, Debug, Default, PartialEq)]
struct TypedImageMetadata {
    #[serde(default, rename = "Uri", deserialize_with = "from_lenient_string")]
    uri: Option<String>,
    #[serde(default, rename = "Title", deserialize_with = "from_lenient_string")]
    title: Option<String>,
    #[serde(default, rename = "Caption", deserialize_with = "from_lenient_string")]
    caption: Option<String>,
    #[serde(default, rename = "Keywords", deserialize_with = "from_lenient_string")]
    keywords: Option<String>,
    #[serde(default, rename = "Author", deserialize_with = "from_lenient_string")]
    author: Option<String>,
    #[serde(
        default,
        rename = "Copyright",
        deserialize_with = "from_lenient_string"
    )]
    copyright: Option<String>,
    #[serde(default, rename = "Make", deserialize_with = "from_lenient_string")]
    make: Option<String>,
    #[serde(default, rename = "Model", deserialize_with = "from_lenient_string")]
    model: Option<String>,
    #[serde(default, rename = "Lens", deserialize_with = "from_lenient_string")]
    lens: Option<String>,
    #[serde(
        default,
        rename = "SerialNumber",
        deserialize_with = "from_lenient_string"
    )]
    serial_number: Option<String>,
    #[serde(default, rename = "Software", deserialize_with = "from_lenient_string")]
    software: Option<String>,
    #[serde(
        default,
        rename = "DateTimeCreated",
        deserialize_with = "from_lenient_string"
    )]
    date_time_created: Option<String>,
    #[serde(
        default,
        rename = "DateDigitized",
        deserialize_with = "from_lenient_string"
    )]
    date_digitized: Option<String>,
    #[serde(
        default,
        rename = "DateTimeModified",
        deserialize_with = "from_lenient_string"
    )]
    date_time_modified: Option<String>,
    #[serde(
        default,
        rename = "MicroDateTimeCreated",
        deserialize_with = "from_lenient_string"
    )]
    micro_date_time_created: Option<String>,
    #[serde(default, rename = "Exposure", deserialize_with = "from_lenient_string")]
    exposure: Option<String>,
    #[serde(default, rename = "Aperture", deserialize_with = "from_lenient_string")]
    aperture: Option<String>,
    #[serde(
        default,
        rename = "FocalLength",
        deserialize_with = "from_lenient_string"
    )]
    focal_length: Option<String>,
    #[serde(
        default,
        rename = "FocalLength35mm",
        deserialize_with = "from_lenient_string"
    )]
    focal_length_35mm: Option<String>,
    #[serde(default, rename = "Flash", deserialize_with = "from_lenient_string")]
    flash: Option<String>,
    #[serde(
        default,
        rename = "WhiteBalance",
        deserialize_with = "from_lenient_string"
    )]
    white_balance: Option<String>,
    #[serde(
        default,
        rename = "ExposureMode",
        deserialize_with = "from_lenient_string"
    )]
    exposure_mode: Option<String>,
    #[serde(
        default,
        rename = "ExposureProgram",
        deserialize_with = "from_lenient_string"
    )]
    exposure_program: Option<String>,
    #[serde(default, rename = "Metering", deserialize_with = "from_lenient_string")]
    metering: Option<String>,
    #[serde(
        default,
        rename = "ColorSpace",
        deserialize_with = "from_lenient_string"
    )]
    color_space: Option<String>,
    #[serde(default, rename = "Altitude", deserialize_with = "from_lenient_string")]
    altitude: Option<String>,
    #[serde(default, rename = "Duration", deserialize_with = "from_lenient_string")]
    duration: Option<String>,
    #[serde(
        default,
        rename = "VideoCodec",
        deserialize_with = "from_lenient_string"
    )]
    video_codec: Option<String>,
    #[serde(
        default,
        rename = "AudioCodec",
        deserialize_with = "from_lenient_string"
    )]
    audio_codec: Option<String>,
    #[serde(default, rename = "City", deserialize_with = "from_lenient_string")]
    city: Option<String>,
    #[serde(default, rename = "State", deserialize_with = "from_lenient_string")]
    state: Option<String>,
    #[serde(default, rename = "Country", deserialize_with = "from_lenient_string")]
    country: Option<String>,
    #[serde(default, rename = "Latitude", deserialize_with = "from_lenient_f64")]
    latitude: Option<f64>,
    #[serde(default, rename = "Longitude", deserialize_with = "from_lenient_f64")]
    longitude: Option<f64>,
    #[serde(default, rename = "ISO", deserialize_with = "from_lenient_u64")]
    iso: Option<u64>,
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
        assert_eq!(metadata.make(), Some("samsung"));
        assert_eq!(metadata.model(), Some("SM-G991U"));
        assert_eq!(metadata.iso(), Some(40));
        assert_eq!(metadata.aperture(), Some("1.8"));
        assert_eq!(metadata.date_digitized(), Some("2026-08-08T08:31:55"));
        assert_eq!(metadata.altitude(), Some("0"));
        assert_eq!(metadata.caption(), None);
        assert_eq!(metadata.audio_codec(), None);
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
        assert_eq!(metadata.altitude(), Some("273.303 m"));
        assert_eq!(metadata.duration(), Some("11.40 s"));
        assert_eq!(metadata.iso(), None);
        assert_eq!(metadata.latitude(), Some(30.5568));
        assert_eq!(metadata.longitude(), Some(-97.7326));
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
        assert_eq!(metadata.make(), Some("samsung"));
    }
}
