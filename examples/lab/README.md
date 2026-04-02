# Orbit Camera Lab

Crate-local standalone lab app for validating the shared `orbit_camera` crate in a real Bevy application.

## Purpose

- verify mouse orbit, pan, and zoom in a deterministic scene
- keep a follow-target setup available for screenshot and BRP checks
- expose live focus, angles, zoom, and follow state through an overlay that E2E and BRP can inspect

## Status

Working

## Run

```bash
cargo run -p orbit_camera_lab
```

## E2E

```bash
cargo run -p orbit_camera_lab --features e2e -- orbit_camera_smoke
cargo run -p orbit_camera_lab --features e2e -- orbit_camera_input
cargo run -p orbit_camera_lab --features e2e -- orbit_camera_follow_target
cargo run -p orbit_camera_lab --features e2e -- snap_orbit_camera_ortho
```

## BRP

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch orbit_camera_lab
uv run --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name bevy_transform::components::transform::Transform
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/orbit_camera_lab.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```

Foreground fallback:

```bash
cargo run -p orbit_camera_lab
```

## Notes

- The lab keeps the controlled camera on a stable `OrbitCameraInputTarget` marker so E2E can exercise the real input path without guessing which camera should respond.
- Follow-target assertions look at the shared `OrbitCamera` component rather than the scene visuals alone, so failures can distinguish logic drift from lighting or presentation drift.
