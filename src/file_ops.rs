use std::path::{PathBuf};
use natord::compare;
use walkdir::WalkDir;

// --- File listing and renaming logic ---
pub fn list_files_in_directory(path: &str, ext: &str) -> Result<Vec<PathBuf>, String> {
    let ext_lower = ext.to_lowercase();
    let mut entries: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.path().to_path_buf();
            let matches = path.extension()
                .map(|e| e.to_string_lossy().to_lowercase() == ext_lower)
                .unwrap_or(false);
            if matches {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    entries.sort_by(|a, b| compare(a.to_string_lossy().as_ref(), b.to_string_lossy().as_ref()));
    Ok(entries)
}

pub fn rename_files_with_leading_zeros(files: &[PathBuf], padding_zeros: usize, include_original_name: bool) -> Vec<String> {
    files.iter()
        .enumerate()
        .map(|(i, path)| {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            let index_str = format!("{:0width$}", i + 1, width=padding_zeros);
            let ext = path.extension()
                      .map(|e| format!(".{}", e.to_string_lossy()))
                      .unwrap_or_default();
            if include_original_name {
                format!("{}_{}", index_str, file_name)
            } else {
                format!("{}{}", index_str, ext)
            }
        })
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File};
    use tempfile::tempdir;

    #[test]
    fn test_list_files_in_directory_filters_extension() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Create dummy files
        File::create(path.join("a.mp3")).unwrap();
        File::create(path.join("b.mp3")).unwrap();
        File::create(path.join("c.txt")).unwrap();

        let result = list_files_in_directory(path.to_str().unwrap(), "mp3").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_rename_files_with_leading_zeros_includes_original() {
        let files = vec![
            PathBuf::from("song1.mp3"),
            PathBuf::from("song2.mp3"),
        ];

        let result = rename_files_with_leading_zeros(&files, 3, true);
        assert_eq!(result[0], "001_song1.mp3");
        assert_eq!(result[1], "002_song2.mp3");
    }

    #[test]
    fn test_rename_files_with_leading_zeros_without_original() {
        let files = vec![
            PathBuf::from("song1.mp3"),
            PathBuf::from("song2.mp3"),
        ];

        let result = rename_files_with_leading_zeros(&files, 2, false);
        assert_eq!(result[0], "01.mp3");
        assert_eq!(result[1], "02.mp3");
    }
}
