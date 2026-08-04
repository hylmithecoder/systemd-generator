use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryLocationInfo {
    pub source_path: PathBuf,
    pub app_name: String,
    pub binary_name: String,
    pub opt_dir: PathBuf,
    pub opt_binary_path: PathBuf,
    pub exists: bool,
}

/// Clean raw input path by removing surrounding single/double quotes and whitespace
pub fn clean_path_input(raw_path: &str) -> &str {
    let mut s = raw_path.trim();
    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        if s.len() >= 2 {
            s = &s[1..s.len() - 1];
        }
    }
    s.trim()
}

/// Expand tilde `~` to home directory if present
pub fn expand_path(raw_path: &str) -> PathBuf {
    let cleaned = clean_path_input(raw_path);
    if cleaned.starts_with("~/") || cleaned == "~" {
        if let Some(home) = std::env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            if cleaned.len() > 2 {
                path.push(&cleaned[2..]);
            }
            return path;
        }
    }
    PathBuf::from(cleaned)
}

/// Checks if a folder name is a common compiler / build output directory
fn is_build_dir(name: &str) -> bool {
    let lowercase = name.to_lowercase();
    matches!(
        lowercase.as_str(),
        "" | "/" | "bin" | "target" | "release" | "debug" | "deps" | "build" | "out" | "dist"
    ) || lowercase.contains("unknown-linux")
        || lowercase.contains("x86_64")
        || lowercase.contains("aarch64")
        || lowercase.contains("gnu")
        || lowercase.contains("musl")
}

/// Derive `/opt/<app_name>/<binary_file>` consistent paths
pub fn resolve_binary_info(raw_path: &str) -> Option<BinaryLocationInfo> {
    let cleaned = clean_path_input(raw_path);
    if cleaned.is_empty() {
        return None;
    }

    let expanded = expand_path(cleaned);

    let binary_name = expanded
        .file_name()
        .map(|s| s.to_string_lossy().to_string())?;

    // Determine app_name by walking up parent directories until a non-build directory is found
    let mut current_parent = expanded.parent();
    let mut app_name = binary_name.clone();

    while let Some(parent) = current_parent {
        let parent_name = parent
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        if !parent_name.is_empty() && !is_build_dir(&parent_name) {
            app_name = parent_name;
            break;
        }
        current_parent = parent.parent();
    }

    let opt_dir = PathBuf::from(format!("/opt/{}", app_name));
    let opt_binary_path = opt_dir.join(&binary_name);
    let exists = expanded.exists();

    Some(BinaryLocationInfo {
        source_path: expanded,
        app_name,
        binary_name,
        opt_dir,
        opt_binary_path,
        exists,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_binary_info_target_release() {
        let path = "'/home/hylmi/Hylmi/Pemrograman_Berorientasi_Objek/Rust/api_siabsen/target/x86_64-unknown-linux-musl/release/api_siabsen'";
        let info = resolve_binary_info(path).unwrap();
        assert_eq!(info.app_name, "api_siabsen");
        assert_eq!(info.binary_name, "api_siabsen");
        assert_eq!(info.opt_dir, PathBuf::from("/opt/api_siabsen"));
        assert_eq!(
            info.opt_binary_path,
            PathBuf::from("/opt/api_siabsen/api_siabsen")
        );
    }

    #[test]
    fn test_resolve_binary_info_single_file() {
        let info = resolve_binary_info("/my-bin").unwrap();
        assert_eq!(info.app_name, "my-bin");
        assert_eq!(info.binary_name, "my-bin");
        assert_eq!(info.opt_binary_path, PathBuf::from("/opt/my-bin/my-bin"));
    }
}
