#![windows_subsystem = "windows"]

pub(crate) mod utils;

use iced::{
    Subscription,
    widget::{center, text},
    window,
};

fn main() -> iced::Result {
    utils::attach();

    iced::daemon(Recon::new, Recon::update, Recon::view)
        .title(Recon::title)
        .subscription(Recon::subscription)
        .run()
}

#[derive(Debug)]
struct Recon {
    main_window: window::Id,
}

#[derive(Debug, Clone)]
enum Message {
    WindowOpened,
    WindowClosed(window::Id),
}

impl Recon {
    fn new() -> (Self, iced::Task<Message>) {
        let (id, open) = window::open(window::Settings {
            size: iced::Size::new(400.0, 300.0),
            ..Default::default()
        });

        (
            Self { main_window: id },
            open.map(|_| Message::WindowOpened),
        )
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::WindowOpened => iced::Task::none(),
            Message::WindowClosed(id) if id == self.main_window => iced::exit(),
            Message::WindowClosed(_) => iced::Task::none(),
        }
    }

    fn title(&self, _window: window::Id) -> String {
        String::from("Recon")
    }

    fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }

    fn view(&self, _window: window::Id) -> iced::Element<'_, Message> {
        center(text("Recon").size(24)).into()
    }
}
