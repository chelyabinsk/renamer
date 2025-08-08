use iced::widget::{button, row, column, text, container, text_input, progress_bar};
use iced::{Element, Length, Task};
use dirs_next::home_dir;
use std::path::{PathBuf};
use iced::widget::{PickList};
use iced::widget::Checkbox;

const PADDING_OPTIONS: [usize; 5] = [1, 2, 3, 4, 5];

use crate::file_ops::{
    list_files_in_directory,
    rename_files_with_leading_zeros,
};

use crate::tasks::{
    folder_selection,
    perform_renaming_with_progress,
};

pub struct State {
    pub folder_selector_opened: bool,
    pub input_folder_path: Option<String>,
    pub output_folder_path: Option<String>,
    pub default_home_dir: PathBuf,
    pub status_message: String,
    pub file_extension: String,
    pub total_files: usize,
    pub renamed_count: usize,
    pub renaming_in_progress: bool,
    pub original_preview: Vec<String>,
    pub renamed_preview: Vec<String>,
    pub padding_zeros: usize,
    pub include_original_name: bool,
    pub auto_padding: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            folder_selector_opened: false,
            input_folder_path: None,
            output_folder_path: None,
            default_home_dir: home_dir().unwrap_or_else(|| PathBuf::from("/")),
            status_message: "".into(),
            file_extension: "mp3".into(),
            total_files: 0,
            renamed_count: 0,
            renaming_in_progress: false,
            original_preview: vec![],
            renamed_preview: vec![],
            padding_zeros: 3,
            include_original_name: true,
            auto_padding: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    FindInputFolder,
    FindOutputFolder,
    InputFolderPathed(String),
    OutputFolderPathed(String),
    StartRenaming,
    RenamingDone(Result<Vec<String>, String>),
    ExtensionChanged(String),
    RenamingProgress(usize, usize),
    PaddingChanged(usize),
    IncludeOriginalNameChanged(bool),
    SetAutoPadding(bool),
}

fn compute_auto_padding(total_files: usize) -> usize {
    if total_files == 0 {
        3 // default minimum padding
    } else {
        (total_files as f64).log10().ceil() as usize + 1
    }
}

fn to_display_string(path: &PathBuf) -> String {
    match path.to_str() {
        Some(valid) => valid.to_string(), // Safe UTF-8 path
        None => path.to_string_lossy().into_owned(), // Fall back to lossy conversion
    }
}

fn update_preview(state: &mut State) {
    if let Some(input_path) = &state.input_folder_path {
        let ext = &state.file_extension;
        match list_files_in_directory(input_path, ext) {
            Ok(files) => {
                state.total_files = files.len();

                if state.auto_padding {
                    state.padding_zeros = compute_auto_padding(state.total_files);
                }

                if files.is_empty() {
                    state.status_message = format!("No files with extension .{} found in input folder.", ext);
                    state.original_preview.clear();
                    state.renamed_preview.clear();
                } else {
                    let renamed_names = rename_files_with_leading_zeros(&files, state.padding_zeros, state.include_original_name);

                    state.original_preview = files.iter()
                        .map(to_display_string)
                        .take(20)
                        .collect();

                    let output_dir = PathBuf::from(&state.output_folder_path.clone().unwrap_or_default());

                    state.renamed_preview = renamed_names
                        .iter()
                        .map(|name| to_display_string(&output_dir.join(name)))
                        .take(20)
                        .collect();

                    state.status_message = format!("Found {} files (preview limited to 20).", files.len());
                }
            }
            Err(e) => {
                state.status_message = format!("Error reading input folder: {}", e);
                state.total_files = 0;
                state.original_preview.clear();
                state.renamed_preview.clear();
            }
        };
       
    }
}


