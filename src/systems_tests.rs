use bevy::{
    app::PostStartup,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    input::touch::{TouchInput, TouchPhase},
    prelude::*,
    time::TimeUpdateStrategy,
};

use crate::{
    OrbitCamera, OrbitCameraFollow, OrbitCameraInputTarget, OrbitCameraPlugin, OrbitCameraSettings,
    OrbitCameraSystems,
};

#[derive(Component)]
struct FollowTargetMarker;

fn spawn_camera(
    app: &mut App,
    camera: OrbitCamera,
    settings: OrbitCameraSettings,
    projection: Projection,
    input_target: bool,
) -> Entity {
    let mut entity = app.world_mut().spawn((camera, settings, projection));
    if input_target {
        entity.insert(OrbitCameraInputTarget);
    }
    entity.id()
}

fn start_runtime(app: &mut App) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.finish();
    app.world_mut().run_schedule(PostStartup);
}

#[test]
fn disabled_settings_block_input_and_auto_rotate() {
    let mut app = App::new();
    let settings = OrbitCameraSettings {
        enabled: false,
        auto_rotate: crate::OrbitCameraAutoRotate {
            enabled: true,
            ..default()
        },
        ..OrbitCameraSettings::default()
    };

    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default());
    let entity = spawn_camera(
        &mut app,
        OrbitCamera::default(),
        settings,
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );

    start_runtime(&mut app);
    app.world_mut()
        .resource_mut::<AccumulatedMouseMotion>()
        .delta = Vec2::new(100.0, 0.0);
    app.update();

    let camera = app
        .world()
        .get::<OrbitCamera>(entity)
        .expect("camera exists");
    assert_eq!(camera.yaw, camera.home.yaw);
    assert_eq!(camera.target_yaw, camera.home.yaw);
}

#[test]
fn transform_changes_after_synthetic_pointer_input() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default());
    let entity = spawn_camera(
        &mut app,
        OrbitCamera::default(),
        OrbitCameraSettings::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );

    start_runtime(&mut app);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.world_mut()
        .resource_mut::<AccumulatedMouseMotion>()
        .delta = Vec2::new(120.0, -60.0);
    app.update();

    let transform = app
        .world()
        .get::<Transform>(entity)
        .expect("transform exists");
    let camera = app
        .world()
        .get::<OrbitCamera>(entity)
        .expect("camera exists");
    assert!(camera.target_yaw.abs() > camera.home.yaw.abs());
    assert_ne!(transform.translation, Vec3::ZERO);
}

#[test]
fn multiple_orbit_cameras_do_not_interfere() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default());
    let active = spawn_camera(
        &mut app,
        OrbitCamera::default(),
        OrbitCameraSettings::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );
    let passive = spawn_camera(
        &mut app,
        OrbitCamera::looking_at(Vec3::ZERO, Vec3::new(0.0, 6.0, 12.0)),
        OrbitCameraSettings::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        false,
    );

    start_runtime(&mut app);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.world_mut()
        .resource_mut::<AccumulatedMouseMotion>()
        .delta = Vec2::new(80.0, 0.0);
    app.update();

    let active_camera = app
        .world()
        .get::<OrbitCamera>(active)
        .expect("active camera exists");
    let passive_camera = app
        .world()
        .get::<OrbitCamera>(passive)
        .expect("passive camera exists");
    assert!(active_camera.target_yaw != active_camera.home.yaw);
    assert_eq!(passive_camera.target_yaw, passive_camera.home.yaw);
}

#[test]
fn missing_follow_target_fails_gracefully() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default());
    let target = app.world_mut().spawn_empty().id();
    let entity = spawn_camera(
        &mut app,
        OrbitCamera::default(),
        OrbitCameraSettings::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        false,
    );
    app.world_mut()
        .entity_mut(entity)
        .insert(OrbitCameraFollow::new(target));

    app.world_mut().despawn(target);
    start_runtime(&mut app);
    app.update();

    let camera = app
        .world()
        .get::<OrbitCamera>(entity)
        .expect("camera exists");
    assert_eq!(camera.target_focus, camera.home.focus);
}

