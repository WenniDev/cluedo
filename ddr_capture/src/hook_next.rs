use memory_rs::generate_aob_pattern;
use memory_rs::internal::process_info::ProcessInfo;
use retour::RawDetour;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use ddr_protocol::Message;

use crate::{socket, state};

type MusicSelectFn = unsafe extern "system" fn(usize) -> usize;

static ORIGINAL_FN: AtomicUsize = AtomicUsize::new(0);
static DETOUR: OnceLock<RawDetour> = OnceLock::new();

pub unsafe extern "system" fn hook(a0: usize) -> usize {
    log::info!("next entered");
    state::reset();
    socket::enqueue(Message::Next);

    let orig: MusicSelectFn = unsafe { std::mem::transmute(ORIGINAL_FN.load(Ordering::SeqCst)) };
    unsafe { orig(a0) }
}

pub fn install(process_info: &ProcessInfo) -> Result<(), Box<dyn std::error::Error>> {
    let target = {
        let memory_pattern = generate_aob_pattern![
            0x48, 0x8B, 0xC4, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57, 0x48, 0x8D,
            0xA8, 0x28, 0xFF, 0xFF, 0xFF, 0x48, 0x81, 0xEC, 0xB0, 0x01, 0x00, 0x00, 0x48, 0xC7,
            0x44, 0x24, 0x70, 0xFE, 0xFF, 0xFF, 0xFF, 0x48, 0x89, 0x58, 0x10, 0x48, 0x89, 0x70,
            0x18, 0x48, 0x89, 0x78, 0x20
        ];
        process_info
            .region
            .scan_aob(&memory_pattern)?
            .ok_or("hook_music_select: pattern not found")?
    } as *const ();

    unsafe {
        let detour = RawDetour::new(target, hook as *const ())?;
        ORIGINAL_FN.store(detour.trampoline() as *const () as usize, Ordering::SeqCst);
        detour.enable()?;
        DETOUR.set(detour).ok();
    }

    log::info!("hook_music_select enabled");
    Ok(())
}
