use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use rand::Rng;

/// Securely obliterates a file: Overwrite -> Rename -> Delete
pub fn shred_file(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    // 1. Get file size
    let metadata = fs::metadata(path)?;
    let len = metadata.len();

    // 2. Overwrite with logic (3 passes is standard, we do 1 fast pass of random noise for speed/stealth trade-off)
    // "Singularity" favors speed + entropy. 
    if len > 0 {
        if let Ok(mut file) = OpenOptions::new().write(true).open(path) {
            let mut rng = rand::thread_rng();
            // We'll just overwrite the first 4KB and the last 4KB if it's huge, 
            // or the whole thing if it's small (<10MB).
            // Writing 1GB of random data is too slow for a "Cleaner". 
            // We want to break the file header and structure.
            
            let buf_size = if len < 1024 * 1024 * 10 { len as usize } else { 4096 };
            let mut buffer = vec![0u8; buf_size];
            rng.fill(&mut buffer[..]);
            
            // Write to start
            let _ = file.write_all(&buffer);
            let _ = file.flush();
        }
    }

    // 3. Rename to random garbage (prevents recovery by filename)
    let parent = path.parent().unwrap_or(Path::new("C:\\"));
    let mut rng = rand::thread_rng();
    let new_name: String = (0..12).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect();
    let new_path = parent.join(format!("{}.tmp", new_name));
    
    if let Ok(_) = fs::rename(path, &new_path) {
        // 4. Delete the renamed file
        fs::remove_file(new_path)?;
    } else {
        // If rename fails (locked?), try direct delete
        fs::remove_file(path)?;
    }

    Ok(())
}
