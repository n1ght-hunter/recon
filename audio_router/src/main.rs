mod sound_devices;

use iced::{
    executor,
    widget::{self, column, container, row, text},
    Alignment, Application, Command, Element, Length, Settings, Theme,
};
use iced_audio::{ModulationRange, Normal};
use sound_devices::{output_hardware_device::OutputHardwareDevice, SoundDevices};

fn main() {
    AudioRouter::run(Settings::default()).unwrap();
}

#[derive(Debug, Clone)]
pub struct AudioRouter {
    sound_devices: SoundDevices,
    range: iced_audio::ModulationRange,
}

#[derive(Debug, Clone)]
pub enum Message {
    MasterSlider(String, f32),
    Slider,
}

impl Application for AudioRouter {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                sound_devices: SoundDevices::new(),
                range: iced_audio::ModulationRange::new(0.0.into(), 1.0.into()),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Audio Router")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::MasterSlider(name, value) => {
                self.sound_devices.output_devices.iter().for_each(|x| {
                    if x.get_master_name().unwrap() == name {
                        x.set_master_volume_scaler(value).unwrap()
                    }
                })
            }
            Message::Slider => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let inputs = row(self
            .sound_devices
            .input_devices
            .iter()
            .map(|x| text(x.get_name().unwrap()).into())
            .collect());

        let outputs = column(
            self.sound_devices
                .output_devices
                .iter()
                .map(|x| display_output(x, self.range))
                .collect(),
        );

        container(widget::column![inputs, outputs])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn display_output(
    hardward_device: &OutputHardwareDevice,
    range: ModulationRange,
) -> iced::Element<Message, iced::Renderer> {
    let name_text = hardward_device.get_master_name().unwrap();
    let name = text(name_text.clone()).size(15);
    let slider = iced_audio::VSlider::new(
        iced_audio::NormalParam {
            value: hardward_device.get_master_volume().unwrap().into(),
            default: 1.0.into(),
        },
        Box::new(move |x: iced_audio::Normal| Message::MasterSlider(name_text.clone(), x.as_f32())),
    )
    .mod_range(&range);
    let master_slider = container(widget::column![name, slider].align_items(Alignment::Center))
        .width(Length::Units(70))
        .height(Length::Units(250));

    let mut row_output: Vec<Element<_>> = vec![master_slider.into()];

    hardward_device.sessions.iter().for_each(|f| {
        let name_text = f.get_name();
        let name = text(name_text.clone()).size(15);
        let slider = iced_audio::VSlider::new(
            iced_audio::NormalParam {
                value: Normal::new(f.get_volume().unwrap()),
                default: 1.0.into(),
            },
            move |x: iced_audio::Normal| {
                f.set_volume(x.as_f32()).unwrap();
                Message::Slider
            },
        )
        .mod_range(&range);

        let con = container(widget::column![name, slider].align_items(Alignment::Center))
            .width(Length::Units(70))
            .height(Length::Units(250));
        row_output.push(con.into());
    });

    widget::Row::with_children(row_output).into()
}
