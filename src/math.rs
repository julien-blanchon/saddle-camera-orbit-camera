use bevy::{
    camera::{OrthographicProjection, PerspectiveProjection},
    math::StableInterpolate,
    prelude::*,
};

pub fn wrap_angle(angle: f32) -> f32 {
    let wrapped = angle.rem_euclid(std::f32::consts::TAU);
    if wrapped > std::f32::consts::PI {
        wrapped - std::f32::consts::TAU
    } else {
        wrapped
    }
}

pub fn shortest_angle_delta(current: f32, target: f32) -> f32 {
    wrap_angle(target - current)
}

pub fn smooth_factor(decay_rate: f32, dt: f32) -> f32 {
    if decay_rate <= 0.0 || dt <= 0.0 {
        1.0
    } else {
        1.0 - (-decay_rate * dt).exp()
    }
}

pub fn smooth_scalar(current: f32, target: f32, decay_rate: f32, dt: f32) -> f32 {
    if decay_rate <= 0.0 || dt <= 0.0 {
        target
    } else {
        current + (target - current) * smooth_factor(decay_rate, dt)
    }
}

pub fn smooth_angle(current: f32, target: f32, decay_rate: f32, dt: f32) -> f32 {
    wrap_angle(current + shortest_angle_delta(current, target) * smooth_factor(decay_rate, dt))
}

pub fn orbit_rotation(yaw: f32, pitch: f32) -> Quat {
    Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0)
}

pub fn orbit_translation(focus: Vec3, yaw: f32, pitch: f32, distance: f32) -> Vec3 {
    focus + orbit_rotation(yaw, pitch) * Vec3::Z * distance.max(0.001)
}

pub fn orbit_state_from_eye(focus: Vec3, eye: Vec3) -> (f32, f32, f32) {
    let offset = eye - focus;
    let distance = offset.length().max(0.001);
    let horizontal = Vec2::new(offset.x, offset.z);
    let yaw = if horizontal.length_squared() <= f32::EPSILON {
        0.0
    } else {
        horizontal.x.atan2(horizontal.y)
    };
    let pitch = (-offset.y).atan2(horizontal.length().max(0.000_1));
    (wrap_angle(yaw), pitch, distance)
}

pub fn perspective_pan_translation(
    viewport_size: Vec2,
    projection: &PerspectiveProjection,
    rotation: Quat,
    distance: f32,
    delta_pixels: Vec2,
) -> Vec3 {
    let width = viewport_size.x.max(1.0);
    let height = viewport_size.y.max(1.0);
    let vertical_span = 2.0 * distance.max(0.001) * (projection.fov * 0.5).tan();
    let horizontal_span = vertical_span * projection.aspect_ratio.max(0.001);
    let delta_x = delta_pixels.x / width * horizontal_span;
    let delta_y = delta_pixels.y / height * vertical_span;
    rotation * (Vec3::X * -delta_x + Vec3::Y * delta_y)
}

pub fn orthographic_pan_translation(
    viewport_size: Vec2,
    projection: &OrthographicProjection,
    rotation: Quat,
    delta_pixels: Vec2,
) -> Vec3 {
    let width = viewport_size.x.max(1.0);
    let height = viewport_size.y.max(1.0);
    let world_width = projection.area.width().abs().max(0.001);
    let world_height = projection.area.height().abs().max(0.001);
    let delta_x = delta_pixels.x / width * world_width;
    let delta_y = delta_pixels.y / height * world_height;
    rotation * (Vec3::X * -delta_x + Vec3::Y * delta_y)
}

pub fn apply_exponential_zoom(current: f32, delta: f32, sensitivity: f32) -> f32 {
    if delta.abs() <= f32::EPSILON || sensitivity <= 0.0 {
        current
    } else {
        current.max(0.001) * (-delta * sensitivity).exp()
    }
}

pub fn fit_perspective_distance_for_sphere(
    projection: &PerspectiveProjection,
    radius: f32,
    padding: f32,
) -> f32 {
    let padded_radius = radius.max(0.001) * padding.max(1.0);
    let half_vertical = projection.fov * 0.5;
    let half_horizontal = (half_vertical.tan() * projection.aspect_ratio.max(0.001)).atan();
    let limiting_half_angle = half_vertical.min(half_horizontal).max(0.01);
    padded_radius / limiting_half_angle.sin().max(0.001)
}

pub fn fit_orthographic_scale_for_sphere(
    projection: &OrthographicProjection,
    radius: f32,
    padding: f32,
) -> f32 {
    let padded_diameter = radius.max(0.001) * padding.max(1.0) * 2.0;
    let current_height = projection.area.height().abs().max(0.001);
    let current_width = projection.area.width().abs().max(0.001);
    let height_scale = projection.scale * padded_diameter / current_height;
    let width_scale = projection.scale * padded_diameter / current_width;
    height_scale.max(width_scale)
}

pub fn smooth_vec3(current: Vec3, target: Vec3, decay_rate: f32, dt: f32) -> Vec3 {
    if decay_rate <= 0.0 || dt <= 0.0 {
        target
    } else {
        let mut value = current;
        value.smooth_nudge(&target, decay_rate, dt);
        value
    }
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod math_tests;
