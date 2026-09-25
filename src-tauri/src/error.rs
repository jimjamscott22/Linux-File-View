use serde::Serialize;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Not found")]
    NotFound,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Already exists")]
    AlreadyExists,
    #[error("Not a directory")]
    NotADirectory,
    #[error("Is a directory")]
    IsADirectory,
    #[error("Cross-device link")]
    CrossDevice,
    #[error("Invalid path")]
    InvalidPath,
    #[error("Directory not empty")]
    DirectoryNotEmpty,
    #[error("Would recurse")]
    WouldRecurse,
    #[error("Cancelled")]
    Cancelled,
    #[error("Unsupported")]
    Unsupported,
    #[error("IO error")]
    Io,
}

#[derive(Debug, thiserror::Error)]
#[error("{kind}: {message}")]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 2)?;

        let kind_str = match self.kind {
            ErrorKind::NotFound => "NotFound",
            ErrorKind::PermissionDenied => "PermissionDenied",
            ErrorKind::AlreadyExists => "AlreadyExists",
            ErrorKind::NotADirectory => "NotADirectory",
            ErrorKind::IsADirectory => "IsADirectory",
            ErrorKind::CrossDevice => "CrossDevice",
            ErrorKind::InvalidPath => "InvalidPath",
            ErrorKind::DirectoryNotEmpty => "DirectoryNotEmpty",
            ErrorKind::WouldRecurse => "WouldRecurse",
            ErrorKind::Cancelled => "Cancelled",
            ErrorKind::Unsupported => "Unsupported",
            ErrorKind::Io => "Io",
        };

        state.serialize_field("kind", kind_str)?;
        state.serialize_field("message", &self.message)?;
        state.end()
    }
}

impl AppError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        let message = error.to_string();
        let kind = match error.kind() {
            io::ErrorKind::NotFound => ErrorKind::NotFound,
            io::ErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists => ErrorKind::AlreadyExists,
            io::ErrorKind::NotADirectory => ErrorKind::NotADirectory,
            io::ErrorKind::IsADirectory => ErrorKind::IsADirectory,
            io::ErrorKind::DirectoryNotEmpty => ErrorKind::DirectoryNotEmpty,
            _ => {
                if let Some(raw) = error.raw_os_error() {
                    match raw {
                        18 => ErrorKind::CrossDevice,       // EXDEV
                        20 => ErrorKind::NotADirectory,     // ENOTDIR
                        21 => ErrorKind::IsADirectory,      // EISDIR
                        39 => ErrorKind::DirectoryNotEmpty, // ENOTEMPTY
                        _ => ErrorKind::Io,
                    }
                } else {
                    ErrorKind::Io
                }
            }
        };

        Self { kind, message }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::discriminant;

    #[test]
    fn maps_io_errors_to_contract_kinds() {
        let cases = [
            (
                io::Error::from(io::ErrorKind::NotFound),
                ErrorKind::NotFound,
            ),
            (
                io::Error::from(io::ErrorKind::PermissionDenied),
                ErrorKind::PermissionDenied,
            ),
            (
                io::Error::from(io::ErrorKind::AlreadyExists),
                ErrorKind::AlreadyExists,
            ),
            (
                io::Error::from(io::ErrorKind::NotADirectory),
                ErrorKind::NotADirectory,
            ),
            (
                io::Error::from(io::ErrorKind::IsADirectory),
                ErrorKind::IsADirectory,
            ),
            (
                io::Error::from(io::ErrorKind::DirectoryNotEmpty),
                ErrorKind::DirectoryNotEmpty,
            ),
            (io::Error::from_raw_os_error(18), ErrorKind::CrossDevice),
            (io::Error::from_raw_os_error(20), ErrorKind::NotADirectory),
            (io::Error::from_raw_os_error(21), ErrorKind::IsADirectory),
            (
                io::Error::from_raw_os_error(39),
                ErrorKind::DirectoryNotEmpty,
            ),
            (io::Error::from_raw_os_error(12345), ErrorKind::Io),
        ];

        for (source, expected) in cases {
            let error = AppError::from(source);
            assert_eq!(discriminant(&error.kind), discriminant(&expected));
        }
    }

    #[test]
    fn serializes_error_as_kind_and_message() {
        let error = AppError::new(ErrorKind::InvalidPath, "Invalid path: /tmp/missing");
        let value = serde_json::to_value(error).expect("serialize AppError");
        assert_eq!(
            value,
            serde_json::json!({
                "kind": "InvalidPath",
                "message": "Invalid path: /tmp/missing"
            })
        );
    }
}