#[test]
fn orthographic_zoom_updates_projection_scale() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default());
    let camera = OrbitCamera::default().with_orthographic_scale(8.0);
    let mut settings = OrbitCameraSettings::default();
    settings.mouse.wheel_zoom_sensitivity = 0.2;
    let entity = spawn_camera(
        &mut app,
        camera,
        settings,
        Projection::Orthographic(OrthographicProjection::default_3d()),
        true,
    );

    start_runtime(&mut app);
    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta = Vec2::new(0.0, 4.0);
    app.update();

    let projection = app
        .world()
        .get::<Projection>(entity)
        .expect("projection exists");
    let Projection::Orthographic(orthographic) = projection else {
        panic!("expected orthographic projection");
    };
    assert!(orthographic.scale < 8.0);
}

#[test]
fn switching_projection_preserves_camera_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default());
    let entity = spawn_camera(
        &mut app,
        OrbitCamera::default(),
        OrbitCameraSettings::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        false,
    );

    start_runtime(&mut app);
    app.world_mut()
        .entity_mut(entity)
        .insert(Projection::Orthographic(
            OrthographicProjection::default_3d(),
        ));
    app.update();

    let projection = app
        .world()
        .get::<Projection>(entity)
        .expect("projection exists");
    let Projection::Orthographic(orthographic) = projection else {
        panic!("expected orthographic projection");
    };
    assert!(orthographic.scale > 0.0);
}

#[test]
fn highest_camera_order_wins_shared_pointer_input() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default());

    let low_order = spawn_camera(
        &mut app,
        OrbitCamera::default(),
        OrbitCameraSettings::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );
    app.world_mut().entity_mut(low_order).insert(Camera {
        order: 0,
        ..default()
    });

    let high_order = spawn_camera(
        &mut app,
        OrbitCamera::looking_at(Vec3::ZERO, Vec3::new(0.0, 6.0, 12.0)),
        OrbitCameraSettings::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );
    app.world_mut().entity_mut(high_order).insert(Camera {
        order: 5,
        ..default()
    });

    start_runtime(&mut app);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.world_mut()
        .resource_mut::<AccumulatedMouseMotion>()
        .delta = Vec2::new(90.0, 0.0);
    app.update();

    let low_order_camera = app.world().get::<OrbitCamera>(low_order).unwrap();
    let high_order_camera = app.world().get::<OrbitCamera>(high_order).unwrap();
    assert_eq!(low_order_camera.target_yaw, low_order_camera.home.yaw);
    assert_ne!(high_order_camera.target_yaw, high_order_camera.home.yaw);
}

#[test]
fn touch_input_orbits_when_enabled() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default());
    let entity = spawn_camera(
        &mut app,
        OrbitCamera::default(),
        OrbitCameraSettings::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );
    let window = app.world_mut().spawn_empty().id();

    start_runtime(&mut app);
    app.world_mut().write_message(TouchInput {
        phase: TouchPhase::Started,
        position: Vec2::new(200.0, 200.0),
        window,
        force: None,
        id: 1,
    });
    app.update();

    app.world_mut().write_message(TouchInput {
        phase: TouchPhase::Moved,
        position: Vec2::new(280.0, 160.0),
        window,
        force: None,
        id: 1,
    });
    app.update();

    let camera = app.world().get::<OrbitCamera>(entity).unwrap();
    assert_ne!(camera.target_yaw, camera.home.yaw);
    assert_ne!(camera.target_pitch, camera.home.pitch);
}

#[test]
fn disabled_touch_controls_ignore_touch_input() {
    let mut app = App::new();
    let mut settings = OrbitCameraSettings::default();
    settings.touch.enabled = false;

    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default());
    let entity = spawn_camera(
        &mut app,
        OrbitCamera::default(),
        settings,
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );
    let window = app.world_mut().spawn_empty().id();

    start_runtime(&mut app);
    app.world_mut().write_message(TouchInput {
        phase: TouchPhase::Started,
        position: Vec2::new(200.0, 200.0),
        window,
        force: None,
        id: 1,
    });
    app.update();

    app.world_mut().write_message(TouchInput {
        phase: TouchPhase::Moved,
        position: Vec2::new(280.0, 160.0),
        window,
        force: None,
        id: 1,
    });
    app.update();

    let camera = app.world().get::<OrbitCamera>(entity).unwrap();
    assert_eq!(camera.target_yaw, camera.home.yaw);
    assert_eq!(camera.target_pitch, camera.home.pitch);
}

