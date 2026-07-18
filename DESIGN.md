# pocket-character — design

A desktop character widget ("digital human") runtime on the Pocket runtime
family, built to answer a concrete question: what does the airi default
character cost when it runs on the Pocket architecture instead of Electron?

The reference point is [moeru-ai/airi](https://github.com/moeru-ai/airi)'s
desktop app: an Electron stage window that renders an animated character,
always-on-top and transparent, over the desktop.

## Parity target

airi's out-of-the-box model is **Live2D** (Momose Hiyori, a `.moc3` model).
Rendering `.moc3` requires the proprietary Live2D Cubism Core, which cannot be
vendored into an open runtime — so the parity target is airi's **VRM mode**,
its "3D digital human" stage, with airi's own defaults:

| Feature | airi VRM stage (default) | pocket-character |
|---|---|---|
| Model | AvatarSample_A.vrm (VRoid official sample) | same file, same source URL |
| Idle motion | `idle_loop.vrma` looped, hips re-anchored | same file, retargeted natively |
| Blink | sine, 0.2 s; random 1–6 s interval | same envelope + distribution |
| Eye saccades | fixation ±0.25, 0.8–4 s interval table | same table |
| Physics | VRM spring bones from model data | native verlet solver, same data |
| Cursor tracking | mode `none` by default (mouse mode exists) | same default, mouse mode supported |
| Lip sync | only while TTS plays (inert without AI) | out of scope (no AI stack) |
| Window | 450×600 transparent, frameless, always-on-top, drag-to-move | same |

## Runtime shape (RUNTIMES.md: ⟨Cores, Surfaces, Guest⟩)

- **Core** (`pocket-character-core`): owns the character's continuous state —
  animation clock, blink envelope, saccade fixation, look-at, spring-bone
  particles, drag state. Ticks at a fixed rate; never calls the guest.
- **Surface** (`character`): string-keyed namespace mounted next to `ui`
  (the open-strike `strike` pattern — no numeric op registry). Facts flow
  guest-ward as per-tick events; intent flows core-ward as queued commands.
- **Guest** (QuickJS, one bundle): behavior policy — which motion plays,
  expression changes, tracking mode, window intents. The airi-parity behavior
  is the base program; a different character personality is just a different
  bundle.

### Why the split matters here

airi runs *everything* in renderer JS: three.js scene graph, per-frame spring
bone math, expression lerps, rAF-driven — on top of Electron's process tree.
The Pocket answer keeps per-frame math (sample clip → procedural pose edits →
springs → palette → draw) in the native core, and JS decides *policy* at
widget cadence. The guest's per-frame work is a few property reads.

## What lives where

**pocketjs main repo (generic engine, PR'd upstream):**
- `pocket3d`: morph targets (sparse CPU deltas + per-instance overlay buffer,
  zero cost while weights are static), explicit-pose instances
  (`ModelInstance::pose`), per-instance alpha-test cutout, widget window mode
  (`AppConfig`: transparent / undecorated / always-on-top / `max_fps` pacing),
  transparent scene clear.
- `pocket-vrm` (new crate): VRM 0.x parsing (humanoid, blend-shape groups,
  spring config, MToon material info), VRMA retargeting, spring-bone verlet
  solver, eye look-at. Generic: any Pocket app that wants a VRM character
  uses this.

**this repo (product-specific):**
- `crates/pocket-character-core`: the character sim (blink/saccade schedulers
  with airi's exact distributions, motion state machine, look-at driver,
  spring integration order, damped drag).
- `crates/pocket-character`: the macOS widget host — winit window in widget
  mode, wgpu renderer via pocket3d, `character` surface mounting, cursor
  event wiring, perf instrumentation (RSS/CPU/frame-time counters).
- `app/`: the guest bundle (SDK + airi-parity behavior policy).
- `scripts/`: Bun TS only (repo law: no shell scripts) — asset fetch, build,
  measurement harness.

## Assets

Fetched at setup time, never committed (same posture as airi, which downloads
them at build time — the VRoid sample-model license and Live2D terms are not
MIT):

- `https://dist.ayaka.moe/vrm-models/VRoid-Hub/AvatarSample-A/AvatarSample_A.vrm`
- `idle_loop.vrma` from the airi repo (`packages/stage-ui-three/src/assets/vrm/animations/`)

## Performance posture

Everything the widget does at rest is event-shaped, so the design goal is
*idle should cost almost nothing*:

- `max_fps` pacing (60 for parity measurements; the loop sleeps, not spins).
- Morph uploads only on weight change (blinks are ~0.2 s out of every 1–6 s).
- Spring bones: preallocated, allocation-free steps.
- One process, one window, no cursor polling when tracking mode is `none`
  (airi's main process polls the global cursor at 60 Hz even when nothing
  consumes it).
- Measurement: same methodology as the airi baseline — ≥60 s steady-state
  sampling of RSS + %CPU over the full process tree, plus `footprint`.
