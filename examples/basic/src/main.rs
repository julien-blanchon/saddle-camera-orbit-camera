use saddle_camera_orbit_camera_example_common as common;

use bevy::{
    camera::{PerspectiveProjection, Projection},
    prelude::*,
};
use saddle_camera_orbit_camera::{OrbitCamera, OrbitCameraAutoRotate, OrbitCameraPlugin, OrbitCameraSettings};

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "orbit_camera basic".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        OrbitCameraPlugin::default(),
    ));
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_reference_world(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Orbit Camera",
        "Left drag: orbit\nMiddle drag: pan\nWheel: zoom\nIdle auto-rotate resumes after a short pause.",
        Color::srgb(0.88, 0.56, 0.20),
    );

    let settings = OrbitCameraSettings {
        auto_rotate: OrbitCameraAutoRotate {
            enabled: true,
            wait_seconds: 1.5,
            speed: 0.22,
        },
        ..OrbitCameraSettings::default()
    };

    common::spawn_orbit_camera(
        &mut commands,
        "Basic Orbit Camera",
        OrbitCamera::looking_at(common::DEFAULT_FOCUS, Vec3::new(-7.5, 5.2, 10.0)),
        settings,
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );
}
