use saddle_camera_orbit_camera_example_common as common;

use bevy::{
    camera::{PerspectiveProjection, Projection},
    prelude::*,
};
use saddle_camera_orbit_camera::{OrbitCamera, OrbitCameraPlugin, OrbitCameraSettings};

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "orbit_camera touch viewer".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }),
        OrbitCameraPlugin::default(),
    ));
    common::install_pane(&mut app);
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
        "Touch Viewer",
        "One finger: orbit\nTwo fingers: pan\nPinch: zoom\nDesktop mouse controls remain active in the same runtime.",
        Color::srgb(0.92, 0.44, 0.30),
    );

    let mut settings = OrbitCameraSettings::default();
    settings.touch.enabled = true;
    settings.touch.pinch_zoom_sensitivity = 0.012;
    settings.mouse.zoom_to_cursor = true;
    let orbit = OrbitCamera::looking_at(common::DEFAULT_FOCUS, Vec3::new(0.0, 4.5, 9.0));

    common::spawn_orbit_camera(
        &mut commands,
        "Touch Orbit Camera",
        orbit.clone(),
        settings.clone(),
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );
    common::queue_example_pane(
        &mut commands,
        common::ExampleOrbitPane::from_setup(&orbit, &settings),
    );
}
