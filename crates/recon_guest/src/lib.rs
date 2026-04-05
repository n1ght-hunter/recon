//! Guest SDK for Recon plugins.
//!
//! Generates bindings from the `recon-app` world which includes both
//! iced UI widgets and the event bus. Plugins should depend only on
//! this crate — not on `igloo_guest` directly.

#![allow(clippy::too_many_arguments)]

pub use igloo_guest::{self, Application, widgets};

wit_bindgen::generate!({
    path: "../../wit",
    world: "recon-app",
    with: {
        "iced:app/shared@0.1.0": igloo_guest::bindings::iced::app::shared,
        "iced:app/length@0.1.0": igloo_guest::bindings::iced::app::length,
        "iced:app/alignment@0.1.0": igloo_guest::bindings::iced::app::alignment,
        "iced:app/text@0.1.0": igloo_guest::bindings::iced::app::text,
        "iced:app/padding@0.1.0": igloo_guest::bindings::iced::app::padding,
        "iced:app/column@0.1.0": igloo_guest::bindings::iced::app::column,
        "iced:app/row@0.1.0": igloo_guest::bindings::iced::app::row,
        "iced:app/container@0.1.0": igloo_guest::bindings::iced::app::container,
        "iced:app/tooltip@0.1.0": igloo_guest::bindings::iced::app::tooltip,
        "iced:app/message-types@0.1.0": igloo_guest::bindings::iced::app::message_types,
        "iced:app/message@0.1.0": igloo_guest::bindings::iced::app::message,
        "iced:app/button@0.1.0": igloo_guest::bindings::iced::app::button,
        "iced:app/rule@0.1.0": igloo_guest::bindings::iced::app::rule,
        "iced:app/checkbox@0.1.0": igloo_guest::bindings::iced::app::checkbox,
        "iced:app/combo-box@0.1.0": igloo_guest::bindings::iced::app::combo_box,
        "iced:app/float@0.1.0": igloo_guest::bindings::iced::app::float,
        "iced:app/grid@0.1.0": igloo_guest::bindings::iced::app::grid,
        "iced:app/progress-bar@0.1.0": igloo_guest::bindings::iced::app::progress_bar,
        "iced:app/toggler@0.1.0": igloo_guest::bindings::iced::app::toggler,
        "iced:app/radio@0.1.0": igloo_guest::bindings::iced::app::radio,
        "iced:app/image@0.1.0": igloo_guest::bindings::iced::app::image,
        "iced:app/keyed@0.1.0": igloo_guest::bindings::iced::app::keyed,
        "iced:app/markdown@0.1.0": igloo_guest::bindings::iced::app::markdown,
        "iced:app/pane-grid@0.1.0": igloo_guest::bindings::iced::app::pane_grid,
        "iced:app/pick-list@0.1.0": igloo_guest::bindings::iced::app::pick_list,
        "iced:app/slider@0.1.0": igloo_guest::bindings::iced::app::slider,
        "iced:app/vertical-slider@0.1.0": igloo_guest::bindings::iced::app::vertical_slider,
        "iced:app/svg@0.1.0": igloo_guest::bindings::iced::app::svg,
        "iced:app/table@0.1.0": igloo_guest::bindings::iced::app::table,
        "iced:app/text-input@0.1.0": igloo_guest::bindings::iced::app::text_input,
        "iced:app/space@0.1.0": igloo_guest::bindings::iced::app::space,
        "iced:app/scrollable@0.1.0": igloo_guest::bindings::iced::app::scrollable,
        "iced:app/element@0.1.0": igloo_guest::bindings::iced::app::element,
        "recon:event-bus/bus@0.1.0": generate,
    },
});

pub mod bus {
    pub use super::recon::event_bus::bus::*;
}
