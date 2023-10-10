use std::ptr;

use windows::{
    core::Interface,
    Win32::{
        Media::Audio::{
            Endpoints::IAudioEndpointVolume, IAudioSessionControl2, IAudioSessionManager2,
            IMMDevice, ISimpleAudioVolume, IAudioSessionEvents,
        },
        System::{
            Com::{
                CoInitializeEx, StructuredStorage::PROPVARIANT, CLSCTX_ALL, COINIT_MULTITHREADED,
                STGM_READWRITE,
            },
            Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        },
    },
};

use super::helper_functions::{get_hardware_device_name, get_process_name};

#[derive(Debug, Clone, PartialEq)]
pub struct OutputHardwareDevice {
    pub device: IMMDevice,
    pub session_manager: IAudioSessionManager2,
    pub audio_endpoint_volume: IAudioEndpointVolume,
    pub sessions: Vec<MixerSession>,
}

impl OutputHardwareDevice {
    pub unsafe fn new(device: IMMDevice) -> OutputHardwareDevice {
        let session_manager = initialize_session_manager(&device);
        OutputHardwareDevice {
            sessions: get_mixer_sessions(&session_manager).unwrap(),
            session_manager: session_manager,
            audio_endpoint_volume: initialize_audio_endpoint(&device),
            device,
        }
    }

    pub unsafe fn get_master_range_db(self) -> Result<(f32, f32, f32), ()> {
        let min_volume = Box::into_raw(Box::new(0.0_f32));
        let max_volume = Box::into_raw(Box::new(0.0_f32));
        let volume_increment = Box::into_raw(Box::new(0.0_f32));
        if let Err(error) =
            self.audio_endpoint_volume
                .GetVolumeRange(min_volume, max_volume, volume_increment)
        {
            return Err(());
        }
        Ok((*min_volume, *max_volume, *volume_increment))
    }

    pub fn get_master_name(&self) -> Result<String, ()> {
        get_hardware_device_name(&self.device)
    }

    pub fn get_master_volume(&self) -> Result<f32, String> {
        unsafe {
            self.audio_endpoint_volume
                .GetMasterVolumeLevelScalar()
                .map_err(|e| format!("{:?}", e))
        }
    }

    /// volume between 0 and 1
    ///
    /// 0 for min and 1 for max
    pub fn set_master_volume_scaler(&self, volume: f32) -> Result<(), String> {
        unsafe {
            self.audio_endpoint_volume
                .SetMasterVolumeLevelScalar(volume, ptr::null())
                .map_err(|e| format!("{:?}", e))
        }
    }
    /// set volume to decibels
    ///
    /// use get db range to see range of dbs
    pub fn set_master_volume_db(&self, volume: f32) -> Result<(), String> {
        unsafe {
            self.audio_endpoint_volume
                .SetMasterVolumeLevel(volume, ptr::null())
                .map_err(|e| format!("{:?}", e))
        }
    }
}

unsafe fn initialize_audio_endpoint(device: &IMMDevice) -> IAudioEndpointVolume {
    let prop: *const PROPVARIANT = &PROPVARIANT::default();

    device
        .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
        .expect("unable to get IAudioEndpointVolume")
}

unsafe fn initialize_session_manager(device: &IMMDevice) -> IAudioSessionManager2 {
    device
        .Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)
        .expect("unable to get IAudioSessionManager2")
}

unsafe fn get_sessions(
    session_manager: &IAudioSessionManager2,
) -> Result<Vec<(IAudioSessionControl2, ISimpleAudioVolume)>, String> {
    let session_ernumerator = session_manager
        .GetSessionEnumerator()
        .expect("cant get session enumerator");

    let session_count = session_ernumerator.GetCount();
    if session_count.is_err() {
        return Err("unable to get sestions count".to_string());
    }
    let sessions = session_count.unwrap();
    Ok((0..sessions)
        .into_iter()
        .map(|id_number| {
            let session: IAudioSessionControl2 = session_ernumerator
                .GetSession(id_number)
                .unwrap()
                .cast()
                .expect("cant cast to IAudioSessionControl2");
            let id = session.GetGroupingParam().unwrap();
            let vol = session_manager
                .GetSimpleAudioVolume(Some( &id), 1)
                .unwrap();

            (session, vol)
        })
        .collect::<Vec<(IAudioSessionControl2, ISimpleAudioVolume)>>())
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixerSession {
    name: String,
    session: IAudioSessionControl2,
    simple_volume: ISimpleAudioVolume,
}

impl MixerSession {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_session(&self) -> &IAudioSessionControl2 {
        &self.session
    }

    pub fn get_volume(&self) -> Result<f32, windows::core::Error> {
        unsafe { self.simple_volume.GetMasterVolume() }
    }

    pub fn set_volume(&self, level: f32) -> Result<(), windows::core::Error> {
        unsafe { self.simple_volume.SetMasterVolume(level, &IAudioSessionEvents::IID) }
    }
}

unsafe fn get_mixer_sessions(
    session_manager: &IAudioSessionManager2,
) -> Result<Vec<MixerSession>, String> {
    let mut return_devices: Vec<MixerSession> = Vec::new();
    for (session, simple_volume) in get_sessions(session_manager).unwrap().into_iter() {
        let session_name = session.GetDisplayName().unwrap().to_string().unwrap();

        let name = if session_name.len() > 0 {
            session_name
        } else {
            let id = session.GetProcessId().unwrap();
            // is system id
            if id == 0 {
                continue;
            }

            let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, id)
                .map_err(|x| format!("error: {}", x))?;
            get_process_name(process).unwrap()
        };

        return_devices.push(MixerSession {
            name: name,
            session: session,
            simple_volume,
        })
    }
    Ok(return_devices)
}
