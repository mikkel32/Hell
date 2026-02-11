use std::fs;
use std::path::{Path, PathBuf};

/// Recursively finds cache and log directories in User Profile
pub fn find_smart_targets() -> Vec<PathBuf> {
    let mut targets = Vec::new();
    
    let vars = ["LOCALAPPDATA", "APPDATA"];
    
    for var in vars {
        if let Ok(path_str) = std::env::var(var) {
            let root = PathBuf::from(path_str);
            // We scan depth 2: AppData/Local/Vendor/App/Cache
            scan_dir(&root, 0, 3, &mut targets);
        }
    }
    
    targets
}

fn scan_dir(dir: &Path, depth: usize, max_depth: usize, results: &mut Vec<PathBuf>) {
    if depth >= max_depth {
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                
                // HEURISTIC TRIGGERS
                if name == "cache" || 
                   name == "gpucache" || 
                   name == "logs" || 
                   name == "temp" || 
                   name == "blob_storage" || 
                   name == "code cache" ||
                   name == "sessions" {
                    
                    results.push(path.clone());
                    // Don't recurse into a target we already pledged to destroy
                    continue; 
                }

                scan_dir(&path, depth + 1, max_depth, results);
            }
        }
    }
}
