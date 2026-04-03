use bevy::prelude::*;

use crate::{OrbitCamera, OrbitCameraPresetView};

#[test]
fn looking_at_round_trips_focus_and_distance() {
    let focus = Vec3::new(2.0, 1.0, -3.0);
    let eye = Vec3::new(6.0, 4.0, 8.0);
    let camera = OrbitCamera::looking_at(focus, eye);

    assert!((camera.focus - focus).length() < 0.000_1);
    assert!((camera.distance - (eye - focus).length()).abs() < 0.000_1);
    assert!((camera.target_distance - camera.distance).abs() < 0.000_1);
}

#[test]
fn reset_to_home_restores_targets() {
    let mut camera = OrbitCamera {
        target_focus: Vec3::new(10.0, 2.0, -8.0),
        target_yaw: 1.2,
        target_pitch: -0.8,
        target_distance: 22.0,
        target_orthographic_scale: 9.0,
        ..OrbitCamera::default()
    };

    camera.reset_to_home();

    assert_eq!(camera.target_focus, camera.home.focus);
    assert_eq!(camera.target_yaw, camera.home.yaw);
    assert_eq!(camera.target_pitch, camera.home.pitch);
    assert_eq!(camera.target_distance, camera.home.distance);
    assert_eq!(
        camera.target_orthographic_scale,
        camera.home.orthographic_scale
    );
}

#[test]
fn frame_aabb_uses_half_extents_length_as_bounds_radius() {
    let mut camera = OrbitCamera::default();
    let projection = Projection::Perspective(PerspectiveProjection::default());

    camera.frame_aabb(
        &projection,
        Vec3::new(1.0, 2.0, 3.0),
        Vec3::new(2.0, 1.0, 2.0),
        1.2,
    );

    assert_eq!(camera.target_focus, Vec3::new(1.0, 2.0, 3.0));
    assert!(camera.target_distance > camera.distance);
}

#[test]
fn preset_views_update_target_angles() {
    let mut camera = OrbitCamera::default();

    camera.set_preset_view(OrbitCameraPresetView::Right);
    assert!((camera.target_yaw - std::f32::consts::FRAC_PI_2).abs() < 0.000_1);
    assert!(camera.target_pitch.abs() < 0.000_1);

    camera.set_preset_view(OrbitCameraPresetView::Top);
    assert!((camera.target_pitch - std::f32::consts::FRAC_PI_2).abs() < 0.000_1);
}
