use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
mod window_capture_sub;

use iced::{widget::image, Application, Command, Element, Settings};

fn main() -> iced::Result {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    Recon::run(Settings::default())
}

struct Recon {
    image: iced::widget::image::Handle,
}

#[derive(Debug, Clone)]
pub enum Message {
    ButtonPressed,
    NewFrame(iced::widget::image::Handle),
}

impl Application for Recon {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Recon, Command<Message>) {
        (Recon {
            image: iced::widget::image::Handle::from_pixels(1, 1, vec![0, 0, 0, 0]),
        }, Command::none())
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Dark
    }

    fn title(&self) -> String {
        String::from("My Iced App")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        window_capture_sub::create_window_capture_sub()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ButtonPressed => {
                println!("Button pressed!");
            }
            Message::NewFrame(image) => {
                self.image = image;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        image(self.image.clone()).into()
    }
}
