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

Only cameras marked with `OrbitCameraInputTarget` consume this shared pointer input. If several active cameras carry the marker, the runtime picks the one with the highest `Camera.order`.

## System Ordering

### `ReadInput`

- `capture_touch_gestures`
- `apply_pointer_input`

`apply_pointer_input` writes only to the selected camera's target state. Mouse orbit modifies `target_yaw` and `target_pitch`. Panning adjusts `target_focus`, or `OrbitCameraFollow.offset` when a follow component is active. Perspective zoom updates `target_distance`; orthographic zoom updates `target_orthographic_scale`.

When `OrbitCameraMouseControls::zoom_to_cursor` is enabled, perspective zoom first solves the point under the cursor on the current focus plane, applies the new target distance, then offsets the focus so that point remains visually stable. This keeps model-viewer zooms feeling closer to Blender-style “zoom toward the cursor” behavior without introducing scene-picking dependencies.

### `ApplyIntent`

- `tick_idle_timers`
- `sync_follow_targets`
- `apply_auto_rotate`
- `advance_state`

`sync_follow_targets` keeps `target_focus` attached to an entity when `OrbitCameraFollow` is present. It uses Bevy's `TransformHelper` so targets that already changed their local `Transform` earlier in the same `Update` frame are followed without waiting for the next transform propagation pass. The runtime preserves the camera's orbit angles and zoom; only the focus point is driven by the target.

`apply_auto_rotate` runs only after the configured idle delay and only when manual input did not fire in the current frame.

`advance_state` clamps target pitch, optional target yaw, distance, and orthographic scale, then smooths the current state toward the target state using frame-rate-independent exponential interpolation.

### `SyncTransform`

`sync_transform` converts the smoothed orbit state into:

- `Transform.rotation`
- `Transform.translation`
- `OrthographicProjection::scale` when the active projection is orthographic

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

## Rotation Policy

This crate currently implements a stable turntable-style yaw and pitch controller around world-up. It does not expose a trackball or free-up-axis mode yet.

That choice keeps the public surface small and predictable for the intended product classes:

- model viewers
- strategy cameras
- level editors
- build-mode cameras

If a future consumer needs trackball semantics, the clean extension point is the math layer plus an additional authored mode enum on `OrbitCameraSettings`.
