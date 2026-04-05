# Architecture

`saddle-camera-orbit-camera` keeps the camera controller state on the camera entity itself and keeps the runtime split into three phases:

1. `ReadInput`
2. `ApplyIntent`
3. `SyncTransform`

That split makes the crate easy to order against gameplay, UI, and presentation systems without exposing internal helper resources as public API.

## State Model

`OrbitCamera` carries both the current smoothed state and the desired target state:

- current:
  `yaw`, `pitch`, `distance`, `orthographic_scale`, `focus`
- target:
  `target_yaw`, `target_pitch`, `target_distance`, `target_orthographic_scale`, `target_focus`
- reset view:
  `home`

The public component is the programmatic control seam. External systems should mutate the target fields or call helper methods on `OrbitCamera` rather than writing to `Transform`.

### Internal State

`OrbitCameraInternalState` is a required, `pub(crate)` component that carries runtime bookkeeping invisible to consumers:

- `idle_seconds` — time since the last manual interaction, used to trigger auto-rotate
- `manual_interaction_this_frame` — flag set by input systems and consumed by idle/inertia systems
- `orbit_velocity: Vec2` — current orbit momentum for inertia
- `pan_velocity: Vec3` — current pan momentum for inertia
- `zoom_velocity: f32` — current zoom momentum for inertia
- `collision_distance: Option<f32>` — override distance set by collision avoidance; `sync_transform` uses this to pull the camera forward when set

## Input Flow

Desktop input uses Bevy's shared mouse resources:

- `ButtonInput<MouseButton>` for orbit and pan button state
- `AccumulatedMouseMotion` for drag delta
- `AccumulatedMouseScroll` for wheel zoom

Touch input is handled from `TouchInput` messages. The runtime keeps a small internal tracker resource that converts active touches into per-frame gesture deltas:

- one touch:
  orbit delta
- two touches:
  pan delta plus pinch delta

Gamepad input reads from `Query<&Gamepad>`:

- right stick:
  orbit (yaw and pitch)
- left stick:
  pan on the camera plane
- triggers (`LeftZ` / `RightZ`):
  zoom in and out

A configurable deadzone filters out stick noise. Gamepad input is disabled by default and enabled via `settings.gamepad.enabled = true`.

Only cameras marked with `OrbitCameraInputTarget` consume this shared pointer input. If several active cameras carry the marker, the runtime picks the one with the highest `Camera.order`.

### Inertia Velocity Tracking

When `settings.inertia.enabled` is `true`, each input system stores the current frame's input delta into the internal state velocity fields (`orbit_velocity`, `pan_velocity`, `zoom_velocity`). When manual input stops, the `apply_inertia` system continues to apply these velocities with exponential friction decay, creating a smooth deceleration.

## System Ordering

### `ReadInput`

- `capture_touch_gestures`
- `apply_pointer_input`
- `apply_gamepad_input`

`apply_pointer_input` writes only to the selected camera's target state. Mouse orbit modifies `target_yaw` and `target_pitch`. Panning adjusts `target_focus`, or `OrbitCameraFollow.offset` when a follow component is active. Perspective zoom updates `target_distance`; orthographic zoom updates `target_orthographic_scale`.

When `OrbitCameraMouseControls::zoom_to_cursor` is enabled, perspective zoom first solves the point under the cursor on the current focus plane, applies the new target distance, then offsets the focus so that point remains visually stable. This keeps model-viewer zooms feeling closer to Blender-style "zoom toward the cursor" behavior without introducing scene-picking dependencies.

`apply_gamepad_input` runs after pointer input and adds gamepad deltas to the same target state fields. It uses time-based sensitivity (radians/sec, units/sec) unlike mouse input which uses per-pixel sensitivity.

Both `reversed_zoom` and `allow_upside_down` are handled at the input level — reversed_zoom flips the sign of zoom deltas, and upside-down mode auto-reverses horizontal orbit when the camera pitch crosses ±PI/2.

### `ApplyIntent`

- `tick_idle_timers`
- `sync_follow_targets`
- `apply_auto_rotate`
- `apply_inertia`
- `advance_state`
- `apply_dolly_zoom`
- `update_collision`

`sync_follow_targets` keeps `target_focus` attached to an entity when `OrbitCameraFollow` is present. It uses Bevy's `TransformHelper` so targets that already changed their local `Transform` earlier in the same `Update` frame are followed without waiting for the next transform propagation pass. The runtime preserves the camera's orbit angles and zoom; only the focus point is driven by the target.

