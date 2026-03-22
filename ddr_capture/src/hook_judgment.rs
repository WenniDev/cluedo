use memory_rs::generate_aob_pattern;
use memory_rs::internal::process_info::ProcessInfo;
use retour::RawDetour;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use ddr_protocol::{JudgmentKind, Judment, Message};

use crate::{socket, state};

type JudgmentFn = unsafe extern "system" fn(usize, u32, usize, usize) -> usize;

static ORIGINAL_FN: AtomicUsize = AtomicUsize::new(0);
static DETOUR: OnceLock<RawDetour> = OnceLock::new();

pub unsafe extern "system" fn hook(a0: usize, raw_kind: u32, raw_event: usize, a3: usize) -> usize {
    if let Ok(kind) = JudgmentKind::try_from(raw_kind) {
        let timing = unsafe { *(raw_event as *const i32).byte_add(4) };

        let counter = match kind {
            JudgmentKind::Marvelous => &state::MARVELOUS,
            JudgmentKind::Perfect => &state::PERFECT,
            JudgmentKind::Great => &state::GREAT,
            JudgmentKind::Good => &state::GOOD,
            JudgmentKind::Miss => &state::MISS,
            JudgmentKind::Ok => &state::OK,
            JudgmentKind::Ng => &state::NG,
        };
        counter.fetch_add(1, Ordering::Relaxed);

        if !matches!(
            kind,
            JudgmentKind::Marvelous | JudgmentKind::Ok | JudgmentKind::Ng
        ) {
            if timing < 0 {
                state::FAST.fetch_add(1, Ordering::Relaxed);
            } else if timing > 0 {
                state::SLOW.fetch_add(1, Ordering::Relaxed);
            }
        }

        let judgment = Judment::new(kind, timing);
        socket::enqueue(Message::Judgment(judgment));
    }

    let orig: JudgmentFn = unsafe { std::mem::transmute(ORIGINAL_FN.load(Ordering::SeqCst)) };
    unsafe { orig(a0, raw_kind, raw_event, a3) }
}

pub fn install(process_info: &ProcessInfo) -> Result<(), Box<dyn std::error::Error>> {
    let target = {
        let memory_pattern = generate_aob_pattern![
            0x48, 0x8B, 0x05, _, _, _, _, 0x48, 0x33, 0xC4, 0x48, 0x89, 0x85, _, _, _, _, 0x48,
            0x8D, 0x05, _, _, _, _
        ];
        process_info
            .region
            .scan_aob(&memory_pattern)?
            .ok_or("hook_judgment: pattern not found")?
            - 0x27
    } as *const ();

    unsafe {
        let detour = RawDetour::new(target, hook as *const ())?;
        ORIGINAL_FN.store(detour.trampoline() as *const () as usize, Ordering::SeqCst);
        detour.enable()?;
        DETOUR.set(detour).ok();
    }

    log::info!("hook_judgment enabled");
    Ok(())
}
