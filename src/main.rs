use voice::traits::{AudioDevice, Host};

fn main() -> voice::Result<()> {
    let context = voice::platform::Context::new()?;
    let host = voice::platform::Host::new(&context);

    for device in host.audio_devices()? {
        println!("{:?}", device.readable_name());
    }

    Ok(())
}