`apply_auto_rotate` runs only after the configured idle delay and only when manual input did not fire in the current frame.

`apply_inertia` runs only when no manual input was detected in the current frame and inertia is enabled. It applies the stored velocity from the last frame of manual input and decays all three velocity channels (orbit, pan, zoom) using exponential friction: `velocity *= exp(-friction * dt)`. Velocities are zeroed when they fall below a small threshold to prevent floating-point drift.

`advance_state` clamps target pitch (unless `allow_upside_down` is set), optional target yaw, distance, orthographic scale, and focus bounds, then smooths the current state toward the target state using frame-rate-independent exponential interpolation. When `settings.force_update` is `true`, state advancement runs even while `settings.enabled` is `false`, then `force_update` auto-resets to `false`.

`apply_dolly_zoom` runs on entities that carry the `OrbitCameraDollyZoom` component with `enabled = true`. It computes the perspective FOV as `2 * atan(reference_width / (2 * distance))` and writes it directly to `PerspectiveProjection::fov`. The effect maintains a constant apparent size at the focus point while the orbit distance changes.

`update_collision` runs on entities with the `OrbitCameraCollision` component. When a `collision_distance` has been set on the internal state (by consumer-side physics code), this system smoothly relaxes the collision distance back toward the full orbit distance. When the collision distance reaches within 0.01 of the orbit distance, it resets to `None`. This is intentionally infrastructure-only — the actual collision detection (raycasts, shape casts) is left to the consumer.

### `SyncTransform`

`sync_transform` converts the smoothed orbit state into:

- `Transform.rotation`
- `Transform.translation`
- `OrthographicProjection::scale` when the active projection is orthographic

When a `collision_distance` override exists on the internal state, the effective distance used for translation is `min(orbit.distance, collision_distance)`. This pulls the camera forward to avoid occluding geometry without altering the orbit state itself.

This runs in `PostUpdate` before `TransformSystems::Propagate` so downstream systems read the final camera pose in the same frame.

## Smoothing Model

The runtime uses a target-state model rather than writing camera transforms directly from raw input. That has several benefits:

- smoothing stays frame-rate-independent
- programmatic camera motion and user motion share the same code path
- follow-target motion can blend with manual orbit and zoom
- tests can assert the public component state without depending on renderer timing

Rotation smoothing uses shortest-angle interpolation to avoid wraparound jumps across `-PI..PI`.

## Follow Mode

`OrbitCameraFollow` is intentionally small:

- `target`
- `offset`
- `enabled`

When follow is active:

- target motion drives `target_focus`
- manual orbit and zoom still work
- manual pan changes `offset` instead of breaking follow

If the tracked entity disappears, the runtime keeps the last focus point and simply stops updating it. There is no panic path or implicit despawn.

If you animate the tracked entity in `Update`, order that motion before `OrbitCameraSystems::ApplyIntent`. `TransformHelper` eliminates stale `GlobalTransform` reads, but it still can only observe transforms that were written earlier in the schedule.

## Dolly Zoom

`OrbitCameraDollyZoom` is an optional component that creates the Hitchcock/vertigo effect — as the camera moves closer to or further from the focus point, the field of view adjusts to keep a reference width at the focus point constant. The visual result is that the background appears to stretch or compress while the subject stays the same size.

The formula is `fov = 2 * atan(reference_width / (2 * distance))`. A larger `reference_width` produces a more dramatic effect. The FOV is clamped to `(0.01, PI - 0.01)` to prevent singularities.

## Camera Collision

`OrbitCameraCollision` provides the infrastructure for collision avoidance without coupling to a physics engine:

1. Consumer code performs raycasts from focus to camera position
2. Consumer sets `collision_distance` on the internal state
3. `sync_transform` uses `min(orbit.distance, collision_distance)` as the effective distance
4. `update_collision` smoothly relaxes the collision distance back to full orbit distance when no longer set

This design keeps the shared crate physics-agnostic while providing the wiring for any raycast or shape-cast implementation.

## Rotation Policy

This crate implements a stable turntable-style yaw and pitch controller around world-up. When `allow_upside_down` is enabled, pitch can exceed ±PI/2 and horizontal orbit auto-reverses to maintain intuitive mouse direction. It does not expose a trackball or free-up-axis mode yet.

That choice keeps the public surface small and predictable for the intended product classes:

- model viewers
- strategy cameras
- level editors
- build-mode cameras

If a future consumer needs trackball semantics, the clean extension point is the math layer plus an additional authored mode enum on `OrbitCameraSettings`.
