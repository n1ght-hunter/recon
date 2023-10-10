use std::ptr;

use windows::Win32::{Media::Audio::{
    Endpoints::IAudioEndpointVolume, IAudioSessionManager2, IMMDevice,
}, System::Com::{StructuredStorage::PROPVARIANT, CLSCTX_ALL}};

use super::helper_functions::get_hardware_device_name;

#[derive(Debug, Clone, PartialEq)]
pub struct InputHardwareDevice {
    device: IMMDevice,
    audio_endpoint_volume: IAudioEndpointVolume,
}

impl InputHardwareDevice {
    pub unsafe fn new(device: IMMDevice) -> Self {
        Self {
            audio_endpoint_volume: device
                .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
                .expect("unable to get IAudioEndpointVolume"),
            device,
        }
    }

    pub unsafe fn get_range_db(self) -> Result<(f32, f32, f32), ()> {
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

    pub fn get_name(&self) -> Result<String, ()> {
        get_hardware_device_name(&self.device)
    }

    pub fn get_volume(&self) -> Result<f32, String> {
        unsafe {
            self.audio_endpoint_volume
                .GetMasterVolumeLevelScalar()
                .map_err(|e| format!("{:?}", e))
        }
    }

    /// volume between 0 and 1
    ///
    /// 0 for min and 1 for max
    pub unsafe fn set_volume_scaler(self, volume: f32) -> Result<(), String> {
        self.audio_endpoint_volume
            .SetMasterVolumeLevelScalar(volume, ptr::null())
            .map_err(|e| format!("{:?}", e))
    }
    /// set volume to decibels
    ///
    /// use get db range to see range of dbs
    pub unsafe fn set_volume_db(self, volume: f32) -> Result<(), String> {
        self.audio_endpoint_volume
            .SetMasterVolumeLevel(volume, ptr::null())
            .map_err(|e| format!("{:?}", e))
    }
}
