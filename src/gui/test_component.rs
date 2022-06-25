use iced::pure::{button, row, text, text_input};
use iced_lazy::pure::{self, Component};
use iced_native::alignment::{self, Alignment};
use iced_native::text;
use iced_native::Length;
use iced_pure::Element;

pub struct MediaControlInput<Message> {
    value: Option<u32>,
    on_change: Box<dyn Fn(Option<u32>) -> Message>,
}

pub fn numeric_input<Message>(
    value: Option<u32>,
    on_change: impl Fn(Option<u32>) -> Message + 'static,
) -> MediaControlInput<Message> {
    MediaControlInput::new(value, on_change)
}

#[derive(Debug, Clone)]
pub enum Event {
    InputChanged(String),
    IncrementPressed,
    DecrementPressed,
}

impl<Message> MediaControlInput<Message> {
    pub fn new(
        value: Option<u32>,
        on_change: impl Fn(Option<u32>) -> Message + 'static,
    ) -> Self {
        Self {
            value,
            on_change: Box::new(on_change),
        }
    }
}

impl<Message, Renderer> Component<Message, Renderer> for MediaControlInput<Message>
where
    Renderer: text::Renderer + 'static,
{
    type State = ();
    type Event = Event;

    fn update(
        &mut self,
        _state: &mut Self::State,
        event: Event,
    ) -> Option<Message> {
        match event {
            Event::IncrementPressed => Some((self.on_change)(Some(
                self.value.unwrap_or_default().saturating_add(1),
            ))),
            Event::DecrementPressed => Some((self.on_change)(Some(
                self.value.unwrap_or_default().saturating_sub(1),
            ))),
            Event::InputChanged(value) => {
                if value.is_empty() {
                    Some((self.on_change)(None))
                } else {
                    value
                        .parse()
                        .ok()
                        .map(Some)
                        .map(self.on_change.as_ref())
                }
            }
        }
    }

    fn view(&self, _state: &Self::State) -> Element<Event, Renderer> {
        let button = |label, on_press| {
            button(
                text(label)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center),
            )
            .width(Length::Units(50))
            .on_press(on_press)
        };

        row()
            .push(button("-", Event::DecrementPressed))
            .push(
                text_input(
                    "Type a number",
                    self.value
                        .as_ref()
                        .map(u32::to_string)
                        .as_ref()
                        .map(String::as_str)
                        .unwrap_or(""),
                    Event::InputChanged,
                )
                .padding(10),
            )
            .push(button("+", Event::IncrementPressed))
            .align_items(Alignment::Fill)
            .spacing(10)
            .into()
    }
}

impl<'a, Message, Renderer> From<MediaControlInput<Message>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'static + text::Renderer,
{
    fn from(numeric_input: MediaControlInput<Message>) -> Self {
        pure::component(numeric_input)
    }
}