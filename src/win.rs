use anyhow::Result;
use hidapi::HidDevice;
use windows::*;

use bindings::{
    Windows,
    Windows::Win32::Foundation::PWSTR,
    Windows::Win32::Media::Audio::CoreAudio::*,
    Windows::Win32::Storage::StructuredStorage::{PROPVARIANT, STGM_READ},
    Windows::Win32::System::Com::*,
    Windows::Win32::System::Console::GetConsoleWindow,
    Windows::Win32::System::PropertiesSystem::{IPropertyStore, PROPERTYKEY},
    Windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE},
};

use crate::hid;

pub mod bindings {
    windows::include_bindings!();
}

extern "C" {
    pub static PKEY_Device_FriendlyName: PROPERTYKEY;
}

const GUID: &str = "84a11b57-3c1c-4fa9-b9c4-d8fdacaf4a96";

pub fn get_volume(client: &IAudioEndpointVolume) -> Result<u8> {
    unsafe {
        Ok((client.GetMasterVolumeLevelScalar()? * 100.0).round() as u8)
    }
}

pub fn set_volume(client: &IAudioEndpointVolume, value: u8) -> Result<()> {
    let guid = windows::Guid::from(GUID);
    let val = value as f32 / 100.0;
    unsafe {
        client.SetMasterVolumeLevelScalar(val, &guid)?;
    }
    Ok(())
}

pub fn register_yeti_updater(client: &IAudioEndpointVolume, yeti_hid: HidDevice) -> Result<()> {
    let callback: IAudioEndpointVolumeCallback = VolumeUpdateCallback { yeti_hid }.into();
    unsafe { client.RegisterControlChangeNotify(callback)?; }
    Ok(())
}


pub fn get_yeti() -> Result<IAudioEndpointVolume> {
    unsafe {
        CoInitialize(std::ptr::null_mut())?;

        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER)?;
        let endpoints: IMMDeviceCollection = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;

        let count = endpoints.GetCount()?;
        //println!("Devices found: {}", count);

        for n in 0..count {
            let device: IMMDevice = endpoints.Item(n)?;

            let props: IPropertyStore = device.OpenPropertyStore(STGM_READ as u32)?;
            let name = {
                let p: PROPVARIANT = props.GetValue(&PKEY_Device_FriendlyName)?;
                read_to_string(p.Anonymous.Anonymous.Anonymous.pwszVal)
            };
            //println!("Device: {:?}", name);

            if name.contains("Yeti") {
                let client: IAudioEndpointVolume = {
                    let mut client = None;
                    device.Activate(&IAudioEndpointVolume::IID, CLSCTX_ALL.0, std::ptr::null_mut(), client.set_abi())?;
                    client.ok_or(anyhow!("cannot obtain volume info"))?
                };

                return Ok(client);
            }
        }
    };
    Err(anyhow!("No yeti found."))
}


#[implement(Windows::Win32::Media::Audio::CoreAudio::IAudioEndpointVolumeCallback)]
#[allow(non_snake_case)]
struct VolumeUpdateCallback {
    pub yeti_hid: HidDevice,
}

#[allow(non_snake_case)]
impl VolumeUpdateCallback {
    fn OnNotify(
        &self,
        pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA,
    ) -> ::windows::Result<()> {
        let guid = unsafe { (*pnotify).guidEventContext };
        let rawVolume = unsafe { (*pnotify).fMasterVolume };
        let volume = (rawVolume * 100.0).round() as u8;

        if guid != Guid::from(GUID) {
            println!("Adjusting Yeti volume to {}.", volume);
            hid::send(&self.yeti_hid, hid::Op::SetVolume, &volume.to_string()).ok();
        }
        Ok(())
    }
}

pub fn hide_window() {
    unsafe {
        let hwnd = GetConsoleWindow();
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}

// Copied from
// https://github.com/microsoft/windows-samples-rs/blob/fd0e3de758243b36ed5e012b5f6e6d06e6db0d7e/spellchecker/src/main.rs#L87-L101
unsafe fn read_to_string(ptr: PWSTR) -> String {
    let mut len = 0usize;
    let mut cursor = ptr;
    loop {
        let val = cursor.0.read();
        if val == 0 {
            break;
        }
        len += 1;
        cursor = PWSTR(cursor.0.add(1));
    }

    let slice = std::slice::from_raw_parts(ptr.0, len);
    String::from_utf16(slice).unwrap()
}
