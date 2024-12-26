use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use iced::{widget::image, Application, Command, Element, Settings};

fn main() -> iced::Result {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
