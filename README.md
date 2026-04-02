# Saddle Camera Orbit Camera

Reusable orbit, pan, and zoom camera for 3D Bevy scenes.

The crate is built for generic inspection and gameplay-adjacent navigation: model viewers, tactics boards, level editors, build modes, debug viewers, and photo-mode style cameras. It owns the camera's orbit state and projection sync, but it stays deliberately Bevy-only and project-agnostic.

## What It Is For

- perspective and orthographic orbit cameras
- focus-point inspection and tracked-entity follow
- mouse orbit, pan, and wheel zoom
- touch orbit, two-finger pan, and pinch zoom
- smoothing, pitch and yaw limits, zoom limits, and idle auto-rotate
- multiple cameras in one world with explicit input opt-in

## What It Is Not For

- spring-arm collision avoidance
- shoulder offsets or combat aim rigs
- lock-on or target-selection gameplay logic
- hardcoded UI integration or editor gizmo policy

## Quick Start

```toml
[dependencies]
saddle-camera-orbit-camera = { git = "https://github.com/julien-blanchon/saddle-camera-orbit-camera" }
bevy = "0.18"
```

```rust,no_run
use bevy::prelude::*;
use saddle_camera_orbit_camera::{OrbitCamera, OrbitCameraInputTarget, OrbitCameraPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, OrbitCameraPlugin::default()))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PointLight {
            intensity: 1_800_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 10.0, 6.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.6, 1.6, 1.6))),
        MeshMaterial3d(materials.add(Color::srgb(0.82, 0.54, 0.24))),
        Transform::from_xyz(0.0, 0.8, 0.0),
    ));

    commands.spawn((
        Name::new("Viewer Camera"),
        OrbitCamera::looking_at(Vec3::ZERO, Vec3::new(-7.0, 5.0, 10.0)),
        OrbitCameraInputTarget,
    ));
}
```

The runtime manages the `Transform` and the projection zoom path from the `OrbitCamera` component. Consumers should drive the camera by mutating `OrbitCamera` target fields or its helper methods instead of mutating the `Transform` directly.

For examples and always-on tools, `OrbitCameraPlugin::always_on(Update)` is the simple constructor.

## Public API

| Type | Purpose |
| --- | --- |
| `OrbitCameraPlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `OrbitCameraSystems` | Public ordering hooks: `ReadInput`, `ApplyIntent`, `SyncTransform` |
| `OrbitCamera` | Main controller component containing current state, target state, and home view |
| `OrbitCameraSettings` | Input, limits, smoothing, and auto-rotate tuning surface |
| `OrbitCameraInputTarget` | Opt-in marker for which camera should consume shared pointer input |
| `OrbitCameraFollow` | Optional follow-target component that keeps focus attached to an entity |
| `OrbitAngleLimit`, `OrbitZoomLimits` | Reusable limit types for pitch, yaw, distance, and orthographic scale |
| `OrbitCameraHome` | Stored reset view for `reset_to_home()` |

## Input Model

- Mouse:
  Left drag orbits by default, middle drag pans, wheel zooms.
- Touch:
  One finger orbits, two fingers pan, pinch zooms.
- Multi-camera safety:
  Only entities marked with `OrbitCameraInputTarget` receive shared pointer input. If several active cameras have the marker, the highest `Camera.order` wins.
- UI gating:
  The crate does not inspect UI hover state. Consumers should temporarily remove `OrbitCameraInputTarget` or disable the camera when a UI layer should own the pointer.

## Programmatic Control

The runtime is intentionally easy to drive from gameplay or tooling code:

- mutate `target_focus`, `target_yaw`, `target_pitch`, `target_distance`, or `target_orthographic_scale`
- call `focus_on`, `reset_to_home`, `capture_home_from_current`, `frame_sphere`, or `frame_aabb`
- add `OrbitCameraFollow` to preserve orbit angles while tracking an entity

If your app uses `bevy_enhanced_input`, keep that layer in consumer code and write its actions into the public `OrbitCamera` component instead of coupling the shared crate to BEI directly.

## Supported Camera Modes

| Mode | Behavior |
| --- | --- |
| Perspective orbit | Orbit around a focus point, pan on the camera plane, zoom by changing orbit distance |
| Orthographic overview | Orbit with the same state model, zoom by changing `OrthographicProjection::scale` |
| Follow target | Keep focus attached to an entity while preserving the current yaw, pitch, and zoom |

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Baseline model-viewer style orbit camera with idle auto-rotate | `cargo run -p saddle-camera-orbit-camera-example-basic` |
| `orthographic` | Orthographic tactics-style board overview with bounded pitch | `cargo run -p saddle-camera-orbit-camera-example-orthographic` |
| `follow_target` | Moving tracked target while orbit and zoom stay interactive | `cargo run -p saddle-camera-orbit-camera-example-follow-target` |
| `fit_bounds` | Public framing helpers cycling between several authored bounds | `cargo run -p saddle-camera-orbit-camera-example-fit-bounds` |
| `touch_viewer` | Touch-first product-viewer layout using the same runtime | `cargo run -p saddle-camera-orbit-camera-example-touch-viewer` |

## Workspace Lab

The richer lab app lives inside the crate at `shared/camera/saddle-camera-orbit-camera/examples/lab`:

```bash
cargo run -p saddle-camera-orbit-camera-lab
```

With E2E enabled:

```bash
cargo run -p saddle-camera-orbit-camera-lab --features e2e -- orbit_camera_input
```

## System Ordering

- `OrbitCameraSystems::ReadInput`:
  Aggregate touch gestures and consume the shared mouse and touch input path.
- `OrbitCameraSystems::ApplyIntent`:
  Tick idle timers, resolve follow targets, apply auto-rotate, clamp target state, and smooth toward it.
- `OrbitCameraSystems::SyncTransform`:
  Write the final `Transform` and orthographic projection scale in `PostUpdate` before transform propagation.

Consumers can map `ReadInput` and `ApplyIntent` into their own update pipeline and order other systems around them. If another system animates an `OrbitCameraFollow` target in `Update`, place that system before `OrbitCameraSystems::ApplyIntent` to make the camera react in the same frame.

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
