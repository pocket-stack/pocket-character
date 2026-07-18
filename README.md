# pocket-character

A 3D digital-human desktop widget on the Pocket runtime family — the
[airi](https://github.com/moeru-ai/airi) VRM stage, reimplemented as **one
native process**: a transparent, always-on-top, frameless window rendering a
VRM character with idle animation, auto-blink, eye saccades and spring-bone
physics, driven by a QuickJS policy bundle.

Built to answer a measured question: *what does the same character widget
cost on the Pocket architecture instead of Electron?* See
[DESIGN.md](DESIGN.md) for the architecture and the parity contract, and the
measurement section below for the answer.

## What it does

- **AvatarSample_A** (VRoid official sample) with airi's `idle_loop.vrma`
  looped natively — retargeted at load, not baked.
- **Auto-blink** (sine 0.2 s envelope, 1–6 s uniform interval) and **idle eye
  saccades** (airi's interval distribution table), both from a deterministic
  seeded sim in `pocket-character-core`.
- **Spring bones** (hair / hood / bust) from the model's VRM data, solved by
  `pocket-vrm`'s verlet solver each tick.
- **Widget window**: 450×600 (airi's stage geometry), transparent,
  undecorated, always-on-top, drag anywhere to move, frame-paced at 60 fps
  (the loop sleeps; `--max-fps` to taste).
- **Guest policy bundle** (`app/main.ts` → QuickJS): the `character` surface
  delivers per-tick facts (`blink`, `hovered`, `fps`, events) and accepts
  intent ops (`setTracking`, `setExpression`, `playClip`, `quit`). The
  airi-parity personality is deliberately near-empty policy; a different
  character is a different bundle, no rebuild of the host.

## Layout

| Path | What |
|---|---|
| `crates/pocket-character` | macOS widget host (winit + wgpu via `pocket3d`) |
| `crates/pocket-character-core` | portable behavior sim (blink/saccade/look-at) |
| `app/` | guest bundle: `character` surface SDK + policy |
| `scripts/` | Bun TS: asset fetch, bundle build, run, measurement |
| `vendor/pocketjs` | the engine, pinned as a submodule |

The generic halves live in the PocketJS main repo:
`pocket3d` (morph targets, pose injection, widget windows) and `pocket-vrm`
(VRM 0.x parsing, spring bones, VRMA retargeting) — see
[pocket-stack/pocketjs#125](https://github.com/pocket-stack/pocketjs/pull/125).

## Run

```sh
bun install          # nothing to install, but sets up the workspace
bun run setup        # submodule + vendored bun install + assets (not committed)
bun run widget       # build guest bundle + release binary, launch the widget
```

Verification without a window (renders the same Game object offscreen,
alpha preserved):

```sh
target/release/pocket-character --headless-shot shot.png --ticks 90
```

Measurement (same methodology as the airi baseline — ≥60 s of `ps` samples
over the process tree + `footprint`):

```sh
bun scripts/measure.ts
```

## Model & animation assets

Fetched at setup, never committed: the VRoid sample-model terms and the
animation's provenance are not MIT. Same posture as airi, which downloads
them at build time.

## Measurements

See [REPORT.md](REPORT.md) for the full comparison against airi on the same
machine, same methodology.
