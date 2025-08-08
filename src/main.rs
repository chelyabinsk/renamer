mod file_ops;
mod tasks;
mod ui;
use iced::{Settings};
use std::borrow::Cow;

const MONO_FONT: &[u8] = include_bytes!("../assets/NotoSansMono-Regular.ttf");


fn main() -> iced::Result {
    let settings = Settings {
        fonts: vec![Cow::Borrowed(MONO_FONT)],
        ..Settings::default()
    };

    iced::application("Renamer", ui::update, ui::view)
        .settings(settings)
        .run()
}