#[test]
fn follow_target_uses_current_frame_transform_changes() {
    fn move_target(mut targets: Query<&mut Transform, With<FollowTargetMarker>>) {
        let Ok(mut transform) = targets.single_mut() else {
            return;
        };
        transform.translation = Vec3::new(3.0, 2.0, -4.0);
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default())
        .add_systems(Update, move_target.before(OrbitCameraSystems::ApplyIntent));

    let target = app
        .world_mut()
        .spawn((
            FollowTargetMarker,
            Transform::from_xyz(-5.0, 1.0, 0.0),
            GlobalTransform::default(),
        ))
        .id();

    let camera = spawn_camera(
        &mut app,
        OrbitCamera::default(),
        OrbitCameraSettings::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        false,
    );
    app.world_mut()
        .entity_mut(camera)
        .insert(OrbitCameraFollow {
            target,
            offset: Vec3::new(0.0, 0.5, 0.0),
            enabled: true,
        });

    start_runtime(&mut app);
    app.update();

    let camera = app.world().get::<OrbitCamera>(camera).unwrap();
    assert!((camera.target_focus - Vec3::new(3.0, 2.5, -4.0)).length() < 0.000_1);
}

#[test]
fn reversed_zoom_inverts_scroll_direction() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default());

    let settings = OrbitCameraSettings {
        reversed_zoom: true,
        mouse: crate::OrbitCameraMouseControls {
            wheel_zoom_sensitivity: 0.2,
            ..default()
        },
        ..default()
    };

    let entity = spawn_camera(
        &mut app,
        OrbitCamera::default(),
        settings,
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );

    start_runtime(&mut app);
    let home_distance = app
        .world()
        .get::<OrbitCamera>(entity)
        .unwrap()
        .home
        .distance;

    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta = Vec2::new(0.0, 4.0);
    app.update();

    let camera = app.world().get::<OrbitCamera>(entity).unwrap();
    // Reversed zoom: positive scroll should zoom out (increase distance)
    assert!(camera.target_distance > home_distance);
}

#[test]
fn focus_bounds_clamp_target_focus_in_advance_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default());

    let settings = OrbitCameraSettings {
        focus_bounds: Some(crate::OrbitCameraFocusBounds::Cuboid {
            min: Vec3::new(-5.0, -5.0, -5.0),
            max: Vec3::new(5.0, 5.0, 5.0),
        }),
        ..default()
    };

    let camera = OrbitCamera {
        target_focus: Vec3::new(20.0, 0.0, 0.0),
        ..default()
    };

    let entity = spawn_camera(
        &mut app,
        camera,
        settings,
        Projection::Perspective(PerspectiveProjection::default()),
        false,
    );

    start_runtime(&mut app);
    app.update();

    let camera = app.world().get::<OrbitCamera>(entity).unwrap();
    assert!(camera.target_focus.x <= 5.0);
}

#[test]
fn force_update_advances_state_while_disabled() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::default());

    let settings = OrbitCameraSettings {
        enabled: false,
        force_update: true,
        ..default()
    };

    let camera = OrbitCamera {
        target_yaw: 1.0,
        ..default()
    };

    let entity = spawn_camera(
        &mut app,
        camera,
        settings,
        Projection::Perspective(PerspectiveProjection::default()),
        false,
    );

    start_runtime(&mut app);
    app.update();

    let camera = app.world().get::<OrbitCamera>(entity).unwrap();
    // force_update should have allowed state advancement despite enabled=false
    assert!(camera.yaw.abs() > 0.0);

    // force_update should auto-reset
    let settings = app.world().get::<OrbitCameraSettings>(entity).unwrap();
    assert!(!settings.force_update);
}
