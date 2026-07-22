//! pocket-character: the airi-parity character widget on the Pocket runtime.
//!
//! Windowed mode is the product: a transparent, undecorated, always-on-top
//! window (450×600 by default — airi's stage geometry; `--size WxH` to
//! taste) rendering the VRM character.
//! `--headless-shot` drives the same [`Game`] object without a window and
//! saves an RGBA screenshot — CI-friendly parity checks.

mod guest;
mod widget;

use std::path::PathBuf;

use anyhow::Result;
use pocket3d::app::{AppConfig, Game};
use pocket3d::gpu::{Gpu, OffscreenTarget};
use pocket3d::input::Input;
use pocket3d::renderer::Renderer;

use widget::{Widget, WidgetConfig};

const SIZE: (u32, u32) = (450, 600);
const TICK_HZ: f32 = 60.0;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str| -> Option<String> {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1).cloned())
    };
    let root = std::env::var("POCKET_CHARACTER_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));

    // --size WxH (default 450x600, airi's stage geometry). The camera fov is
    // vertical, so any size shows the same framing scaled; changing the aspect
    // ratio crops horizontally.
    let size: (u32, u32) = flag("--size")
        .and_then(|s| {
            let (w, h) = s.split_once('x')?;
            Some((w.parse().ok()?, h.parse().ok()?))
        })
        .unwrap_or(SIZE);

    let cfg = WidgetConfig {
        model_path: flag("--model")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("assets/AvatarSample_A.vrm")),
        vrma_path: flag("--vrma")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("assets/idle_loop.vrma")),
        bundle_path: flag("--bundle")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("dist/character.js")),
        size,
        frames: flag("--frames").and_then(|s| s.parse().ok()),
    };

    if let Some(out) = flag("--headless-shot") {
        let ticks: u32 = flag("--ticks").and_then(|s| s.parse().ok()).unwrap_or(60);
        return headless_shot(cfg, ticks, PathBuf::from(out));
    }
    if let Some(dir) = flag("--headless-seq") {
        let ticks: u32 = flag("--ticks").and_then(|s| s.parse().ok()).unwrap_or(300);
        let skip: u32 = flag("--skip").and_then(|s| s.parse().ok()).unwrap_or(0);
        return headless_seq(cfg, ticks, skip, PathBuf::from(dir));
    }

    let max_fps = flag("--max-fps").and_then(|s| s.parse().ok()).unwrap_or(60.0);
    let widget = Widget::new(cfg);
    pocket3d::app::run(
        AppConfig {
            title: "pocket-character".into(),
            size,
            tick_hz: TICK_HZ,
            capture_mouse: false,
            transparent: true,
            decorations: false,
            always_on_top: true,
            resizable: false,
            max_fps: Some(max_fps),
            drag_window: true,
        },
        widget,
    )
}

/// Like `headless_shot`, but renders EVERY tick after `skip` into
/// `dir/frame-%05d.png` — filmstrips and videos for docs come from this.
fn headless_seq(cfg: WidgetConfig, ticks: u32, skip: u32, dir: PathBuf) -> Result<()> {
    let size = cfg.size;
    let gpu = Gpu::new_headless()?;
    let mut renderer = Renderer::new(&gpu, pocket3d::gpu::OFFSCREEN_FORMAT)?;
    let mut widget = Widget::new(cfg);
    widget.init(&gpu, &mut renderer)?;
    std::fs::create_dir_all(&dir)?;

    let input = Input::default();
    let dt = 1.0 / TICK_HZ;
    let target = OffscreenTarget::new(&gpu, size.0, size.1);
    for i in 0..(skip + ticks) {
        widget.frame(dt, &input);
        widget.tick(dt, &input);
        if i < skip {
            continue;
        }
        let (scene, camera, hud) = widget.compose(0.0, i as f32 * dt, size);
        renderer.render(&gpu, &target.view, size, scene, camera, hud);
        target.save_png(&gpu, &dir.join(format!("frame-{:05}.png", i - skip)))?;
    }
    println!("wrote {} frames to {}", ticks, dir.display());
    Ok(())
}

/// Drive the widget for `ticks` fixed steps without a window, render one
/// frame offscreen, save it (alpha preserved — the transparent background
/// stays transparent in the PNG).
fn headless_shot(cfg: WidgetConfig, ticks: u32, out: PathBuf) -> Result<()> {
    let size = cfg.size;
    let gpu = Gpu::new_headless()?;
    let mut renderer = Renderer::new(&gpu, pocket3d::gpu::OFFSCREEN_FORMAT)?;
    let mut widget = Widget::new(cfg);
    widget.init(&gpu, &mut renderer)?;

    let input = Input::default();
    let dt = 1.0 / TICK_HZ;
    for _ in 0..ticks {
        widget.frame(dt, &input);
        widget.tick(dt, &input);
    }
    let (scene, camera, hud) = widget.compose(0.0, ticks as f32 * dt, size);
    let target = OffscreenTarget::new(&gpu, size.0, size.1);
    renderer.render(&gpu, &target.view, size, scene, camera, hud);
    target.save_png(&gpu, &out)?;
    println!("wrote {}", out.display());
    Ok(())
}
