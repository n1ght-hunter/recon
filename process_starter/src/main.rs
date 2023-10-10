use std::sync::Arc;

use iced::{widget::text, Application, Settings};
use process_watcher::process_watcher;
use tokio::sync::Mutex;

pub mod process_watcher;

fn main() {
    ProcessWatcherGui::run(Settings::default()).unwrap();
}

struct ProcessWatcherGui {
    process_paths: Arc<Mutex<Vec<String>>>,
}

#[derive(Debug)]
pub enum Message {
    None,
}

impl Application for ProcessWatcherGui {
    type Executor = iced::executor::Default;

    type Message = Message;

    type Theme = iced::Theme;

    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            ProcessWatcherGui {
                process_paths: Arc::new(Mutex::new(Vec::new())),
            },
            iced::Command::none(),
        )
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        process_watcher()
    }

    fn title(&self) -> String {
        String::from("Process Watcher")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        text("Hello, world!").into()
    }
}