// --- Update function ---
pub fn update(state: &mut State, message: Message) -> Task<Message> {
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
                state.input_folder_path = Some(new_input.clone());
                state.output_folder_path = Some(PathBuf::from(new_input.clone()).join("output").to_string_lossy().to_string());

                let ext = &state.file_extension;
                match list_files_in_directory(&new_input, ext) {
                    Ok(files) => {
                        state.total_files = files.len();
                        if files.is_empty() {
                            state.status_message = format!("No files with extension .{} found in input folder.", ext);
                            state.original_preview.clear();
                            state.renamed_preview.clear();
                        } else {
                            let renamed_names = rename_files_with_leading_zeros(&files, state.padding_zeros, state.include_original_name);

                            state.original_preview = files.iter()
                                .map(|p| p.to_string_lossy().to_string())
                                .take(20)
                                .collect();

                            let output_dir = PathBuf::from(&state.output_folder_path.clone().unwrap_or_default());

                            state.renamed_preview = renamed_names
                                .iter()
                                .map(|name| output_dir.join(name).to_string_lossy().to_string())
                                .take(20)
                                .collect();

                            state.status_message = format!("Found {} files (preview limited to 20).", files.len());
                        }
                    }
                    Err(e) => {
                        state.status_message = format!("Error reading input folder: {}", e);
                        state.total_files = 0;
                        state.original_preview.clear();
                        state.renamed_preview.clear();
                    }
                };
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
            if state.input_folder_path.is_none() || state.output_folder_path.is_none() {
                state.status_message = "Please select both input and output folders before starting.".into();
                Task::none()
            } else {
                state.renaming_in_progress = true;
                state.renamed_count = 0;
                state.total_files = 0;

                let input = state.input_folder_path.clone();
                let output = state.output_folder_path.clone();
                let ext = state.file_extension.clone();

                Task::stream(perform_renaming_with_progress(
                    input,
                    output,
                    ext,
                    state.padding_zeros,
                    state.include_original_name
                ))
            }
        }
        Message::RenamingDone(result) => {
            state.renaming_in_progress = false;
            match result {
                Ok(files) => {
                    state.status_message = format!("Renaming complete! {} files renamed.", files.len());
                }
                Err(e) => {
                    state.status_message = format!("Error: {}", e);
                }
            }
            Task::none()
        }
        Message::ExtensionChanged(ext) => {
            state.file_extension = ext.trim_start_matches('.').to_string();
            update_preview(state);
            Task::none()
        }
        Message::RenamingProgress(done, total) => {
            state.renamed_count = done;
            state.total_files = total;
            state.status_message = format!("Renaming... {}/{}", done, total);
            Task::none()
        },
        Message::PaddingChanged(value) => {
            state.padding_zeros = value;
            update_preview(state);
            Task::none()
        },

        Message::IncludeOriginalNameChanged(include) => {
            state.include_original_name = include;
            update_preview(state);
            Task::none()
        },
        Message::SetAutoPadding(auto) => {
            state.auto_padding = auto;
            update_preview(state);
            Task::none()
        },
    }
}

// --- View function ---
pub fn view(state: &State) -> Element<Message> {
    let input_display = state.input_folder_path
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Click to select a folder --->");

    let output_display = state.output_folder_path
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Click to select a folder --->");

    let progress_value = if state.total_files == 0 {
        0.0
    } else if state.renaming_in_progress {
        state.renamed_count as f32 / state.total_files as f32
    } else {
        1.0
    };

    let original_preview_list = state.original_preview.iter().fold(
        column![],
        |col, file_name| col.push(text(file_name).size(14)),
    );

    let renamed_preview_list = state.renamed_preview.iter().fold(
        column![],
        |col, file_name| col.push(text(file_name).size(14)),
    );

    let previews = row![
        column![
            text("Original file names:").size(16),
            original_preview_list
        ]
        .width(Length::FillPortion(1)),

        column![
            text("Renamed file names:").size(16),
            renamed_preview_list
        ]
        .width(Length::FillPortion(1))
    ];

    let progress = progress_bar(0.0..=1.0, progress_value)
        .width(Length::Fill)
        .height(20);

    let main_content = column![
        text("Input folder"),
        row![
            text_input("Click to select a folder --->", input_display),
            text_input(
                "e.g. mp3",
                if state.file_extension.is_empty() { "" } else { &state.file_extension }
            )
            .on_input(Message::ExtensionChanged)
            .width(100),
            button("+").on_press(Message::FindInputFolder),
        ],
        row![ text("Output folder") ].spacing(10),
        row![
            text_input("Click to select a folder --->", output_display),
            button("+").on_press(Message::FindOutputFolder),
        ],

        column![
            text("Number of leading zeros (e.g. 001, 002...)").size(14),
            Checkbox::new(
                "Automatic padding",
                state.auto_padding,
            )
            .on_toggle(|checked| if checked { Message::SetAutoPadding(true) } else { Message::SetAutoPadding(false) }),
            PickList::new(
                &PADDING_OPTIONS[..],
                Some(state.padding_zeros),
                Message::PaddingChanged,
            )
            .placeholder("Padding")
            .width(100),

            Checkbox::new(
                "Include original name in the new filename",
                state.include_original_name,
            )
            .on_toggle(Message::IncludeOriginalNameChanged)
            .spacing(10),
        ]
        .spacing(5),

        column![
            text(&state.status_message).size(14),
            previews,
        ]
        .width(Length::FillPortion(1))
        .padding(10),

        container(
            row![ button("Start renaming").on_press(Message::StartRenaming) ].spacing(10)
        )
        .center_x(Length::Fill),
    ]
    .spacing(10)
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fill);

    container(
        column![
            main_content,
            container(progress)
                .padding(10)
                .width(Length::Fill)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
