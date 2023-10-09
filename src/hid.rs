use std::time::Instant;

use anyhow::Result;

pub fn get_yeti(api: &hidapi::HidApi) -> Result<hidapi::HidDevice> {
    for device in api.device_list() {
        if device.vendor_id() == 0x46d && (device.product_id() == 0xaaf || device.product_id() == 0xad1) {
            if device.usage() == 1 {
                let dev = device.open_device(&api)?;
                return Ok(dev);
            }
            // there are other usage ids. one is used by GHub, the other doesn't emit anything.
        }
    }
    return Err(anyhow!("Yeti not found."));
}

pub fn send(device: &hidapi::HidDevice, opcode: Op, val: &str) -> Result<()> {
    let mut buf = [0u8; 65];
    buf[0] = 0x01;
    buf[4] = opcode as u8;
    buf[8] = 8 + val.len() as u8;
    buf[9..9 + val.len()].copy_from_slice(val.as_bytes());
    //println!("Send hex: {:x?}", buf);
    let res = device.write(&buf)?;
    ensure!(res == 64, "did not write all bytes");
    Ok(())
}

pub fn read(device: &hidapi::HidDevice, op: Recv, timeout: i32) -> Result<Option<u8>> {
    let start = Instant::now();
    loop {
        let mut buf = [0u8; 0x40];

        let bytes_read = if timeout >= 0 {
            let elapsed = start.elapsed().as_millis();
            if elapsed >= timeout as u128 {
                return Ok(None);
            }
            device.read_timeout(&mut buf, timeout - elapsed as i32)?
        } else {
            device.read(&mut buf)?
        };

        if bytes_read == 0 {
            return Ok(None);
        }

        //println!("{:02x?}", buf);

        if &buf[..4] == &[0x01, 0x80, 0x00, 0x00] {
            match Recv::from_u8(buf[4]) {
                Ok(r) if r == op => {
                    let val = buf[9];
                    return Ok(Some(val));
                }
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
    }
}


#[allow(dead_code)]
pub enum Op {
    GetVolume = 0x01,
    UnknownX05 = 0x05,
    SetPattern = 0x08,
    SetBlend = 0x14,
    SetGain = 0x17,
    Mute = 0x20,
    SetVolume = 0x23,
}

#[derive(Debug, PartialEq)]
pub enum Recv {
    OldPattern,
    Pattern,
    SoftwareBlend,
    DeviceBlend,
    SoftwareGain,
    DeviceGain,
    SoftwareMute,
    DeviceMute,
    SoftwareVolume,
    DeviceVolume,
    UnknownX05,
}

impl Recv {
    pub fn from_u8(value: u8) -> Result<Recv> {
        Ok(match value {
            0x01 => Recv::DeviceVolume,
            0x05 => Recv::UnknownX05,
            0x08 => Recv::OldPattern,
            0x12 => Recv::Pattern,
            0x14 => Recv::SoftwareBlend,
            0x15 => Recv::DeviceBlend,
            0x17 => Recv::SoftwareGain,
            0x18 => Recv::DeviceGain,
            0x20 => Recv::SoftwareMute,
            0x21 => Recv::DeviceMute,
            0x23 => Recv::SoftwareVolume,
            0x24 => Recv::DeviceVolume,
            _ => return Err(anyhow!("Received unknown message type: {:x}", value)),
        })
    }
}

#[allow(dead_code)]
pub mod pattern {
    pub const STEREO: &str = "0";
    pub const OMNIDIRECTIONAL: &str = "1";
    pub const CARDIOID: &str = "2";
    pub const BIDIRECTIONAL: &str = "3";
}
