/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use serde::Serialize;
use strum_macros::{EnumString, IntoStaticStr};

#[derive(Debug, EnumString, IntoStaticStr)]
pub enum SortMethod {
    Organizer,
    SortIndex,
    Name,
    DateAdded,
    DateModified,
}

#[derive(Debug, EnumString, IntoStaticStr)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Serialize, EnumString, IntoStaticStr)]
pub enum PrivacyLevel {
    Unknown,
    Public,
    Unlisted,
    Private,
}

#[derive(Debug, EnumString, IntoStaticStr)]
pub enum NodeTypeFilters {
    Any,
    Album,
    Folder,
    Page,
    #[strum(to_string = "System Album")]
    SystemAlbum,
    #[strum(to_string = "Folder Album Page")]
    FolderAlbumPage,
}

#[derive(Debug, Serialize, EnumString, IntoStaticStr)]
pub enum NodeType {
    Unknown,
    Album,
    Folder,
    Page,
    #[strum(to_string = "System Folder")]
    SystemFolder,
    #[strum(to_string = "System Page")]
    SystemPage,
}
