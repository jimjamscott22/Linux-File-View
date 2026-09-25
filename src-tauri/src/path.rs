use crate::error::{AppError, ErrorKind};
use std::path::{Path, PathBuf};

pub fn canonicalize_existing(input: &str) -> Result<PathBuf, AppError> {
    if input.is_empty() || input.contains('\0') || !Path::new(input).is_absolute() {
        return Err(AppError::new(
            ErrorKind::InvalidPath,
            format!("Invalid path: {input:?}"),
        ));
    }
    std::fs::canonicalize(input).map_err(|error| {
        let mut app_error = AppError::from(error);
        app_error.message = format!("{}: {input}", app_error.message);
        app_error
    })
}

pub fn canonicalize_new_target(parent: &str, name: &str) -> Result<PathBuf, AppError> {
    validate_name(name)?;
    let parent = canonicalize_existing(parent)?;
    let target = parent.join(name);
    reject_mutation_target(&target)?;
    Ok(target)
}

pub fn validate_name(name: &str) -> Result<(), AppError> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\0')
        || name.len() > 255
    {
        Err(AppError::new(
            ErrorKind::InvalidPath,
            format!("Invalid name: {name:?}"),
        ))
    } else {
        Ok(())
    }
}

pub fn reject_mutation_target(path: &Path) -> Result<(), AppError> {
    if ["/", "/proc", "/sys", "/dev"].iter().any(|protected| {
        path == Path::new(protected) || (protected != &"/" && path.starts_with(protected))
    }) {
        return Err(AppError::new(
            ErrorKind::InvalidPath,
            format!("Protected path: {}", path.display()),
        ));
    }
    Ok(())
}

pub fn reject_recursive_destination(source: &Path, destination: &Path) -> Result<(), AppError> {
    if destination.starts_with(source) {
        Err(AppError::new(
            ErrorKind::WouldRecurse,
            format!(
                "Destination {} is inside source {}",
                destination.display(),
                source.display()
            ),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;

    #[test]
    fn rejects_invalid_path_inputs() {
        for path in ["", "relative", "../relative", "/tmp/has\0nul"] {
            let error = canonicalize_existing(path).expect_err(path);
            assert!(matches!(error.kind, ErrorKind::InvalidPath), "{path:?}");
        }
    }

    #[test]
    fn canonicalizes_existing_directory_and_reports_missing_path() {
        assert_eq!(
            canonicalize_existing("/tmp/..").unwrap(),
            std::path::PathBuf::from("/")
        );
        let error = canonicalize_existing("/definitely-not-a-tfm-directory-928343").unwrap_err();
        assert!(matches!(error.kind, ErrorKind::NotFound));
    }
    #[test]
    fn rejects_invalid_names_and_protected_mutation_targets() {
        for name in ["", ".", "..", "a/b", &"a".repeat(256)] {
            assert!(matches!(
                validate_name(name).unwrap_err().kind,
                ErrorKind::InvalidPath
            ));
        }
        for path in ["/", "/proc", "/proc/self", "/sys", "/dev/null"] {
            assert!(matches!(
                reject_mutation_target(std::path::Path::new(path))
                    .unwrap_err()
                    .kind,
                ErrorKind::InvalidPath
            ));
        }
    }

    #[test]
    fn canonicalizes_new_target_parent_before_joining_name() {
        let target = canonicalize_new_target("/tmp/..", "tfm-new-file").unwrap();
        assert_eq!(target, PathBuf::from("/tfm-new-file"));
        assert!(matches!(
            canonicalize_new_target("/tmp", "../escape")
                .unwrap_err()
                .kind,
            ErrorKind::InvalidPath
        ));
        assert!(matches!(
            canonicalize_new_target("/proc", "new-file")
                .unwrap_err()
                .kind,
            ErrorKind::InvalidPath
        ));
    }

    #[test]
    fn refuses_to_copy_directory_into_its_own_subtree() {
        assert!(matches!(
            reject_recursive_destination(
                std::path::Path::new("/tmp/source"),
                std::path::Path::new("/tmp/source/child")
            )
            .unwrap_err()
            .kind,
            ErrorKind::WouldRecurse
        ));
    }
}
