use crate::error::AppError;
use crate::model::{DirEntry, EntryKind, ListOptions, SortKey};
use crate::util::system_time_to_ms;
use std::cmp::Ordering;
use std::fs;
use std::path::Path;

pub fn list_dir(path: &Path, options: &ListOptions) -> Result<Vec<DirEntry>, AppError> {
    let mut entries = Vec::new();
    for item in fs::read_dir(path).map_err(AppError::from)? {
        let item = item.map_err(AppError::from)?;
        let name = item.file_name().to_string_lossy().into_owned();
        if !options.show_hidden && name.starts_with('.') {
            continue;
        }
        let entry_path = item.path();
        let metadata = match fs::symlink_metadata(&entry_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(AppError::from(error)),
        };
        let kind = kind_of(&metadata);
        let (target_kind, is_broken) = if metadata.file_type().is_symlink() {
            match fs::metadata(&entry_path) {
                Ok(target) => (Some(kind_of(&target)), false),
                Err(_) => (None, true),
            }
        } else {
            (None, false)
        };
        let readable = match target_kind.as_ref().unwrap_or(&kind) {
            EntryKind::Directory => fs::read_dir(&entry_path).is_ok(),
            EntryKind::File => fs::File::open(&entry_path).is_ok(),
            _ => false,
        };
        entries.push(DirEntry {
            name: name.clone(),
            path: entry_path.to_string_lossy().into_owned(),
            kind,
            size: metadata.len(),
            modified_ms: metadata.modified().ok().and_then(system_time_to_ms),
            is_hidden: name.starts_with('.'),
            is_symlink: metadata.file_type().is_symlink(),
            symlink_target_kind: target_kind,
            is_broken_symlink: is_broken,
            readable,
        });
    }
    entries.sort_by(|a, b| {
        let group = group_of(a).cmp(&group_of(b));
        if group != Ordering::Equal {
            return group;
        }
        let key = compare_key(a, b, &options.sort);
        let key = if options.descending {
            key.reverse()
        } else {
            key
        };
        if key == Ordering::Equal {
            compare_name(a, b)
        } else {
            key
        }
    });
    Ok(entries)
}

fn kind_of(metadata: &fs::Metadata) -> EntryKind {
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        EntryKind::Symlink
    } else if file_type.is_dir() {
        EntryKind::Directory
    } else if file_type.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    }
}

fn group_of(entry: &DirEntry) -> u8 {
    match entry.symlink_target_kind.as_ref().unwrap_or(&entry.kind) {
        EntryKind::Directory => 0,
        EntryKind::Other => 2,
        _ => 1,
    }
}

fn compare_name(a: &DirEntry, b: &DirEntry) -> Ordering {
    a.name
        .to_lowercase()
        .cmp(&b.name.to_lowercase())
        .then_with(|| a.name.cmp(&b.name))
}

fn compare_key(a: &DirEntry, b: &DirEntry, key: &SortKey) -> Ordering {
    match key {
        SortKey::Name => compare_name(a, b),
        SortKey::Size => a.size.cmp(&b.size),
        SortKey::Modified => a.modified_ms.cmp(&b.modified_ms),
        SortKey::Kind => group_of(a).cmp(&group_of(b)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ListOptions, SortKey};
    use std::os::unix::fs::symlink;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn permission_denied_directory_is_an_error_not_an_empty_list() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!(
            "tfm-unreadable-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o000)).unwrap();
        let options = ListOptions {
            sort: SortKey::Name,
            descending: false,
            show_hidden: false,
        };
        let result = list_dir(&root, &options);
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::remove_dir(&root).unwrap();
        assert!(matches!(
            result.unwrap_err().kind,
            crate::error::ErrorKind::PermissionDenied
        ));
    }

    #[test]
    fn lists_directory_first_and_hides_dotfiles_by_default() {
        let root = std::env::temp_dir().join(format!(
            "tfm-list-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(root.join("z-dir")).unwrap();
        std::fs::write(root.join("b-file"), b"longer").unwrap();
        std::fs::write(root.join("a-file"), b"x").unwrap();
        std::fs::write(root.join(".hidden"), b"").unwrap();
        symlink("z-dir", root.join("link-dir")).unwrap();
        symlink("missing", root.join("broken")).unwrap();

        let options = ListOptions {
            sort: SortKey::Name,
            descending: false,
            show_hidden: false,
        };
        let entries = list_dir(&root, &options).unwrap();
        let names: Vec<_> = entries.iter().map(|entry| entry.name.as_str()).collect();
        assert_eq!(names, ["link-dir", "z-dir", "a-file", "b-file", "broken"]);
        assert!(entries[0].is_symlink);
        assert!(entries[4].is_broken_symlink);
        assert_eq!(entries[0].path, root.join("link-dir").to_string_lossy());

        let options = ListOptions {
            sort: SortKey::Size,
            descending: true,
            show_hidden: true,
        };
        let sorted = list_dir(&root, &options).unwrap();
        let names: Vec<_> = sorted.iter().map(|entry| entry.name.as_str()).collect();
        assert_eq!(&names[..2], &["z-dir", "link-dir"]);
        assert_eq!(&names[2..], &["broken", "b-file", "a-file", ".hidden"]);
        std::fs::remove_dir_all(root).unwrap();
    }
}
