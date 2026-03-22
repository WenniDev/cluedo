mod ddr_capture;
mod hook_judgment;
mod hook_next;
mod overlay;
mod socket;
mod state;

use std::ffi::c_void;
use windows::Win32::Foundation::{BOOL, HINSTANCE, TRUE};
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

fn init_logger() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp(None)
        .init();
}

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    reserved: *const c_void,
) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            init_logger();

            if let Err(err) = ddr_capture::initialize() {
                log::error!("Failed to initialize ddrgp_overlay: {err:#}");
            }
        }
        DLL_PROCESS_DETACH => {}
    }

    TRUE
}
