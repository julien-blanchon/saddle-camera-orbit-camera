use bevy::camera::{OrthographicProjection, PerspectiveProjection};
use bevy::prelude::*;

use crate::math::{
    apply_exponential_zoom, fit_orthographic_scale_for_sphere, fit_perspective_distance_for_sphere,
    orbit_state_from_eye, orthographic_pan_translation, perspective_pan_translation, smooth_angle,
    smooth_scalar, wrap_angle,
};

#[test]
fn wrap_angle_keeps_values_in_signed_pi_range() {
    let wrapped = wrap_angle(4.5);
    assert!(wrapped <= std::f32::consts::PI);
    assert!(wrapped >= -std::f32::consts::PI);
}

#[test]
fn orbit_state_from_eye_matches_expected_distance() {
    let (_, pitch, distance) = orbit_state_from_eye(Vec3::ZERO, Vec3::new(0.0, 4.0, 8.0));
    assert!(pitch < 0.0);
    assert!((distance - (Vec3::new(0.0, 4.0, 8.0)).length()).abs() < 0.000_1);
}

#[test]
fn exponential_zoom_is_multiplicative_and_never_negative() {
    let closer = apply_exponential_zoom(10.0, 1.0, 0.25);
    let farther = apply_exponential_zoom(10.0, -1.0, 0.25);
    assert!(closer < 10.0);
    assert!(farther > 10.0);
    assert!(closer > 0.0);
}

#[test]
fn perspective_fit_grows_with_padding() {
    let projection = PerspectiveProjection {
        fov: std::f32::consts::FRAC_PI_3,
        aspect_ratio: 16.0 / 9.0,
        ..default()
    };

    let tight = fit_perspective_distance_for_sphere(&projection, 2.0, 1.0);
    let padded = fit_perspective_distance_for_sphere(&projection, 2.0, 1.4);
    assert!(padded > tight);
}

#[test]
fn orthographic_fit_grows_with_radius() {
    let mut projection = OrthographicProjection::default_3d();
    projection.scale = 4.0;
    projection.area = Rect::from_center_size(Vec2::ZERO, Vec2::new(16.0, 9.0));

    let close = fit_orthographic_scale_for_sphere(&projection, 1.0, 1.1);
    let far = fit_orthographic_scale_for_sphere(&projection, 4.0, 1.1);
    assert!(far > close);
}

#[test]
fn perspective_pan_translation_scales_with_distance() {
    let projection = PerspectiveProjection {
        fov: std::f32::consts::FRAC_PI_3,
        aspect_ratio: 16.0 / 9.0,
        ..default()
    };
    let near = perspective_pan_translation(
        Vec2::new(1280.0, 720.0),
        &projection,
        Quat::IDENTITY,
        4.0,
        Vec2::new(64.0, 0.0),
    );
    let far = perspective_pan_translation(
        Vec2::new(1280.0, 720.0),
        &projection,
        Quat::IDENTITY,
        16.0,
        Vec2::new(64.0, 0.0),
    );

    assert!(far.length() > near.length());
}

#[test]
fn orthographic_pan_translation_uses_projection_area() {
    let mut projection = OrthographicProjection::default_3d();
    projection.area = Rect::from_center_size(Vec2::ZERO, Vec2::new(20.0, 10.0));

    let translation = orthographic_pan_translation(
        Vec2::new(1000.0, 500.0),
        &projection,
        Quat::IDENTITY,
        Vec2::new(100.0, 0.0),
    );
    assert!(translation.x < 0.0);
    assert!((translation.x.abs() - 2.0).abs() < 0.000_1);
}

#[test]
fn smoothing_converges_without_overshooting() {
    let value = smooth_scalar(0.0, 10.0, 12.0, 1.0 / 60.0);
    assert!(value > 0.0);
    assert!(value < 10.0);

    let angle = smooth_angle(0.0, std::f32::consts::PI, 12.0, 1.0 / 60.0);
    assert!(angle.abs() > 0.0);
    assert!(angle.abs() < std::f32::consts::PI);
}
