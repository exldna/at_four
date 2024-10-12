use voice::traits::{AudioDevice, Host};

fn main() -> voice::Result<()> {
    let host = voice::platform::Host::new()?;

    for device in host.audio_devices()? {
        println!("{:?}", device.readable_name());
    }

    Ok(())
}
