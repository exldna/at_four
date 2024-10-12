use voice::traits::{AudioDevice, Host};

fn main() -> voice::Result<()> {
    let context = voice::platform::Context::new()?;
    let host = voice::platform::Host::new(&context);

    for (i, device) in host.audio_devices()?.iter().enumerate() {
        println!("{i}: {}", device.readable_name()?);
    }

    Ok(())
}
