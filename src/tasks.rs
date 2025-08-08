use std::path::{Path};
use std::path::PathBuf;
use rfd::FileDialog;
use async_stream::stream;
use std::fs;
use crate::ui::Message;
use crate::file_ops::list_files_in_directory;
use crate::file_ops::rename_files_with_leading_zeros;

pub async fn folder_selection(default_dir: PathBuf) -> String {
    FileDialog::new()
        .set_directory(default_dir)
        .pick_folder()
        .map_or("".to_string(), |p| p.to_string_lossy().to_string())
}

// Streamed renaming with progress
pub fn perform_renaming_with_progress(
    input: Option<String>,
    output: Option<String>,
    ext: String,
    padding_zeros: usize,
    include_original_name: bool,
) -> impl futures::Stream<Item = Message> {
    let input_path = input.unwrap_or_default();
    let output_path = output.unwrap_or_default();
    let ext_clean = ext.trim_start_matches('.').to_string();

    stream! {
        let files = match list_files_in_directory(&input_path, &ext_clean) {
            Ok(f) => f,
            Err(e) => {
                yield Message::RenamingDone(Err(e));
                return;
            }
        };

        let total_files = files.len();
        if total_files == 0 {
            yield Message::RenamingDone(Err("No files found to rename.".to_string()));
            return;
        }

        let output_dir = Path::new(&output_path);
        if let Err(e) = fs::create_dir_all(output_dir) {
            yield Message::RenamingDone(Err(e.to_string()));
            return;
        }

        let new_names = rename_files_with_leading_zeros(&files, padding_zeros, include_original_name);
        let mut result_names = Vec::new();

        for (i, (old_path, new_name)) in files.iter().zip(new_names.iter()).enumerate() {
            let new_path = output_dir.join(new_name);
            if let Err(e) = fs::copy(old_path, &new_path) {
                yield Message::RenamingDone(Err(e.to_string()));
                return;
            }

            result_names.push(new_path.to_string_lossy().to_string());

            yield Message::RenamingProgress(i + 1, total_files);
        }

        yield Message::RenamingDone(Ok(result_names));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File};
    use std::io::Write;
    use tempfile::tempdir;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_perform_renaming_with_progress_success() {
        let input_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        // Create some .mp3 files in the input directory
        for i in 1..=3 {
            let file_path = input_dir.path().join(format!("track{}.mp3", i));
            let mut file = File::create(file_path).unwrap();
            writeln!(file, "Dummy content").unwrap();
        }

        // Run the stream
        let mut stream = Box::pin(perform_renaming_with_progress(
            Some(input_dir.path().to_string_lossy().to_string()),
            Some(output_dir.path().to_string_lossy().to_string()),
            "mp3".into(),
            3,
            true,
        ));

        let mut progress_updates = Vec::new();
        let mut final_result = None;

        while let Some(msg) = stream.next().await {
            match msg {
                Message::RenamingProgress(done, total) => {
                    progress_updates.push((done, total));
                }
                Message::RenamingDone(result) => {
                    final_result = Some(result);
                }
                _ => {}
            }
        }

        assert_eq!(progress_updates.len(), 3);
        assert!(matches!(final_result, Some(Ok(_))));
        let Ok(renamed_files) = final_result.unwrap() else { panic!("Expected Ok result") };
        assert_eq!(renamed_files.len(), 3);

        // Check that the files exist in output
        for file in renamed_files {
            assert!(Path::new(&file).exists());
        }
    }

    #[tokio::test]
    async fn test_perform_renaming_with_progress_empty_input() {
        let input_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        // No files created in input_dir

        let mut stream = Box::pin(perform_renaming_with_progress(
            Some(input_dir.path().to_string_lossy().to_string()),
            Some(output_dir.path().to_string_lossy().to_string()),
            "mp3".into(),
            3,
            true,
        ));

        let mut final_result = None;

        while let Some(msg) = stream.next().await {
            if let Message::RenamingDone(result) = msg {
                final_result = Some(result);
            }
        }

        let Err(err) = final_result.unwrap() else { panic!("Expected error for empty input") };
        assert_eq!(err, "No files found to rename.");
    }
}
