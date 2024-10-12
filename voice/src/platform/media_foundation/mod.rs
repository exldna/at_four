mod context;
pub use context::Context;

use smallvec::SmallVec;
use std::cell::OnceCell;

use windows::core::PWSTR;
use windows::Win32::Media::MediaFoundation::*;

pub struct Host<'ctx> {
    context: &'ctx Context,
    devices: Devices<'ctx>,
}

impl<'ctx> Host<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            devices: Devices::new(),
        }
    }
}

impl<'ctx> crate::traits::Host for Host<'ctx> {
    type Device = AudioDevice<'ctx>;

    fn audio_devices(&self) -> crate::error::Result<&[Self::Device]> {
        Ok(&self.devices.get(&self.context)?)
    }
}

type CaptureDevices<'ctx> = SmallVec<[AudioDevice<'ctx>; 10]>;

struct Devices<'ctx> {
    devices: OnceCell<CaptureDevices<'ctx>>,
}

impl<'ctx> Devices<'ctx> {
    fn new() -> Self {
        Self {
            devices: OnceCell::new(),
        }
    }

    fn get(&self, context: &'ctx Context) -> crate::error::Result<&[AudioDevice<'ctx>]> {
        #[inline(always)]
        fn get_or_try_init_devices<'a, 'ctx>(
            context: &'ctx Context,
            devices: &'a OnceCell<CaptureDevices<'ctx>>,
        ) -> crate::error::Result<&'a CaptureDevices<'ctx>> {
            devices.get_or_try_init(|| {
                let capture = Devices::get_capture_devices(context)?;
                Ok(SmallVec::from_iter(
                    capture.iter().map(|source| AudioDevice::from(source)),
                ))
            })
        }

        let devices = get_or_try_init_devices(context, &self.devices)?;
        Ok(devices.as_ref())
    }

    fn get_capture_devices(context: &'ctx Context) -> crate::error::Result<&'ctx [IMFActivate]> {
        unsafe {
            let attributes = context.create_attributes(1)?;
            // SAFETY: Media Foundation MUST be initialized (Guaranteed by context).
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

    // SAFETY: Context MUST be initialized.
    unsafe fn get_readable_name(&self) -> crate::error::Result<String> {
        let mut name: PWSTR = PWSTR::null();
        let mut name_len: u32 = 0;
        // SAFETY: Media Foundation MUST be initialized.
        self.source.GetAllocatedString(
            &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
            &mut name,
            &mut name_len,
        )?;
        // SAFETY: We got this string from API and copy it into safe place...
        // What could go wrong?
        Ok(name.to_string()?)
    }
}

impl<'ctx> crate::traits::AudioDevice for AudioDevice<'ctx> {
    fn readable_name(&self) -> crate::error::Result<&str> {
        let readable_name = self
            .readable_name
            .get_or_try_init(|| unsafe { self.get_readable_name() })?;
        Ok(readable_name.as_str())
    }
}

impl<'ctx> From<&'ctx IMFActivate> for AudioDevice<'ctx> {
    fn from(source: &'ctx IMFActivate) -> Self {
        AudioDevice::new(source)
    }
}
