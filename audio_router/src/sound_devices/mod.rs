pub mod helper_functions;
pub mod input_hardware_device;
pub mod output_hardware_device;

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

use self::{
    helper_functions::{get_hardware_device_name, get_process_name},
    input_hardware_device::InputHardwareDevice,
    output_hardware_device::OutputHardwareDevice,
};


#[derive(Debug, Clone, PartialEq)]
pub struct SoundDevices {
    pub input_devices: Vec<InputHardwareDevice>,
    pub output_devices: Vec<OutputHardwareDevice>,
    pub default_output: OutputHardwareDevice,
    pub default_input: InputHardwareDevice,
}

impl SoundDevices {
    pub fn new() -> SoundDevices {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
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
                .map(|number| {
                    OutputHardwareDevice::new(output_audio_endpoints.Item(number).unwrap())
                })
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
}
