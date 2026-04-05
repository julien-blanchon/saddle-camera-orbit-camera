# Configuration

`OrbitCameraSettings` is the main tuning surface for `saddle-camera-orbit-camera`. It is per-camera, not global, so several orbit cameras can coexist with different input and feel profiles.

## `OrbitCameraSettings`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `true` or `false` | Turns the runtime update path on or off for that camera | When `false`, the camera stops consuming input and stops smoothing toward target state |
| `mouse` | `OrbitCameraMouseControls` | see below | per field | Desktop orbit, pan, and wheel tuning | Applies only to cameras with `OrbitCameraInputTarget` |
| `touch` | `OrbitCameraTouchControls` | see below | per field | Touch orbit, pan, and pinch tuning | Touch input is ignored when `touch.enabled` is `false` |
| `gamepad` | `OrbitCameraGamepadControls` | see below | per field | Gamepad orbit, pan, and zoom tuning | Gamepad input is ignored when `gamepad.enabled` is `false` |
| `inversion` | `OrbitAxisInversion` | all `false` | per field | Per-axis inversion for orbit, pan, and zoom | Shared between mouse, touch, and gamepad |
| `pitch_limits` | `OrbitAngleLimit` | `[-1.45, 1.45]` | any finite radians with `min <= max` | Clamp for `target_pitch` | Keep away from `+-PI/2` singularities unless you intentionally want near-pole views |
| `yaw_limits` | `Option<OrbitAngleLimit>` | `None` | `None` or any finite radians with `min <= max` | Optional clamp for `target_yaw` | `None` means full 360-degree orbit |
| `zoom_limits` | `OrbitZoomLimits` | see below | positive finite values | Clamp for perspective distance and orthographic scale | Perspective uses `distance`; orthographic uses `OrthographicProjection::scale` |
| `smoothing` | `OrbitCameraSmoothing` | see below | non-negative finite values | Decay rates for rotation, focus, and zoom smoothing | `0.0` means snap immediately to target |
| `auto_rotate` | `OrbitCameraAutoRotate` | disabled | see below | Idle auto-rotation config | Useful for product viewers and photo-mode idles |
| `inertia` | `OrbitCameraInertia` | disabled | see below | Friction settings for orbit, pan, and zoom momentum | Flick to spin, release to decelerate |
| `focus_bounds` | `Option<OrbitCameraFocusBounds>` | `None` | see below | Restricts `target_focus` within a Sphere or Cuboid | Prevents panning outside the authored play area |
| `allow_upside_down` | `bool` | `false` | `true` or `false` | Allows pitch to exceed ±PI/2, enabling upside-down orbit | Horizontal orbit auto-reverses when upside-down |
| `reversed_zoom` | `bool` | `false` | `true` or `false` | Inverts the zoom direction for wheel and pinch | Useful for trackball-style navigation preferences |
| `force_update` | `bool` | `false` | `true` or `false` | Forces one frame of state advancement even while `enabled` is `false` | Auto-resets to `false` after one frame |

## `OrbitCameraMouseControls`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `orbit_button` | `MouseButton` | `Left` | any mouse button | Button that turns mouse motion into orbit input | Consumers can rebind to `Right` or another button if selection owns left click |
| `pan_button` | `MouseButton` | `Middle` | any mouse button | Button that turns mouse motion into pan input | Pan modifies follow offset when follow mode is active |
| `orbit_sensitivity` | `Vec2` | `(0.008, 0.008)` | non-negative finite values | Orbit radians per mouse pixel on X and Y | Larger values feel faster |
| `pan_sensitivity` | `f32` | `1.0` | non-negative finite values | Multiplier for screen-space pan drag | World-space pan also scales with distance or orthographic area |
| `wheel_zoom_sensitivity` | `f32` | `0.14` | non-negative finite values | Multiplier for exponential wheel zoom | Perspective and orthographic both use multiplicative zoom feel |
| `zoom_to_cursor` | `bool` | `false` | `true` or `false` | Keeps the point under the cursor stable while perspective zoom changes distance | Uses a focus-plane solve, not scene picking |

