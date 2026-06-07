//! Chainloading: hand control to the selected OS's EFI loader.
//!
//! We build a device path that points at the target `.efi` **on the same ESP
//! volume we were loaded from**, then `load_image` + `start_image`. Loading via
//! a proper device path (rather than a raw buffer) matters for Windows:
//! `bootmgfw.efi` inspects its own loaded-image device path to locate its BCD.

use alloc::vec::Vec;
use uefi::boot::{self, LoadImageSource};
use uefi::proto::device_path::build::{media::FilePath, DevicePathBuilder};
use uefi::proto::device_path::DevicePath;
use uefi::proto::loaded_image::LoadedImage;
use uefi::{CStr16, Status};

/// Loader paths on the ESP.
pub const STEAMOS_LOADER: &CStr16 = uefi::cstr16!("\\EFI\\steamos\\steamcl.efi");
pub const WINDOWS_LOADER: &CStr16 = uefi::cstr16!("\\EFI\\Microsoft\\Boot\\bootmgfw.efi");

/// Whether a given loader file currently exists on the ESP (so the UI can show
/// an actionable error tile instead of a black screen on a missing OS).
pub fn loader_present(path: &CStr16) -> bool {
    let img = boot::image_handle();
    let Ok(proto) = boot::get_image_file_system(img) else { return false };
    let mut fs: uefi::fs::FileSystem = proto.into();
    fs.metadata(uefi::fs::Path::new(path)).is_ok()
}

/// Load and start the loader at `file_path` on our boot volume. On success this
/// does not return (the new image takes over); on failure it returns an error.
pub fn boot_loader(file_path: &CStr16) -> uefi::Result<()> {
    let image = boot::image_handle();

    // Device path of the volume we were loaded from (the ESP partition).
    let loaded = boot::open_protocol_exclusive::<LoadedImage>(image)?;
    let device = loaded.device().ok_or(Status::NOT_FOUND)?;
    let dev_path = boot::open_protocol_exclusive::<DevicePath>(device)?;

    // Rebuild: all partition nodes (drop the trailing End node) + our FilePath.
    let mut buf = Vec::new();
    let mut builder = DevicePathBuilder::with_vec(&mut buf);
    for node in dev_path.node_iter() {
        builder = builder.push(&node).map_err(|_| Status::OUT_OF_RESOURCES)?;
    }
    builder = builder
        .push(&FilePath { path_name: file_path })
        .map_err(|_| Status::OUT_OF_RESOURCES)?;
    let full = builder.finalize().map_err(|_| Status::OUT_OF_RESOURCES)?;

    let handle = boot::load_image(
        image,
        LoadImageSource::FromDevicePath { device_path: full, boot_policy: Default::default() },
    )?;
    boot::start_image(handle)?;
    Ok(())
}
