use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub kind: EntryKind,
    pub size: u64,
    pub modified_ms: Option<u64>,
    pub is_hidden: bool,
    pub is_symlink: bool,
    pub symlink_target_kind: Option<EntryKind>,
    pub is_broken_symlink: bool,
    pub readable: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryInfo {
    pub name: String,
    pub path: String,
    pub kind: EntryKind,
    pub size: u64,
    pub size_human: String,
    pub modified_ms: Option<u64>,
    pub accessed_ms: Option<u64>,
    pub created_ms: Option<u64>,
    pub mode: u32,
    pub mode_string: String,
    pub owner: String,
    pub group: String,
    pub uid: u32,
    pub gid: u32,
    pub nlink: u64,
    pub inode: u64,
    pub is_hidden: bool,
    pub is_symlink: bool,
    pub symlink_target: Option<String>,
    pub symlink_resolved: Option<String>,
    pub is_broken_symlink: bool,
    pub mime_type: Option<String>,
    pub child_count: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortKey {
    Name,
    Size,
    Modified,
    Kind,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOptions {
    pub sort: SortKey,
    pub descending: bool,
    pub show_hidden: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirSize {
    pub path: String,
    pub bytes: u64,
    pub size_human: String,
    pub file_count: u64,
    pub dir_count: u64,
    pub complete: bool,
    pub error_count: u64,
    pub measured_at_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextHead {
    pub content: String,
    pub truncated: bool,
    pub is_binary: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub label: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpOutcome {
    pub succeeded: Vec<String>,
    pub failed: Vec<OpFailure>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpFailure {
    pub path: String,
    pub error: AppError,
}
