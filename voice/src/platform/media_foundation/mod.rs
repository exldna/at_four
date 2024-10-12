mod context;
pub use context::Context;

use smallvec::SmallVec;
use std::cell::OnceCell;

use windows::core::PWSTR;
use windows::Win32::Media::MediaFoundation::*;

pub struct Host<'ctx> {
    context: Context,
    devices: Devices<'ctx>,
}

impl<'ctx> Host<'ctx> {
    pub fn new() -> crate::error::Result<Self> {
        Ok(Self {
            context: Context::new()?,
            devices: Devices::new(),
        })
    }
}

impl<'ctx> crate::traits::Host for Host<'ctx> {
    type Device = AudioDevice<'ctx>;

    fn audio_devices(&self) -> crate::error::Result<&[Self::Device]> {
        Ok(Devices::get(&self.devices, &self.context)?)
    }
}

struct Devices<'ctx> {
    capture: OnceCell<&'ctx [IMFActivate]>,
    devices: OnceCell<SmallVec<[AudioDevice<'ctx>; 10]>>,
}

impl<'ctx> Devices<'ctx> {
    fn new() -> Self {
        Self {
            capture: OnceCell::new(),
            devices: OnceCell::new(),
        }
    }

    fn get(&self, context: &'ctx Context) -> crate::error::Result<&[AudioDevice<'ctx>]> {
        let devices = self
            .devices
            .get_or_try_init(|| -> crate::error::Result<_> {
                let capture_devices = self
                    .capture
                    .get_or_try_init(|| self.get_capture_devices(context))?;
                Ok(SmallVec::from_iter(
                    capture_devices
                        .iter()
                        .map(|source| AudioDevice::from(source)),
                ))
            })?;
        Ok(devices.as_ref())
    }

    fn get_capture_devices(
        &self,
        context: &'ctx Context,
    ) -> crate::error::Result<&'ctx [IMFActivate]> {
        let attributes = context.create_attributes(1)?;
        unsafe {
            // SAFETY: Media Foundation MUST be initialized.
            attributes.SetGUID(
                &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_GUID,
            )?;
            // SAFETY: Attributes set to capture audio devices.
            Ok(context.enum_device_sources(&attributes)?)
        }
    }
}

pub struct AudioDevice<'ctx> {
    source: &'ctx IMFActivate,
    readable_name: OnceCell<String>,
}

impl<'ctx> AudioDevice<'ctx> {
    pub fn new(source: &'ctx IMFActivate) -> Self {
        Self {
            source,
            readable_name: OnceCell::new(),
        }
    }

    fn get_readable_name(&self) -> crate::error::Result<String> {
        let mut name: PWSTR = PWSTR::null();
        let mut name_len: u32 = 0;
        unsafe {
            self.source.GetAllocatedString(
                &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
                &mut name,
                &mut name_len,
            )?;
            Ok(name.to_string()?)
        }
    }
}

impl<'ctx> crate::traits::AudioDevice for AudioDevice<'ctx> {
    fn readable_name(&self) -> crate::error::Result<&str> {
        let readable_name = self
            .readable_name
            .get_or_try_init(|| self.get_readable_name())?;
        Ok(readable_name.as_str())
    }
}

impl<'ctx> From<&'ctx IMFActivate> for AudioDevice<'ctx> {
    fn from(source: &'ctx IMFActivate) -> Self {
        AudioDevice::new(source)
    }
}
