#![windows_subsystem = "windows"]

pub(crate) mod utils;

use iced::{
    Subscription,
    widget::{center, text},
    window,
};
use igloo::plugin_manager::PluginManager;

fn main() -> iced::Result {
    utils::attach();

    iced::daemon(Recon::new, Recon::update, Recon::view)
        .title(Recon::title)
        .subscription(Recon::subscription)
        .run()
}

struct Recon {
    main_window: window::Id,
    plugins: PluginManager,
}

#[derive(Debug, Clone)]
enum Message {
    WindowOpened,
    WindowClosed(window::Id),
    Plugin(String, igloo::Message),
}

impl Recon {
    fn new() -> (Self, iced::Task<Message>) {
        let (id, open) = window::open(window::Settings {
            size: iced::Size::new(800.0, 600.0),
            ..Default::default()
        });

        let mut plugins = PluginManager::new().expect("failed to create plugin manager");
        plugins
            .add_plugin_from_file(
                "test",
                "target/wasm32-wasip2/release/test_plugin.wasm",
            )
            .expect("failed to load test plugin");

        (
            Self {
                main_window: id,
                plugins,
            },
            open.map(|_| Message::WindowOpened),
        )
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::WindowOpened => iced::Task::none(),
            Message::WindowClosed(id) if id == self.main_window => iced::exit(),
            Message::WindowClosed(_) => iced::Task::none(),
            Message::Plugin(id, msg) => {
                if let Err(e) = self.plugins.plugin_update(&id, msg) {
                    tracing::error!("plugin update error for {id}: {e}");
                }
                iced::Task::none()
            }
        }
    }

    fn title(&self, _window: window::Id) -> String {
        String::from("Recon")
    }

    fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }

    fn view(&self, window: window::Id) -> iced::Element<'_, Message> {
        if window != self.main_window {
            return center(text("Unknown window")).into();
        }

        let mut elements: Vec<iced::Element<'_, Message>> = vec![text("Recon").size(24).into()];

        self.plugins.ids().into_iter().for_each(|id| {
            if let Some(plugin_el) = self.plugins.plugin_view(&id) {
                let id_clone = id.clone();
                elements.push(plugin_el.map(move |m| Message::Plugin(id_clone.clone(), m)));
            }
        });

        center(iced::widget::Column::from_vec(elements)).into()
    }
}
