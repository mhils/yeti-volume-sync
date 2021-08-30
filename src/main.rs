#[macro_use]
extern crate anyhow;
extern crate hidapi;


use std::{thread, time};

use anyhow::Result;

mod win;
mod hid;

fn main() -> Result<()> {
    if std::env::args().any(|x| x == "--hide") {
        win::hide_window();
    } else {
        println!("Run with --hide to hide console window.");
    }
    loop {
        let err = sync_volumes().err().unwrap();
        println!("Error syncing Yeti ({}). Sleeping for 5s...", err);
        thread::sleep(time::Duration::from_secs(5));
    }
}

fn sync_volumes() -> Result<()> {
    let api = hidapi::HidApi::new()?;
    let yeti_hid = hid::get_yeti(&api)?;
    let yeti_snd = win::get_yeti()?;

    println!("Yeti found.");

    // We need to send this for the yeti to get into a proper state.
    // It's still somewhat flaky, but that's also the case with the Blue Sherpa software.
    hid::send(&yeti_hid, hid::Op::UnknownX05, "")?;
    hid::send(&yeti_hid, hid::Op::GetVolume, "")?;
    let vol_yeti = hid::read(&yeti_hid, hid::Recv::DeviceVolume, 5000)?
        .ok_or(anyhow!("cannot get yeti volume"))?;

    let vol_windows = win::get_volume(&yeti_snd)?;
    if vol_yeti < vol_windows {
        win::set_volume(&yeti_snd, vol_yeti)?;
        println!("Windows volume lowered from {} to {}.", vol_windows, vol_yeti);
    } else if vol_windows < vol_yeti {
        hid::send(&yeti_hid, hid::Op::SetVolume, &vol_windows.to_string())?;
        println!("Yeti volume lowered from {} to {}.", vol_yeti, vol_windows);
    } else {
        println!("Both volumes are at {}.", vol_yeti);
    }

    //let current_volume = win::get_volume(&yeti_snd)?.to_string();
    //hid::send(&yeti_hid, hid::Op::SetVolume, &current_volume)?;
    //hid::send(&yeti_hid, hid::Op::SetPattern, hid::pattern::CARDIOID)?;
    //hid::send(&yeti_hid, hid::Op::SetBlend, "50")?;
    //hid::send(&yeti_hid, hid::Op::SetGain, "25")?;
    //hid::send(&yeti_hid, hid::Op::Mute, "0")?;

    // Register for notifications for Windows -> Yeti updates.
    println!("Watching changes...");
    win::register_yeti_updater(&yeti_snd, hid::get_yeti(&api)?)?;

    // Read HID messages to perform Yeti -> Windows updates.
    loop {
        let new_yeti_volume = hid::read(&yeti_hid, hid::Recv::DeviceVolume, -1)?
            .ok_or(anyhow!("no data read"))?;
        println!("Adjusting Windows volume to {}.", new_yeti_volume);
        win::set_volume(&yeti_snd, new_yeti_volume)?;
    }
}
