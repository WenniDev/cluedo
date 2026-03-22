use crate::{hook_judgment, hook_next, overlay, socket};

use memory_rs::internal::process_info::ProcessInfo;

pub fn initialize() -> Result<(), anyhow::Error> {
    let process_info = ProcessInfo::new(Some("ddr-konaste.exe"))?;

    if let Err(e) = hook_judgment::install(&process_info) {
        log::error!("hook_judgment failed: {e:#}");
    }

    if let Err(e) = hook_next::install(&process_info) {
        log::error!("hook_scene failed: {e:#}");
    }

    if let Err(e) = socket::connect() {
        log::error!("socket failed: {e:#}");
    }

    if let Err(e) = overlay::install() {
        log::error!("overlay failed: {e:#}");
    }

    Ok(())
}
