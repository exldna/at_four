use std::sync::{Arc, Mutex, Weak};

use windows::Win32::Media::MediaFoundation::*;

/// Ensures MediaFoundation initialization correctness.
#[derive(Clone)]
pub struct Context {
    _initialization: Arc<Initialization>,
}

impl Context {
    pub fn new() -> crate::error::Result<Self> {
        Ok(Self {
            _initialization: Initialization::global()?,
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

static MEDIA_FOUNDATION_INITIALIZATION: Mutex<Option<Weak<Initialization>>> = Mutex::new(None);

impl Initialization {
    fn global() -> crate::error::Result<Arc<Initialization>> {
        let mut guard = MEDIA_FOUNDATION_INITIALIZATION.lock().unwrap();
        let instance = match guard.as_mut() {
            None => Arc::new_cyclic(|weak| {
                *guard = Some(weak.clone());
                // TODO: Dont panic if MF Startup corrupted.
                unsafe { Initialization::new() }.unwrap()
            }),
            Some(init) => Weak::upgrade(init).unwrap(),
        };
        Ok(instance)
    }

    /// SAFETY: MUST be called exactly once before all references go out of scope.
    unsafe fn new() -> crate::error::Result<Self> {
        unsafe { MFStartup(MF_VERSION, MFSTARTUP_LITE) }?;
        Ok(Self {})
    }
}

impl Drop for Initialization {
    fn drop(&mut self) {
        let mut guard = MEDIA_FOUNDATION_INITIALIZATION.lock().unwrap();
        *guard = None;

        unsafe { MFShutdown() }.unwrap();
    }
}
