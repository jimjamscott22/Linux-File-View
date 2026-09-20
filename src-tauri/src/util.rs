use std::time::SystemTime;

pub fn mode_string(mode: u32, is_dir: bool, is_symlink: bool) -> String {
    // We only have the raw st_mode.
    // The file type is technically in the upper bits of st_mode (S_IFMT),
    // but in Rust's fs::Metadata, we often just check is_dir() etc.
    // Let's deduce type char from the S_IFMT bits if possible,
    // falling back to is_dir/is_symlink.

    let s_ifmt = mode & 0o170000;
    let type_char = match s_ifmt {
        0o140000 => 's', // socket
        0o120000 => 'l', // symlink
        0o100000 => '-', // regular file
        0o060000 => 'b', // block device
        0o040000 => 'd', // directory
        0o020000 => 'c', // char device
        0o010000 => 'p', // fifo
        _ => {
            if is_symlink {
                'l'
            } else if is_dir {
                'd'
            } else {
                '-'
            }
        }
    };

    let mut result = String::with_capacity(10);
    result.push(type_char);

    // User
    result.push(if (mode & 0o400) != 0 { 'r' } else { '-' });
    result.push(if (mode & 0o200) != 0 { 'w' } else { '-' });
    let setuid = (mode & 0o4000) != 0;
    let u_x = (mode & 0o100) != 0;
    result.push(match (setuid, u_x) {
        (true, true) => 's',
        (true, false) => 'S',
        (false, true) => 'x',
        (false, false) => '-',
    });

    // Group
    result.push(if (mode & 0o040) != 0 { 'r' } else { '-' });
    result.push(if (mode & 0o020) != 0 { 'w' } else { '-' });
    let setgid = (mode & 0o2000) != 0;
    let g_x = (mode & 0o010) != 0;
    result.push(match (setgid, g_x) {
        (true, true) => 's',
        (true, false) => 'S',
        (false, true) => 'x',
        (false, false) => '-',
    });

    // Other
    result.push(if (mode & 0o004) != 0 { 'r' } else { '-' });
    result.push(if (mode & 0o002) != 0 { 'w' } else { '-' });
    let sticky = (mode & 0o1000) != 0;
    let o_x = (mode & 0o001) != 0;
    result.push(match (sticky, o_x) {
        (true, true) => 't',
        (true, false) => 'T',
        (false, true) => 'x',
        (false, false) => '-',
    });

    result
}

pub fn human_size(bytes: u64) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < units.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, units[0])
    } else {
        format!("{:.1} {}", size, units[unit_index])
    }
}

pub fn system_time_to_ms(time: SystemTime) -> Option<u64> {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_string() {
        assert_eq!(mode_string(0o040755, true, false), "drwxr-xr-x");
        assert_eq!(mode_string(0o100644, false, false), "-rw-r--r--");
        assert_eq!(mode_string(0o120777, false, true), "lrwxrwxrwx");

        // setuid, setgid, sticky
        assert_eq!(mode_string(0o104755, false, false), "-rwsr-xr-x");
        assert_eq!(mode_string(0o102755, false, false), "-rwxr-sr-x");
        assert_eq!(mode_string(0o041755, true, false), "drwxr-xr-t");

        assert_eq!(mode_string(0o106644, false, false), "-rwSr-Sr--"); // uppercase when x bit is clear
        assert_eq!(mode_string(0o041644, true, false), "drw-r--r-T"); // uppercase when x bit is clear
    }

    #[test]
    fn test_human_size() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(999), "999 B");
        assert_eq!(human_size(1024), "1.0 KiB");
        assert_eq!(human_size(1024 + 512), "1.5 KiB");
        assert_eq!(human_size(1048576), "1.0 MiB");
        assert_eq!(human_size(1048576 * 1024), "1.0 GiB");
    }
}
