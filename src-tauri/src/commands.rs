use crate::error::{AppError, ErrorKind};
use crate::model::{DirEntry, ListOptions};
use crate::{path, read};
use std::path::Path;

#[tauri::command]
pub fn resolve_dir(path: String) -> Result<String, AppError> {
    let canonical = path::canonicalize_existing(&path)?;
    if !canonical.is_dir() {
        return Err(AppError::new(
            ErrorKind::NotADirectory,
            format!("Not a directory: {}", canonical.display()),
        ));
    }
    Ok(canonical.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn list_dir(path: String, options: ListOptions) -> Result<Vec<DirEntry>, AppError> {
    let canonical = path::canonicalize_existing(&path)?;
    read::list_dir(&canonical, &options).map_err(|mut error| {
        error.message = format!("{}: {}", error.message, canonical.display());
        error
    })
}

#[tauri::command]
pub fn home_dir() -> Result<String, AppError> {
    let home = std::env::var("HOME")
        .map_err(|error| AppError::new(ErrorKind::Io, format!("HOME is unavailable: {error}")))?;
    resolve_dir(home)
}

#[tauri::command]
pub fn parent_dir(path: String) -> Result<Option<String>, AppError> {
    let canonical = path::canonicalize_existing(&path)?;
    Ok(canonical
        .parent()
        .map(Path::to_string_lossy)
        .map(|parent| parent.into_owned()))
}
