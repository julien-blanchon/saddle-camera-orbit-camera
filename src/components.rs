use bevy::prelude::*;

use crate::math::{
    fit_orthographic_scale_for_sphere, fit_perspective_distance_for_sphere, orbit_state_from_eye,
};

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct OrbitAngleLimit {
    pub min: f32,
    pub max: f32,
}

impl OrbitAngleLimit {
    pub const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn clamp(self, value: f32) -> f32 {
        value.clamp(self.min, self.max)
    }
}

impl Default for OrbitAngleLimit {
    fn default() -> Self {
        Self::new(-1.45, 1.45)
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct OrbitZoomLimits {
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_orthographic_scale: f32,
    pub max_orthographic_scale: f32,
}

impl OrbitZoomLimits {
    pub const fn new(
        min_distance: f32,
        max_distance: f32,
        min_orthographic_scale: f32,
        max_orthographic_scale: f32,
    ) -> Self {
        Self {
            min_distance,
            max_distance,
            min_orthographic_scale,
            max_orthographic_scale,
        }
    }

    pub fn clamp_distance(self, value: f32) -> f32 {
        value.clamp(self.min_distance, self.max_distance)
    }

    pub fn clamp_orthographic_scale(self, value: f32) -> f32 {
        value.clamp(self.min_orthographic_scale, self.max_orthographic_scale)
    }
}

impl Default for OrbitZoomLimits {
    fn default() -> Self {
        Self::new(0.5, 250.0, 0.05, 128.0)
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Default)]
pub struct OrbitAxisInversion {
    pub orbit_x: bool,
    pub orbit_y: bool,
    pub pan_x: bool,
    pub pan_y: bool,
    pub zoom: bool,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct OrbitCameraMouseControls {
    pub orbit_button: MouseButton,
    pub pan_button: MouseButton,
    pub orbit_sensitivity: Vec2,
    pub pan_sensitivity: f32,
    pub wheel_zoom_sensitivity: f32,
    pub zoom_to_cursor: bool,
}

impl Default for OrbitCameraMouseControls {
    fn default() -> Self {
        Self {
            orbit_button: MouseButton::Left,
            pan_button: MouseButton::Middle,
            orbit_sensitivity: Vec2::splat(0.008),
            pan_sensitivity: 1.0,
            wheel_zoom_sensitivity: 0.14,
            zoom_to_cursor: false,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbitCameraPresetView {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}

impl OrbitCameraPresetView {
    pub fn angles(self) -> (f32, f32) {
        match self {
            Self::Front => (0.0, 0.0),
            Self::Back => (std::f32::consts::PI, 0.0),
            Self::Left => (-std::f32::consts::FRAC_PI_2, 0.0),
            Self::Right => (std::f32::consts::FRAC_PI_2, 0.0),
            Self::Top => (0.0, std::f32::consts::FRAC_PI_2),
            Self::Bottom => (0.0, -std::f32::consts::FRAC_PI_2),
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct OrbitCameraTouchControls {
    pub enabled: bool,
    pub orbit_sensitivity: Vec2,
    pub pan_sensitivity: f32,
    pub pinch_zoom_sensitivity: f32,
}

impl Default for OrbitCameraTouchControls {
    fn default() -> Self {
        Self {
            enabled: true,
            orbit_sensitivity: Vec2::splat(0.01),
            pan_sensitivity: 1.0,
            pinch_zoom_sensitivity: 0.01,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct OrbitCameraGamepadControls {
    pub enabled: bool,
    pub orbit_sensitivity: Vec2,
    pub zoom_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub deadzone: f32,
}

impl Default for OrbitCameraGamepadControls {
    fn default() -> Self {
        Self {
            enabled: false,
            orbit_sensitivity: Vec2::splat(2.5),
            zoom_sensitivity: 3.0,
            pan_sensitivity: 8.0,
            deadzone: 0.15,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct OrbitCameraSmoothing {
    pub rotation_decay: f32,
    pub focus_decay: f32,
    pub zoom_decay: f32,
}

impl Default for OrbitCameraSmoothing {
    fn default() -> Self {
        Self {
            rotation_decay: 16.0,
            focus_decay: 20.0,
            zoom_decay: 18.0,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct OrbitCameraAutoRotate {
    pub enabled: bool,
    pub wait_seconds: f32,
    pub speed: f32,
}

impl Default for OrbitCameraAutoRotate {
    fn default() -> Self {
        Self {
            enabled: false,
            wait_seconds: 2.0,
            speed: 0.45,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct OrbitCameraInertia {
    pub enabled: bool,
    pub orbit_friction: f32,
    pub pan_friction: f32,
    pub zoom_friction: f32,
}

impl Default for OrbitCameraInertia {
    fn default() -> Self {
        Self {
            enabled: false,
            orbit_friction: 5.0,
            pan_friction: 6.0,
            zoom_friction: 8.0,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub enum OrbitCameraFocusBounds {
    Sphere { center: Vec3, radius: f32 },
    Cuboid { min: Vec3, max: Vec3 },
}

impl OrbitCameraFocusBounds {
    pub fn clamp_focus(self, focus: Vec3) -> Vec3 {
        match self {
            Self::Sphere { center, radius } => {
                let offset = focus - center;
                let distance = offset.length();
                if distance <= radius || distance < f32::EPSILON {
                    focus
                } else {
                    center + offset / distance * radius
                }
            }
            Self::Cuboid { min, max } => Vec3::new(
                focus.x.clamp(min.x, max.x),
                focus.y.clamp(min.y, max.y),
                focus.z.clamp(min.z, max.z),
            ),
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct OrbitCameraSettings {
    pub enabled: bool,
    pub mouse: OrbitCameraMouseControls,
    pub touch: OrbitCameraTouchControls,
    pub gamepad: OrbitCameraGamepadControls,
    pub inversion: OrbitAxisInversion,
    pub pitch_limits: OrbitAngleLimit,
    pub yaw_limits: Option<OrbitAngleLimit>,
    pub zoom_limits: OrbitZoomLimits,
    pub smoothing: OrbitCameraSmoothing,
    pub auto_rotate: OrbitCameraAutoRotate,
    pub inertia: OrbitCameraInertia,
    pub focus_bounds: Option<OrbitCameraFocusBounds>,
    pub allow_upside_down: bool,
    pub reversed_zoom: bool,
    pub force_update: bool,
}

impl Default for OrbitCameraSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            mouse: OrbitCameraMouseControls::default(),
            touch: OrbitCameraTouchControls::default(),
            gamepad: OrbitCameraGamepadControls::default(),
            inversion: OrbitAxisInversion::default(),
            pitch_limits: OrbitAngleLimit::default(),
            yaw_limits: None,
            zoom_limits: OrbitZoomLimits::default(),
            smoothing: OrbitCameraSmoothing::default(),
            auto_rotate: OrbitCameraAutoRotate::default(),
            inertia: OrbitCameraInertia::default(),
            focus_bounds: None,
            allow_upside_down: false,
            reversed_zoom: false,
            force_update: false,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct OrbitCameraHome {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub orthographic_scale: f32,
    pub focus: Vec3,
}

impl OrbitCameraHome {
    pub const fn new(
        yaw: f32,
        pitch: f32,
        distance: f32,
        orthographic_scale: f32,
        focus: Vec3,
    ) -> Self {
        Self {
            yaw,
            pitch,
            distance,
            orthographic_scale,
            focus,
        }
    }

    pub fn from_camera(camera: &OrbitCamera) -> Self {
        Self {
            yaw: camera.yaw,
            pitch: camera.pitch,
            distance: camera.distance,
            orthographic_scale: camera.orthographic_scale,
            focus: camera.focus,
        }
    }
}

impl Default for OrbitCameraHome {
    fn default() -> Self {
        let camera = OrbitCamera::default();
        camera.home
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(Camera3d, Transform, OrbitCameraSettings, OrbitCameraInternalState)]
pub struct OrbitCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub orthographic_scale: f32,
    pub focus: Vec3,
    pub target_yaw: f32,
    pub target_pitch: f32,
    pub target_distance: f32,
    pub target_orthographic_scale: f32,
    pub target_focus: Vec3,
    pub home: OrbitCameraHome,
}

impl OrbitCamera {
    pub fn new(focus: Vec3, distance: f32) -> Self {
        let yaw = 0.0;
        let pitch = -0.35;
        let orthographic_scale = 6.0;
        let home = OrbitCameraHome::new(yaw, pitch, distance, orthographic_scale, focus);
        Self {
            yaw,
            pitch,
            distance,
            orthographic_scale,
            focus,
            target_yaw: yaw,
            target_pitch: pitch,
            target_distance: distance,
            target_orthographic_scale: orthographic_scale,
            target_focus: focus,
            home,
        }
    }

    pub fn looking_at(focus: Vec3, eye: Vec3) -> Self {
        let (yaw, pitch, distance) = orbit_state_from_eye(focus, eye);
        let orthographic_scale = 6.0;
        let home = OrbitCameraHome::new(yaw, pitch, distance, orthographic_scale, focus);
        Self {
            yaw,
            pitch,
            distance,
            orthographic_scale,
            focus,
            target_yaw: yaw,
            target_pitch: pitch,
            target_distance: distance,
            target_orthographic_scale: orthographic_scale,
            target_focus: focus,
            home,
        }
    }

    pub fn with_home(mut self, home: OrbitCameraHome) -> Self {
        self.home = home;
        self
    }

    pub fn with_orthographic_scale(mut self, scale: f32) -> Self {
        self.orthographic_scale = scale;
        self.target_orthographic_scale = scale;
        self.home.orthographic_scale = scale;
        self
    }

    pub fn reset_to_home(&mut self) {
        self.target_yaw = self.home.yaw;
        self.target_pitch = self.home.pitch;
        self.target_distance = self.home.distance;
        self.target_orthographic_scale = self.home.orthographic_scale;
        self.target_focus = self.home.focus;
    }

    pub fn capture_home_from_current(&mut self) {
        self.home = OrbitCameraHome::from_camera(self);
    }

    pub fn snap_to_target(&mut self) {
        self.yaw = self.target_yaw;
        self.pitch = self.target_pitch;
        self.distance = self.target_distance;
        self.orthographic_scale = self.target_orthographic_scale;
        self.focus = self.target_focus;
    }

    pub fn focus_on(&mut self, point: Vec3) {
        self.target_focus = point;
    }

    pub fn set_target_angles(&mut self, yaw: f32, pitch: f32) {
        self.target_yaw = yaw;
        self.target_pitch = pitch;
    }

    pub fn set_target_distance(&mut self, distance: f32) {
        self.target_distance = distance;
    }

    pub fn set_target_orthographic_scale(&mut self, scale: f32) {
        self.target_orthographic_scale = scale;
    }

    pub fn set_preset_view(&mut self, preset: OrbitCameraPresetView) {
        let (yaw, pitch) = preset.angles();
        self.target_yaw = yaw;
        self.target_pitch = pitch;
    }

    pub fn frame_sphere(
        &mut self,
        projection: &Projection,
        center: Vec3,
        radius: f32,
        padding: f32,
    ) {
        let padded_radius = radius.max(0.001);
        self.target_focus = center;
        match projection {
            Projection::Perspective(perspective) => {
                self.target_distance =
                    fit_perspective_distance_for_sphere(perspective, padded_radius, padding);
            }
            Projection::Orthographic(orthographic) => {
                self.target_orthographic_scale =
                    fit_orthographic_scale_for_sphere(orthographic, padded_radius, padding);
            }
            _ => {}
        }
    }

    pub fn frame_aabb(
        &mut self,
        projection: &Projection,
        center: Vec3,
        half_extents: Vec3,
        padding: f32,
    ) {
        self.frame_sphere(projection, center, half_extents.length(), padding);
    }
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self::looking_at(Vec3::ZERO, Vec3::new(0.0, 4.0, 8.0))
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[reflect(Component)]
pub struct OrbitCameraInputTarget;

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub struct OrbitCameraFollow {
    pub target: Entity,
    pub offset: Vec3,
    pub enabled: bool,
}

impl OrbitCameraFollow {
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            offset: Vec3::ZERO,
            enabled: true,
        }
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub struct OrbitCameraDollyZoom {
    pub enabled: bool,
    pub reference_width: f32,
}

impl OrbitCameraDollyZoom {
    pub fn new(reference_width: f32) -> Self {
        Self {
            enabled: true,
            reference_width,
        }
    }
}

impl Default for OrbitCameraDollyZoom {
    fn default() -> Self {
        Self {
            enabled: false,
            reference_width: 4.0,
        }
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub struct OrbitCameraCollision {
    pub enabled: bool,
    pub min_distance: f32,
    pub smooth_speed: f32,
}

impl Default for OrbitCameraCollision {
    fn default() -> Self {
        Self {
            enabled: true,
            min_distance: 0.3,
            smooth_speed: 12.0,
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub(crate) struct OrbitCameraInternalState {
    pub idle_seconds: f32,
    pub manual_interaction_this_frame: bool,
    pub orbit_velocity: Vec2,
    pub pan_velocity: Vec3,
    pub zoom_velocity: f32,
    pub collision_distance: Option<f32>,
}

#[cfg(test)]
#[path = "components_tests.rs"]
mod components_tests;
