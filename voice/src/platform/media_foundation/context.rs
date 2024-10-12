use std::sync::atomic::{AtomicU16, Ordering};

use windows::Win32::Media::MediaFoundation::*;

/// Ensures MediaFoundation initialization correctness.
pub struct Context {
    _initialization: Initialization,
}

impl Context {
    pub fn new() -> crate::error::Result<Self> {
        Ok(Self {
            _initialization: Initialization::new()?,
        })
    }

    pub fn create_attributes(&self, initial_size: u32) -> windows::core::Result<IMFAttributes> {
        let mut attrs: Option<IMFAttributes> = None;
        unsafe {
            // SAFETY: Media Foundation MUST be initialized (Guaranteed by self).
            MFCreateAttributes(&mut attrs, initial_size)?;
        }
        Ok(attrs.unwrap())
    }

    /// SAFETY: You must ensure set attributes correctness.
    pub unsafe fn enum_device_sources(
        &self,
        attributes: &IMFAttributes,
    ) -> windows::core::Result<&[IMFActivate]> {
        let mut devices_ptr = std::ptr::null_mut();
        let mut count = 0;
        // SAFETY: Media Foundation MUST be initialized (Guaranteed by self).
        MFEnumDeviceSources(attributes, &mut devices_ptr, &mut count)?;
        let device_sources;
        // SAFETY: We assume that API call succeeded,
        // that means `devices_ptr` and `count` describes valid c-array.
        device_sources =
            std::slice::from_raw_parts(std::mem::transmute(devices_ptr), count as usize);
        Ok(device_sources)
    }
}

// Actually tracks Media Foundation initialization.
struct Initialization {}

static MEDIA_FOUNDATION_INITIALIZED: AtomicU16 = AtomicU16::new(0);

impl Initialization {
    fn new() -> windows::core::Result<Self> {
        if MEDIA_FOUNDATION_INITIALIZED.fetch_add(1, Ordering::SeqCst) == 0 {
            unsafe { MFStartup(MF_VERSION, MFSTARTUP_LITE) }?;
        }
        Ok(Self {})
    }
}

impl Drop for Initialization {
    fn drop(&mut self) {
        if MEDIA_FOUNDATION_INITIALIZED.fetch_sub(1, Ordering::SeqCst) == 1 {
            unsafe { MFShutdown() }.unwrap();
        }
    }
}
