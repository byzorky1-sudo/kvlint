use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const RELEVANT_EXTENSIONS: &[&str] = &[
    "py", "ts", "js", "rs", "go", "json", "yaml", "yml", "toml", "md", "txt", "prompt",
];

pub fn find_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if RELEVANT_EXTENSIONS.contains(&ext) {
                    // Skip git, node_modules, target, venv
                    let path_str = path.to_string_lossy();
                    if !path_str.contains("/.git/")
                        && !path_str.contains("/node_modules/")
                        && !path_str.contains("/target/")
                        && !path_str.contains("/.venv/")
                    {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }
    }
    files
}
