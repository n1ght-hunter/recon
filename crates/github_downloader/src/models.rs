//! This module contains the models for the github_downloader crate.

use serde::{Deserialize, Serialize};

pub type Objects = Vec<Object>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: i64,
    pub url: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    #[serde(rename = "git_url")]
    pub git_url: String,
    #[serde(rename = "download_url")]
    pub download_url: Option<String>,
    #[serde(rename = "type")]
    pub type_field: FieldType,
    #[serde(rename = "_links")]
    pub links: Links,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    #[serde(rename = "self")]
    pub self_field: String,
    pub git: String,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "dir")]
    Dir,
}
