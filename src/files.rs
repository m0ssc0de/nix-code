use std::path::{Path, PathBuf};

pub fn find_nix_file(dir: &Path) -> Option<PathBuf> {
    let nix_files = dir
        .read_dir()
        .ok()?
        .filter_map(|entry| {
            entry.ok().and_then(|entry| {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "nix") {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    // Sort nix_files so that shell.nix comes first, then default.nix, then others
    let mut nix_files = nix_files;
    nix_files.sort_by(|a, b| {
        let a_str = a.file_name().unwrap().to_str().unwrap();
        let b_str = b.file_name().unwrap().to_str().unwrap();

        if a_str == "shell.nix" {
            std::cmp::Ordering::Less
        } else if b_str == "shell.nix" {
            std::cmp::Ordering::Greater
        } else if a_str == "default.nix" {
            std::cmp::Ordering::Less
        } else if b_str == "default.nix" {
            std::cmp::Ordering::Greater
        } else {
            a_str.cmp(b_str)
        }
    });

    // Choose the first nix file, or return an error if there are none
    nix_files.first().cloned()
}
