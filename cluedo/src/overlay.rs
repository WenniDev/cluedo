use crate::state;

use hudhook::hooks::dx9::ImguiDx9Hooks;
use hudhook::{Hudhook, ImguiRenderLoop, imgui};
use std::sync::atomic::Ordering;

struct DdrOverlay {
    visible: bool,
}

impl DdrOverlay {
    fn new() -> Self {
        Self { visible: false }
    }
}

struct OverlayRow {
    label: &'static str,
    color: [f32; 4],
    count: u32,
}

impl ImguiRenderLoop for DdrOverlay {
    fn render(&mut self, ui: &mut imgui::Ui) {
        if ui.io().key_ctrl && ui.is_key_pressed(imgui::Key::J) {
            self.visible = !self.visible;
        }

        if !self.visible {
            return;
        }

        ui.window("DDR Overlay")
            .size([180.0, 180.0], imgui::Condition::FirstUseEver)
            .build(|| {
                let judgments: &[OverlayRow] = &[
                    OverlayRow {
                        label: "MARVELOUS",
                        color: [0.933, 0.910, 0.667, 1.0],
                        count: state::MARVELOUS.load(Ordering::Relaxed),
                    },
                    OverlayRow {
                        label: "PERFECT",
                        color: [1.0, 1.0, 0.0, 1.0],
                        count: state::PERFECT.load(Ordering::Relaxed),
                    },
                    OverlayRow {
                        label: "GREAT",
                        color: [0.0, 0.502, 0.0, 1.0],
                        count: state::GREAT.load(Ordering::Relaxed),
                    },
                    OverlayRow {
                        label: "GOOD",
                        color: [0.0, 0.502, 0.502, 1.0],
                        count: state::GOOD.load(Ordering::Relaxed),
                    },
                    OverlayRow {
                        label: "O.K.",
                        color: [0.8, 0.4, 0.0, 1.0],
                        count: state::OK.load(Ordering::Relaxed),
                    },
                    OverlayRow {
                        label: "MISS",
                        color: [0.545, 0.0, 0.0, 1.0],
                        count: state::MISS.load(Ordering::Relaxed)
                            + state::NG.load(Ordering::Relaxed),
                    },
                ];
                let timing: &[OverlayRow] = &[
                    OverlayRow {
                        label: "FAST",
                        color: [0.098, 0.380, 1.0, 1.0],
                        count: state::FAST.load(Ordering::Relaxed),
                    },
                    OverlayRow {
                        label: "SLOW",
                        color: [1.0, 0.447, 0.812, 1.0],
                        count: state::SLOW.load(Ordering::Relaxed),
                    },
                ];

                for judg in judgments {
                    ui.text_colored(judg.color, judg.label);
                    ui.same_line_with_spacing(120.0, 0.0);
                    ui.text(judg.count.to_string());
                }
                ui.separator();
                for time in timing {
                    ui.text_colored(time.color, time.label);
                    ui.same_line_with_spacing(120.0, 0.0);
                    ui.text(time.count.to_string());
                }
            });
    }
}

pub fn install() -> Result<(), anyhow::Error> {
    std::thread::spawn(|| {
        if let Err(e) = Hudhook::builder()
            .with::<ImguiDx9Hooks>(DdrOverlay::new())
            .build()
            .apply()
        {
            log::error!("hudhook failed: {e:?}");
        }
    });

    Ok(())
}
