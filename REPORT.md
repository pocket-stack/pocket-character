# Measured: the airi character widget on the Pocket runtime

**Question.** [airi](https://github.com/moeru-ai/airi)'s desktop companion —
an animated character in a transparent always-on-top window — costs an
Electron process tree (user-visible: "two ~600 MB processes and a full CPU
core"). What does the *same* character, with the same out-of-the-box visual
and motion feature set, cost as a Pocket runtime?

**Answer.** One process, ~118 MB RSS, ~3.9 % of one core at 60 fps.
Same machine, same methodology, same character, same idle behaviors.

## Setup

- Machine: Apple M3 Max, 128 GB RAM, macOS 26.5.2.
- airi: v0.11.0 arm64 release build (Electron 41), fresh first-run state.
- pocket-character: release build, engine pinned at
  [pocket-stack/pocketjs#125](https://github.com/pocket-stack/pocketjs/pull/125).
- Methodology: launch → settle → ≥60 s of `ps` samples at 5 s intervals over
  the **full process tree** (RSS + %CPU, median reported), plus macOS
  `footprint` (phys_footprint counts GPU/IOSurface memory that RSS misses).

**How to read the CPU column.** Every %CPU in this report is the standard
per-process convention used by `ps`, `top` and Activity Monitor's process
list: **percent of one core** (100 % = one core saturated; a 16-core M3 Max
totals 1600 %). Both sides of the comparison are measured in the same unit,
so the ratios hold. Activity Monitor's bottom System/User/Idle summary is a
different quantity — whole-machine utilization across *all* processes — and
cannot be attributed to any one app. When eyeballing Activity Monitor,
judge only after ~1 min idle and hands off: launch spends ~2 CPU-seconds
decoding the model, and clicking/dragging the widget adds transient work,
both of which inflate short averaging windows. Compositor cost
(WindowServer) is excluded on both sides of the comparison.

Cross-check with interval sampling (`top -l`, the same scheme Activity
Monitor uses, 12×5 s hands-off): pocket-character reads **2.4–2.8 % at
30 fps and 4.4–4.8 % at 60 fps** — slightly above the `ps` decaying-average
medians in the table (2.1 % / 3.9 %). Quote the interval-sampled numbers
when comparing against what Activity Monitor shows.

airi's literal default character is Live2D (Momose Hiyori, proprietary
Cubism runtime); its 3D digital-human mode is VRM. The parity target is the
VRM stage — same model (VRoid AvatarSample_A), same idle animation
(`idle_loop.vrma`), auto-blink (0.2 s sine, 1–6 s), idle eye saccades
(airi's interval table), spring-bone physics, tracking mode `none`,
transparent frameless always-on-top window. Both airi modes were measured
for context.

## Results

### Steady idle, full process tree

| Runtime | Processes | RSS (median) | CPU (median, one core) | footprint |
|---|---|---|---|---|
| airi — Live2D default | 8 | 1742 MB | 90.7 % | ~1508 MB |
| airi — VRM mode (parity reference) | 8 | 2184 MB | 44.4 % | ~1870 MB |
| **pocket-character, 60 fps** | **1** | **118 MB** | **3.9 %** | **518 MB** |
| **pocket-character, 30 fps** | **1** | **117 MB** | **2.1 %** | **518 MB** |

Notes:
- airi numbers include its always-on hidden BeatSync renderer window and the
  first-run onboarding window (~264 MB RSS, ~0.6 % CPU); subtracting
  onboarding still leaves ~1.9 GB RSS / ~43.8 % CPU for the VRM stage.
- airi VRM cost centers: stage renderer ~900 MB RSS at ~19 % CPU, GPU helper
  ~421 MB RSS at ~23 % CPU (927 MB footprint), main process polling the
  global cursor at 60 Hz regardless of tracking mode.
- pocket-character cost centers: everything in one process; the guest
  (QuickJS policy bundle) ticks in ~0.03 ms; blink morphs upload only on
  weight change; the render loop is frame-paced (sleeps, never spins).

### Ratios (vs airi VRM mode, the apples-to-apples reference)

- Processes: **8 → 1**
- RSS: **2184 MB → 118 MB (~18×)**
- CPU at idle: **44.4 % → 3.9 % (~11×)** — at 30 fps, **~21×**
- phys_footprint: **~1870 MB → 518 MB (~3.6×)**
- Disk: airi 1.8 GB installed (822 MB DMG) → pocket-character binary ~5 MB +
  27 MB model assets

Against the default Live2D stage the user actually sees on first launch
(90.7 % of a core), the CPU gap at 60 fps is ~23×.

### Where the remaining memory goes

Of pocket-character's 518 MB footprint, CPU heap is ~16 MB dirty; the rest
is process-owned graphics memory (Metal textures + wgpu pools). The single
biggest lever was capping the model's authoring-resolution textures (four
4096² maps → 2048², invisible at 450×600): footprint 931 MB → 518 MB. The
morph system stores sparse CPU deltas, so facial animation adds no
steady-state cost.

## Feature parity checklist

| Feature | airi VRM stage | pocket-character |
|---|---|---|
| Model | AvatarSample_A.vrm | same file ✓ |
| Idle motion | idle_loop.vrma looped, hips re-anchored | same file, retargeted at load ✓ |
| Blink | sine 0.2 s, 1–6 s uniform | same envelope + distribution ✓ |
| Eye saccades | ±0.25 fixation, 0.8–4.8 s table | same table ✓ |
| Spring bones | model data (10 groups, 22 colliders) | same data, verlet solver ✓ |
| Cursor tracking | `none` by default, mouse mode exists | same default; mouse mode wired ✓ |
| Window | transparent, frameless, always-on-top, drag | ✓ (450×600, airi's default geometry) |
| Lip sync | inert without an AI/TTS stack | out of scope (no AI stack) |
| UI chrome | controls island overlay | not reproduced (policy-layer concern) |

Rendering differences, honestly stated: airi tone-maps (ACES) with an HDR
environment + post-processing; pocket-character renders MToon-approx (near
unlit + cutout). The difference is a subtle grade, not a different look:

| airi VRM stage | pocket-character idle | pocket-character mid-blink |
|---|---|---|
| ![airi](docs/airi-vrm-stage.png) | ![idle](docs/pocket-character-idle.png) | ![blink](docs/pocket-character-blink.png) |

(pocket-character stills are `--headless-shot` output — the PNG alpha is the
actual window transparency.)

## Conclusion

The scenario airi spends an Electron on is, structurally, a *fixed-function
character player*: one skinned mesh, one animation clip, a handful of
schedulers, a physics chain, a transparent window. On the Pocket
architecture (native core owns the per-frame math; QuickJS guest owns
policy; the surface between them is a spec'd vocabulary), that player fits
in one process at single-digit CPU and ~120 MB RSS — an order of magnitude
on every axis, with behavior-level parity and the personality still fully
scriptable in JS.

The generic pieces this produced (morph targets, pose injection, widget
windows, `pocket-vrm`) are upstreamed in
[pocketjs#125](https://github.com/pocket-stack/pocketjs/pull/125); nothing
in this repo is airi-specific except ~200 lines of behavior constants and
the policy bundle.
