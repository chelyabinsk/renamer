mod file_ops;
mod tasks;
mod ui;
use iced::{Settings, Font};
use std::borrow::Cow;

const MONO_FONT_BYTES: &[u8] = include_bytes!("../assets/NotoSans-Regular.ttf");
const MONO_FONT_NAME: &str = "Noto Sans";

fn main() -> iced::Result {
    let settings = Settings {
        fonts: vec![Cow::Borrowed(MONO_FONT_BYTES)],
        default_font: Font::with_name(MONO_FONT_NAME),
        ..Settings::default()
    };

    iced::application("Renamer", ui::update, ui::view)
        .settings(settings)
        .run()
}