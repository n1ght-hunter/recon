mod helpers;

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use iced::{
    widget::{container, image},
    Element, Task,
};

fn main() {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    iced::application("recon", update, view).run().unwrap();
}

#[derive(Debug, Default)]
struct Recon;

#[derive(Debug)]
pub enum Message {}

fn update(_state: &mut Recon, _message: Message) -> Task<Message> {
    info!("update");
    Task::none()
}

fn view(_app: &Recon) -> Element<Message> {
    container("recon").into()
}
