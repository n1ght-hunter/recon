// mod test_component;

use std::collections::HashMap;

use futures::executor::block_on;
use iced::{
    executor,
    pure::{
        column, container, pick_list, row, vertical_space,
        widget::{Button, Column, Text},
        Application, Element,
    },
    Alignment, Command, Length,
};
use iced_pure::horizontal_space;
use serde::{Deserialize, Serialize};
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus;

use crate::{
    key_watcher::rdev::Key,
    media_controls::{
        controller::{app_instance, application_task, status, status_state_to_string, Controls},
        sources::media_sources,
    },
    media_listener::storage_control::{MediaAction, CURRENT_MEDIA},
    settings::load_file,
};


#[derive(Default)]
pub struct MediaControl {
    app_state: GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    current_application: Option<String>,
    current_hotkey: Option<HashMap<MediaAction, Vec<Key>>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    SetCurrentApplication(Option<String>),
    SetCurrentHotkeys(Option<HashMap<MediaAction, Vec<Key>>>),
    PrintValue(String),
}

impl Application for MediaControl {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self::default(),
            Command::perform(load(), Message::SetCurrentApplication),
        )
    }

    fn title(&self) -> String {
        String::from("Recon")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Play => match &self.current_application {
                Some(x) => {
                    let app = block_on(app_instance(x));
                    if let Ok(app) = app {
                        block_on(application_task(app, &Controls::Play));
                        let app = block_on(app_instance(x)).unwrap();
                        self.app_state = status(app);
                    }
                }
                None => {
                    println!("No application selected");
                }
            },
            Message::Pause => match &self.current_application {
                Some(x) => {
                    let app = block_on(app_instance(x));
                    if let Ok(app) = app {
                        block_on(application_task(app, &Controls::Pause));
                        let app = block_on(app_instance(x)).unwrap();
                        self.app_state = status(app);
                    }
                }
                None => {
                    println!("No application selected");
                }
            },
            Message::Stop => match &self.current_application {
                Some(x) => {
                    let app = block_on(app_instance(x));
                    if let Ok(app) = app {
                        block_on(application_task(app, &Controls::Stop));
                        let app = block_on(app_instance(x));
                        if app.is_ok() {
                            let app = app.unwrap();
                            self.app_state = status(app);
                        }
                    }
                }
                None => {
                    println!("No application selected");
                }
            },
            Message::Next => match &self.current_application {
                Some(x) => {
                    let app = block_on(app_instance(x));
                    if let Ok(app) = app {
                        block_on(application_task(app, &Controls::Next));
                        let app = block_on(app_instance(x)).unwrap();
                        self.app_state = status(app);
                    }
                }
                None => {
                    println!("No application selected");
                }
            },
            Message::Previous => match &self.current_application {
                Some(x) => {
                    let app = block_on(app_instance(x));
                    if let Ok(app) = app {
                        block_on(application_task(app, &Controls::Previous));
                        let app = block_on(app_instance(x)).unwrap();
                        self.app_state = status(app);
                    }
                }
                None => {
                    println!("No application selected");
                }
            },
            Message::SetCurrentApplication(name) => match name {
                Some(x) => {
                    let app = block_on(app_instance(&x));
                    if let Ok(app) = app {
                        self.app_state = status(app);
                        self.current_application = Some(x.clone());
                        self.current_hotkey = load_hot_keys(x.clone());
                    }
                }
                None => {
                    self.current_application = None;
                }
            },
            Message::SetCurrentHotkeys(hotkeys) => {
                self.current_hotkey = hotkeys;
            }
            Message::PrintValue(value) => {
                println!("{}", value);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let controls: Column<Message> = column()
            .padding(20)
            .align_items(Alignment::Center)
            .push(Text::new(status_state_to_string(self.app_state)).size(50))
            .push(Button::new(Text::new("Play")).on_press(Message::Play))
            .push(Button::new(Text::new("Pause")).on_press(Message::Pause))
            .push(Button::new(Text::new("Stop")).on_press(Message::Stop))
            .push(Button::new(Text::new("Next")).on_press(Message::Next))
            .push(Button::new(Text::new("Previous")).on_press(Message::Previous));

        let pick_list = pick_list(
            block_on(media_sources()),
            self.current_application.clone(),
            |x| Message::SetCurrentApplication(Some(x)),
        )
        .placeholder("Choose an application");

        let children = |input: Option<HashMap<MediaAction, Vec<Key>>>, action: String| {
            let keys = if input.is_some() {
                if input.clone().unwrap().get(&action).is_some() {
                    let key_strings = input
                        .unwrap()
                        .get(&action)
                        .unwrap()
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>();
                    key_strings.join(" + ")
                } else {
                    "No keys".to_string()
                }
            } else {
                "No hashmap".to_string()
            };
            row()
                .push(Text::new(keys))
                .push(horizontal_space(Length::Units(40)))
                .push(Button::new("Remove Hotkey").on_press(Message::PrintValue(action.clone())))
                .push(Button::new("Add Hotkey").on_press(Message::PrintValue(action)))
        };

        let test = column()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .push(children(
                self.current_hotkey.clone(),
                Controls::Play.to_string(),
            ))
            .push(children(
                self.current_hotkey.clone(),
                Controls::Pause.to_string(),
            ))
            .push(children(
                self.current_hotkey.clone(),
                Controls::Stop.to_string(),
            ))
            .push(children(
                self.current_hotkey.clone(),
                Controls::Next.to_string(),
            ))
            .push(children(
                self.current_hotkey.clone(),
                Controls::Previous.to_string(),
            ));

        let content = column()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .spacing(10)
            .push(vertical_space(Length::Units(100)))
            .push(Text::new(
                self.current_application
                    .clone()
                    .unwrap_or("No application selected".to_string()),
            ))
            .push(vertical_space(Length::Units(40)))
            .push(pick_list)
            .push(vertical_space(Length::Units(600)));

        let base = row()
            .width(Length::Fill)
            .push(content)
            .push(controls)
            .push(test);

        // let left = Column::new()
        //
        // let right = <PickListAPP as Sandbox>::new();
        container(base)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct GuiPersist {
    pub selected_media: Option<String>,
}

async fn load() -> Option<String> {
    let result = load_file::<GuiPersist>("./src/persist/gui.json");
    if result.is_ok() {
        let settings = result.unwrap();
        if settings.selected_media.is_some()
            && media_sources()
                .await
                .contains(&settings.selected_media.clone().unwrap())
        {
            return Some(settings.selected_media.unwrap());
        }
    }
    None
}

fn load_hot_keys(curret_application: String) -> Option<HashMap<MediaAction, Vec<Key>>> {
    unsafe {
        // println!("{}", curret_application);
        let result = &CURRENT_MEDIA;
        // println!("{:?}", result);
        if result.is_some() {
            let media = result.clone().unwrap();
            if media.get(&curret_application.clone()).is_some() {
                return Some(media.get(&curret_application).unwrap().clone());
            }
        }
    }
    None
}
