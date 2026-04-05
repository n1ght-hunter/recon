use igloo_guest::{
    Element,
    widgets::{button, column, container, text},
};
use recon_guest::bus;

#[allow(missing_debug_implementations)]
pub struct TestPlugin {
    count: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
    Decrement,
}

impl igloo_guest::Application<TestPlugin, Message> for TestPlugin {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn view(&self) -> Element<Message> {
        let content: Element<Message> = column()
            .push(text(format!("Count: {}", self.count)).size(24.0))
            .push(button(text("Increment")).on_press(Message::Increment))
            .push(button(text("Decrement")).on_press(Message::Decrement))
            .spacing(10.0)
            .into();

        container(content).padding(20.0).into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.count += 1;
                let _ = bus::publish("test/count", &self.count.to_string());
            }
            Message::Decrement => {
                self.count -= 1;
                let _ = bus::publish("test/count", &self.count.to_string());
            }
        }
    }
}

igloo_guest::export_guest!(TestPlugin, Message);