## `OrbitCameraTouchControls`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `true` or `false` | Enables touch gesture handling | Touch is ignored when disabled |
| `orbit_sensitivity` | `Vec2` | `(0.01, 0.01)` | non-negative finite values | Orbit radians per touch pixel on X and Y | One finger drives orbit |
| `pan_sensitivity` | `f32` | `1.0` | non-negative finite values | Multiplier for two-finger pan drag | Uses the same world-space pan scaling rules as mouse panning |
| `pinch_zoom_sensitivity` | `f32` | `0.01` | non-negative finite values | Multiplier for pinch-distance delta | Positive pinch growth zooms in by default |

## `OrbitCameraGamepadControls`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `enabled` | `bool` | `false` | `true` or `false` | Enables gamepad input handling | Set to `true` to enable gamepad orbit, pan, and zoom |
| `orbit_sensitivity` | `Vec2` | `(2.5, 2.5)` | non-negative finite values | Orbit speed in radians/sec per stick axis | Right-stick drives orbit |
| `zoom_sensitivity` | `f32` | `3.0` | non-negative finite values | Zoom speed per trigger axis value | Right trigger zooms in, left trigger zooms out |
| `pan_sensitivity` | `f32` | `8.0` | non-negative finite values | Pan speed in world-space units/sec per stick axis | Left-stick drives pan |
| `deadzone` | `f32` | `0.15` | `0.0..1.0` | Stick deadzone threshold | Values below this are ignored |

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

## `OrbitCameraInertia`

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `enabled` | `bool` | `false` | `true` or `false` | Enables inertia/momentum for orbit, pan, and zoom | Camera continues moving after input stops |
| `orbit_friction` | `f32` | `5.0` | `> 0` | Exponential decay rate for orbit velocity | Higher values stop faster |
| `pan_friction` | `f32` | `6.0` | `> 0` | Exponential decay rate for pan velocity | Higher values stop faster |
| `zoom_friction` | `f32` | `8.0` | `> 0` | Exponential decay rate for zoom velocity | Higher values stop faster |

## `OrbitCameraFocusBounds`

An enum with two variants:

| Variant | Fields | Effect |
| --- | --- | --- |
| `Sphere` | `center: Vec3`, `radius: f32` | Restricts `target_focus` within a sphere |
| `Cuboid` | `min: Vec3`, `max: Vec3` | Restricts `target_focus` within an axis-aligned box |

Set via `settings.focus_bounds = Some(OrbitCameraFocusBounds::Cuboid { min, max })`.

## `OrbitCameraDollyZoom`

An optional component added alongside `OrbitCamera` for the Hitchcock/vertigo effect.

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `enabled` | `bool` | `false` | `true` or `false` | Activates simultaneous distance + FOV adjustment | FOV is computed as `2 * atan(reference_width / (2 * distance))` |
| `reference_width` | `f32` | `4.0` | `> 0` | The world-space width to maintain at the focus point | Determines how dramatic the effect is |

## `OrbitCameraCollision`

An optional component that provides collision avoidance infrastructure.

| Field | Type | Default | Valid Range | Effect | Notes |
| --- | --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `true` or `false` | Enables collision distance override | When collision_distance is set on internal state, camera pulls forward |
| `min_distance` | `f32` | `0.3` | `>= 0` | Minimum camera distance after collision | Prevents clipping into geometry |
| `smooth_speed` | `f32` | `12.0` | `> 0` | How quickly the collision distance relaxes back to full orbit distance | Higher is snappier |

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
- `set_preset_view(OrbitCameraPresetView)`

## `OrbitCameraPresetView`

Available presets:

- `Front`
- `Back`
- `Left`
- `Right`
- `Top`
- `Bottom`

## Perspective vs Orthographic Notes

- Perspective zoom changes `distance`
- Orthographic zoom changes `OrthographicProjection::scale`
- Both use multiplicative zoom feel rather than naive linear deltas
- Panning always happens on the camera plane
- Perspective pan speed scales with distance and FOV
- Orthographic pan speed scales with the resolved projection area
