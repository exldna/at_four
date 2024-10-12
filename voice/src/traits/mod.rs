pub trait Host {
    type Device: AudioDevice;

    fn audio_devices(&self) -> crate::error::Result<&[Self::Device]>;
}

pub trait AudioDevice {
    fn readable_name(&self) -> crate::error::Result<&str>;
}

pub trait Stream {}
