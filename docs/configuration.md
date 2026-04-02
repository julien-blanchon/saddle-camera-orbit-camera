# Configuration

`OrbitCameraSettings` is the main tuning surface for `saddle-camera-orbit-camera`. It is per-camera, not global, so several orbit cameras can coexist with different input and feel profiles.

## `OrbitCameraSettings`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `true` or `false` | Turns the runtime update path on or off for that camera | When `false`, the camera stops consuming input and stops smoothing toward target state |
| `mouse` | `OrbitCameraMouseControls` | see below | per field | Desktop orbit, pan, and wheel tuning | Applies only to cameras with `OrbitCameraInputTarget` |
| `touch` | `OrbitCameraTouchControls` | see below | per field | Touch orbit, pan, and pinch tuning | Touch input is ignored when `touch.enabled` is `false` |
| `inversion` | `OrbitAxisInversion` | all `false` | per field | Per-axis inversion for orbit, pan, and zoom | Shared between mouse and touch |
| `pitch_limits` | `OrbitAngleLimit` | `[-1.45, 1.45]` | any finite radians with `min <= max` | Clamp for `target_pitch` | Keep away from `+-PI/2` singularities unless you intentionally want near-pole views |
| `yaw_limits` | `Option<OrbitAngleLimit>` | `None` | `None` or any finite radians with `min <= max` | Optional clamp for `target_yaw` | `None` means full 360-degree orbit |
| `zoom_limits` | `OrbitZoomLimits` | see below | positive finite values | Clamp for perspective distance and orthographic scale | Perspective uses `distance`; orthographic uses `OrthographicProjection::scale` |
| `smoothing` | `OrbitCameraSmoothing` | see below | non-negative finite values | Decay rates for rotation, focus, and zoom smoothing | `0.0` means snap immediately to target |
| `auto_rotate` | `OrbitCameraAutoRotate` | disabled | see below | Idle auto-rotation config | Useful for product viewers and photo-mode idles |

## `OrbitCameraMouseControls`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `orbit_button` | `MouseButton` | `Left` | any mouse button | Button that turns mouse motion into orbit input | Consumers can rebind to `Right` or another button if selection owns left click |
| `pan_button` | `MouseButton` | `Middle` | any mouse button | Button that turns mouse motion into pan input | Pan modifies follow offset when follow mode is active |
| `orbit_sensitivity` | `Vec2` | `(0.008, 0.008)` | non-negative finite values | Orbit radians per mouse pixel on X and Y | Larger values feel faster |
| `pan_sensitivity` | `f32` | `1.0` | non-negative finite values | Multiplier for screen-space pan drag | World-space pan also scales with distance or orthographic area |
| `wheel_zoom_sensitivity` | `f32` | `0.14` | non-negative finite values | Multiplier for exponential wheel zoom | Perspective and orthographic both use multiplicative zoom feel |

## `OrbitCameraTouchControls`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `true` or `false` | Enables touch gesture handling | Touch is ignored when disabled |
| `orbit_sensitivity` | `Vec2` | `(0.01, 0.01)` | non-negative finite values | Orbit radians per touch pixel on X and Y | One finger drives orbit |
| `pan_sensitivity` | `f32` | `1.0` | non-negative finite values | Multiplier for two-finger pan drag | Uses the same world-space pan scaling rules as mouse panning |
| `pinch_zoom_sensitivity` | `f32` | `0.01` | non-negative finite values | Multiplier for pinch-distance delta | Positive pinch growth zooms in by default |

## `OrbitAxisInversion`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `orbit_x` | `bool` | `false` | `true` or `false` | Flips horizontal orbit |
| `orbit_y` | `bool` | `false` | `true` or `false` | Flips vertical orbit |
| `pan_x` | `bool` | `false` | `true` or `false` | Flips horizontal pan |
| `pan_y` | `bool` | `false` | `true` or `false` | Flips vertical pan |
| `zoom` | `bool` | `false` | `true` or `false` | Flips wheel and pinch zoom direction |

## `OrbitZoomLimits`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `min_distance` | `f32` | `0.5` | `> 0` | Lower clamp for perspective orbit radius | Keeps the camera from collapsing into the focus point |
| `max_distance` | `f32` | `250.0` | `>= min_distance` | Upper clamp for perspective orbit radius | Useful for strategy and overview cameras |
| `min_orthographic_scale` | `f32` | `0.05` | `> 0` | Lower clamp for `OrthographicProjection::scale` | Smaller values zoom in further |
| `max_orthographic_scale` | `f32` | `128.0` | `>= min_orthographic_scale` | Upper clamp for `OrthographicProjection::scale` | Larger values zoom out further |

## `OrbitCameraSmoothing`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `rotation_decay` | `f32` | `16.0` | `>= 0` | How quickly yaw and pitch converge to their target values |
| `focus_decay` | `f32` | `20.0` | `>= 0` | How quickly focus converges to `target_focus` |
| `zoom_decay` | `f32` | `18.0` | `>= 0` | How quickly distance or orthographic scale converges |

Higher decay means a snappier camera. `0.0` disables smoothing for that channel and snaps immediately.

## `OrbitCameraAutoRotate`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `enabled` | `bool` | `false` | `true` or `false` | Enables idle auto-rotation | Useful for model viewers and attract loops |
| `wait_seconds` | `f32` | `2.0` | `>= 0` | Idle time before the camera starts rotating | Resets whenever manual input fires |
| `speed` | `f32` | `0.45` | any finite radians per second | Yaw speed applied while idle | Positive values rotate clockwise around world-up |

## `OrbitCamera`

`OrbitCamera` itself is also part of the tuning and control surface:

| Field Group | Purpose |
| --- | --- |
| current state | `yaw`, `pitch`, `distance`, `orthographic_scale`, `focus` |
| target state | `target_yaw`, `target_pitch`, `target_distance`, `target_orthographic_scale`, `target_focus` |
| home view | `home` |

Useful helpers:

- `focus_on(point)`
- `reset_to_home()`
- `capture_home_from_current()`
- `snap_to_target()`
- `frame_sphere(projection, center, radius, padding)`
- `frame_aabb(projection, center, half_extents, padding)`

## Perspective vs Orthographic Notes

- Perspective zoom changes `distance`
- Orthographic zoom changes `OrthographicProjection::scale`
- Both use multiplicative zoom feel rather than naive linear deltas
- Panning always happens on the camera plane
- Perspective pan speed scales with distance and FOV
- Orthographic pan speed scales with the resolved projection area
