use iced::widget::{button, row, column, text, container, text_input};
use iced::{Fill, Element, Task};
use rfd::FileDialog;
use dirs_next::home_dir;
use std::path::{Path, PathBuf};
use std::fs;
use natord::compare;

pub fn main() -> iced::Result {
    iced::application("Renamer", update, view).run()
}

struct State {
    folder_selector_opened: bool,
    input_folder_path: Option<String>,
    output_folder_path: Option<String>,
    default_home_dir: PathBuf,
    // Add a place to show result messages (like file list or errors)
    status_message: String,
}

impl Default for State {
    fn default() -> State {
        State {
            folder_selector_opened: false,
            input_folder_path: None,
            output_folder_path: None,
            default_home_dir: home_dir().unwrap_or_else(|| PathBuf::from("/")),
            status_message: "".into(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    FindInputFolder,
    FindOutputFolder,
    InputFolderPathed(String),
    OutputFolderPathed(String),
    StartRenaming,
    RenamingDone(Result<Vec<String>, String>),
}

async fn folder_selection(default_dir: PathBuf) -> String {
    match FileDialog::new()
        .set_directory(default_dir)
        .pick_folder()
    {
        Some(path) => path.to_string_lossy().to_string(),
        None => "".to_string(),
    }
}

// List files with given extension, sorted naturally
fn list_files_in_directory(path: &str, ext: &str) -> Result<Vec<PathBuf>, String> {
    let ext_lower = ext.to_lowercase();
    let mut entries: Vec<PathBuf> = fs::read_dir(Path::new(path))
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() {
                let matches = path.extension()
                    .map(|e| e.to_string_lossy().to_lowercase() == ext_lower)
                    .unwrap_or(false);
                if matches {
                    Some(path)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    entries.sort_by(|a, b| compare(a.to_string_lossy().as_ref(), b.to_string_lossy().as_ref()));
    Ok(entries)
}

// Generate new file names with leading zeros
fn rename_files_with_leading_zeros(files: &[PathBuf]) -> Vec<String> {
    files.iter()
        .enumerate()
        .map(|(i, path)| {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            format!("{:03}_{}", i + 1, file_name)
        })
        .collect()
}

// Rename files: move or copy to output folder with new names
fn rename_files(
    input_files: Vec<PathBuf>, 
    output_dir: &Path
) -> Result<Vec<String>, String> {
    fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;

    let new_names = rename_files_with_leading_zeros(&input_files);
    let mut result_names = Vec::new();

    for (old_path, new_name) in input_files.iter().zip(new_names.iter()) {
        let new_path = output_dir.join(new_name);

        // Here we rename (move) files; change to copy if needed
        fs::rename(old_path, &new_path).map_err(|e| e.to_string())?;

        result_names.push(new_path.to_string_lossy().to_string());
    }

    Ok(result_names)
}

async fn perform_renaming(input: Option<String>, output: Option<String>) -> Result<Vec<String>, String> {
    let input_path = input.ok_or("Input folder not selected".to_string())?;
    let output_path = output.ok_or("Output folder not selected".to_string())?;

    let files = list_files_in_directory(&input_path, "mp3")?;
    if files.is_empty() {
        return Err("No files with extension .mp3 found in input folder".to_string());
    }

    rename_files(files, Path::new(&output_path))
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::FindInputFolder => {
            if !state.folder_selector_opened {
                state.folder_selector_opened = true;
                let dir = state.input_folder_path
                    .as_ref()
                    .map_or(state.default_home_dir.clone(), |p| PathBuf::from(p));
                Task::perform(folder_selection(dir), Message::InputFolderPathed)
            } else {
                Task::none()
            }
        }
        Message::FindOutputFolder => {
            if !state.folder_selector_opened {
                state.folder_selector_opened = true;
                let dir = state.output_folder_path
                    .as_ref()
                    .map_or(state.default_home_dir.clone(), |p| PathBuf::from(p));
                Task::perform(folder_selection(dir), Message::OutputFolderPathed)
            } else {
                Task::none()
            }
        }
        Message::InputFolderPathed(path) => {
            if !path.is_empty() {
                let new_input = path.to_string();
                let should_update_output = match &state.output_folder_path {
                    None => true,
                    Some(current_output) => {
                        let input_path = PathBuf::from(&new_input);
                        let current_output_path = PathBuf::from(current_output);
                        current_output_path == input_path || current_output_path == input_path.join("output")
                    }
                };

                state.input_folder_path = Some(new_input.clone());

                if should_update_output {
                    state.output_folder_path = Some(PathBuf::from(new_input).join("output").to_string_lossy().to_string());
                }
            }
            state.folder_selector_opened = false;
            Task::none()
        }
        Message::OutputFolderPathed(path) => {
            if !path.is_empty() {
                state.output_folder_path = Some(path);
            }
            state.folder_selector_opened = false;
            Task::none()
        }
        Message::StartRenaming => {
            // Check if both input and output folders are selected
            if state.input_folder_path.is_none() || state.output_folder_path.is_none() {
                state.status_message = "Please select both input and output folders before starting.".into();
                Task::none()
            } else {
                let input = state.input_folder_path.clone();
                let output = state.output_folder_path.clone();
                Task::perform(
                    async move {
                        perform_renaming(input, output).await
                    },
                    Message::RenamingDone
                )
            }
        }
        Message::RenamingDone(result) => {
            match result {
                Ok(files) => {
                    state.status_message = format!("Renamed files:\n{}", files.join("\n"));
                }
                Err(e) => {
                    state.status_message = format!("Error: {}", e);
                }
            }
            Task::none()
        }
    }
}


fn view(state: &State) -> Element<Message> {
    let input_display = state.input_folder_path
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Click to select a folder --->");

    let output_display = state.output_folder_path
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Click to select a folder --->");

    container(
        column![
            text("Input folder"),
            row![
                text_input("Click to select a folder --->", input_display),
                button("+").on_press(Message::FindInputFolder),
            ],
            row![text("Output folder")].spacing(10),
            row![
                text_input("Click to select a folder --->", output_display),
                button("+").on_press(Message::FindOutputFolder),
            ],
            container(
                row![button("Start renaming").on_press(Message::StartRenaming)]
                .spacing(10)
            )
            .center_x(Fill),
            container(
                text(&state.status_message)
                    .size(14)
            )
            .padding(10)
            .width(iced::Length::Fill)
            .height(150)
        ],
    )
    .padding(10)
    .center_x(Fill)
    .into()
}
