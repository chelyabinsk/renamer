mod file_ops;
mod tasks;
mod ui;

fn main() -> iced::Result {
    iced::application("Renamer", ui::update, ui::view).run()
}