mod helper_functions;
mod input_hardware_device;
mod output_hardware_device;

use std::{
    convert::TryInto,
    ffi::{c_void, OsString},
    mem,
    os::windows::prelude::OsStringExt,
    ptr::{self},
    slice,
};

use image::ImageFormat;
use windows::{
    core::Interface,
    Win32::{
        Devices::Properties,
        Foundation::HANDLE,
        Media::Audio::{
            self, eAll, eCapture, eMultimedia, eRender, Endpoints::IAudioEndpointVolume,
            IAudioSessionControl2, IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator,
            MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        },
        System::{
            Com::{
                self, CoCreateInstance, CoInitializeEx,
                StructuredStorage::{self, PROPVARIANT},
                CLSCTX_ALL, COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED,
            },
            Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        },
    },
};

use crate::audio_router::helper_functions::get_icon;

use self::{
    helper_functions::{get_hardware_device_name, get_process_name},
    input_hardware_device::InputHardwareDevice,
    output_hardware_device::OutputHardwareDevice,
};

// supported interfaces MME, KS, WASAPI, Direct-X, and ASIO
pub fn audio_router() {
    unsafe {
        CoInitializeEx(ptr::null(), COINIT_MULTITHREADED).unwrap();
        let sound_devices = SoundDevices::new();
        sound_devices.input_devices.iter().for_each(|device| {
            let name = device.get_name().unwrap();
            println!("Input \n name: {} ", name,);
        });
        sound_devices.output_devices.iter().for_each(|device| {
            let name = device.get_master_name().unwrap();
            let seestion_name = device
                .sessions
                .iter()
                .map(|i| i.get_name())
                .collect::<Vec<String>>();
            println!("Ouput \n name: {} sestions: {:?}", name, seestion_name);
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundDevices {
    input_devices: Vec<InputHardwareDevice>,
    output_devices: Vec<OutputHardwareDevice>,
    default_output: OutputHardwareDevice,
    default_input: InputHardwareDevice,
}

impl SoundDevices {
    unsafe fn new() -> SoundDevices {
        let guuid: *const ::windows::core::GUID = &MMDeviceEnumerator;

        let device_enumerator: IMMDeviceEnumerator =
            CoCreateInstance(guuid, None, CLSCTX_ALL).unwrap();

        let input_audio_endpoints = device_enumerator
            .EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)
            .unwrap();
        let output_audio_endpoints = device_enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .unwrap();

        let input_endpoints = (0..input_audio_endpoints.GetCount().unwrap())
            .into_iter()
            .map(|number| InputHardwareDevice::new(input_audio_endpoints.Item(number).unwrap()))
            .collect::<Vec<InputHardwareDevice>>();

        let output_endpoints = (0..output_audio_endpoints.GetCount().unwrap())
            .into_iter()
            .map(|number| OutputHardwareDevice::new(output_audio_endpoints.Item(number).unwrap()))
            .collect::<Vec<OutputHardwareDevice>>();

        let default_input = device_enumerator
            .GetDefaultAudioEndpoint(eCapture, eMultimedia)
            .unwrap();
        let default_output = device_enumerator
            .GetDefaultAudioEndpoint(eRender, eMultimedia)
            .unwrap();

        SoundDevices {
            default_input: InputHardwareDevice::new(default_input),
            default_output: OutputHardwareDevice::new(default_output),
            input_devices: input_endpoints,
            output_devices: output_endpoints,
        }
    }
}