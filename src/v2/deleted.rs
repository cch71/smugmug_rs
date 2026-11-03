/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::parsers::from_lenient_datetime;
use crate::v2::Pages;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An album in the user's trash, from the `!deletedalbums` endpoint.
/// Retrieve these with [`crate::v2::User::deleted_albums`].
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/user.html) for more
/// details.
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct DeletedAlbum {
    /// e.g. `/api/v2/deleted/album/4XDwHf`
    #[serde(default, rename = "Uri")]
    pub uri: String,

    #[serde(default, rename = "Name")]
    pub name: String,

    /// Path the album had on the site before it was deleted, e.g. `/2023-Campout`
    #[serde(default, rename = "UrlPath")]
    pub url_path: String,

    #[serde(
        default,
        rename = "DateAdded",
        deserialize_with = "from_lenient_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_added: Option<DateTime<Utc>>,

    #[serde(
        default,
        rename = "DateDeleted",
        deserialize_with = "from_lenient_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_deleted: Option<DateTime<Utc>>,

    #[serde(
        default,
        rename = "DateModified",
        deserialize_with = "from_lenient_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_modified: Option<DateTime<Utc>>,
}

impl DeletedAlbum {
    /// The album key, taken from the end of [`Self::uri`]
    pub fn album_key(&self) -> &str {
        last_uri_segment(&self.uri)
    }
}

/// A folder in the user's trash, from the `!deletedfolders` endpoint.
/// Retrieve these with [`crate::v2::User::deleted_folders`].
///
/// See [SmugMug API Docs](https://api.smugmug.com/api/v2/doc/reference/user.html) for more
/// details.
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct DeletedFolder {
    /// e.g. `/api/v2/deleted/folder/2bDJGF`
    #[serde(default, rename = "Uri")]
    pub uri: String,

    #[serde(default, rename = "Name")]
    pub name: String,

    /// Path the folder had on the site before it was deleted, e.g. `/2016-Fall`
    #[serde(default, rename = "UrlPath")]
    pub url_path: String,

    #[serde(
        default,
        rename = "DateAdded",
        deserialize_with = "from_lenient_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_added: Option<DateTime<Utc>>,

    #[serde(
        default,
        rename = "DateDeleted",
        deserialize_with = "from_lenient_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_deleted: Option<DateTime<Utc>>,

    #[serde(
        default,
        rename = "DateModified",
        deserialize_with = "from_lenient_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_modified: Option<DateTime<Utc>>,
}

impl DeletedFolder {
    /// The folder's key, taken from the end of [`Self::uri`]
    pub fn folder_key(&self) -> &str {
        last_uri_segment(&self.uri)
    }
}

fn last_uri_segment(uri: &str) -> &str {
    uri.rsplit('/').next().unwrap_or(uri)
}

// Expected response for a request to get a user's deleted albums
#[derive(Deserialize, Debug)]
pub(crate) struct DeletedAlbumsResponse {
    #[serde(default, rename = "DeletedAlbum")]
    pub(crate) albums: Vec<DeletedAlbum>,

    #[serde(rename = "Pages")]
    pub(crate) pages: Option<Pages>,
}

// Expected response for a request to get a user's deleted folders
#[derive(Deserialize, Debug)]
pub(crate) struct DeletedFoldersResponse {
    #[serde(default, rename = "DeletedFolder")]
    pub(crate) folders: Vec<DeletedFolder>,

    #[serde(rename = "Pages")]
    pub(crate) pages: Option<Pages>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Trimmed from live `!deletedalbums` and `!deletedfolders` responses
    #[test]
    fn parses_deleted_albums_and_folders() {
        let albums: DeletedAlbumsResponse = serde_json::from_value(json!({
            "DeletedAlbum": [{
                "DateAdded": "2023-07-22T16:49:16+00:00",
                "DateDeleted": "2024-10-03T22:19:23+00:00",
                "DateModified": "2024-10-03T22:19:23+00:00",
                "Name": "2023 Eagle Scout Project", "Uri": "/api/v2/deleted/album/4XDwHf",
                "Uris": {"RecoverDeletedAlbum": "/api/v2/deleted/album/4XDwHf!recover"},
                "UrlPath": "/2023-Eagle-Scout-Project"
            }],
            "Pages": {"NextPage": "/api/v2/user/nick!deletedalbums?count=1&start=2"}
        }))
        .unwrap();
        let album = &albums.albums[0];
        assert_eq!(album.album_key(), "4XDwHf");
        assert_eq!(album.url_path, "/2023-Eagle-Scout-Project");
        assert_eq!(
            album.date_deleted.unwrap().to_rfc3339(),
            "2024-10-03T22:19:23+00:00"
        );
        assert!(albums.pages.unwrap().next_page.is_some());

        let folders: DeletedFoldersResponse = serde_json::from_value(json!({
            "DeletedFolder": [{
                "DateDeleted": "2016-09-04T01:37:11+00:00", "Name": "2016 Fall COH",
                "Uri": "/api/v2/deleted/folder/2bDJGF", "UrlPath": "/2016-Fall-COH"
            }]
        }))
        .unwrap();
        assert_eq!(folders.folders[0].folder_key(), "2bDJGF");
        assert_eq!(folders.folders[0].date_modified, None);
        assert!(folders.pages.is_none());
    }
}
