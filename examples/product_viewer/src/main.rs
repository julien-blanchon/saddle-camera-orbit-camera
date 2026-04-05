//! # Orbit Camera — Product Viewer Example
//!
//! Showcases inertia, auto-framing, auto-rotate, and gamepad controls
//! in a product viewer scenario. Flick to spin, pinch to zoom, or use
//! a gamepad right-stick to orbit.

use saddle_camera_orbit_camera_example_common as common;

use bevy::{
    camera::{PerspectiveProjection, Projection},
    prelude::*,
};
use saddle_camera_orbit_camera::{
    OrbitCamera, OrbitCameraAutoRotate, OrbitCameraGamepadControls, OrbitCameraInertia,
    OrbitCameraPlugin, OrbitCameraSettings,
};

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "orbit_camera product viewer".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        OrbitCameraPlugin::default(),
    ));
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, animate_product);
    app.run();
}

#[derive(Component)]
struct ProductModel;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_reference_world(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Product Viewer",
        "Inertia: flick the mouse to spin the view\n\
         Gamepad: right-stick orbit, triggers zoom\n\
         Auto-rotate resumes after idle\n\
         Left drag: orbit  Middle drag: pan  Wheel: zoom",
        Color::srgb(0.90, 0.72, 0.20),
    );

    // Product model: a torus knot approximation using stacked toruses
    let torus_mesh = meshes.add(Torus::new(0.35, 0.95));
    for i in 0..3 {
        let angle = i as f32 * std::f32::consts::TAU / 3.0;
        commands.spawn((
            Name::new(format!("Product Ring {}", i + 1)),
            ProductModel,
            Mesh3d(torus_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.95, 0.78, 0.22),
                metallic: 0.9,
                perceptual_roughness: 0.15,
                ..default()
            })),
            Transform::from_translation(Vec3::new(0.0, 1.2, 0.0))
                .with_rotation(Quat::from_axis_angle(Vec3::Y, angle)),
        ));
    }

    let settings = OrbitCameraSettings {
        auto_rotate: OrbitCameraAutoRotate {
            enabled: true,
            wait_seconds: 2.0,
            speed: 0.3,
        },
        inertia: OrbitCameraInertia {
            enabled: true,
            orbit_friction: 4.0,
            pan_friction: 6.0,
            zoom_friction: 8.0,
        },
        gamepad: OrbitCameraGamepadControls {
            enabled: true,
            ..OrbitCameraGamepadControls::default()
        },
        ..OrbitCameraSettings::default()
    };
    let orbit = OrbitCamera::looking_at(common::DEFAULT_FOCUS, Vec3::new(0.0, 3.5, 7.0));

    common::spawn_orbit_camera(
        &mut commands,
        "Product Viewer Camera",
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

fn animate_product(time: Res<Time>, mut models: Query<&mut Transform, With<ProductModel>>) {
    let t = time.elapsed_secs();
    for mut transform in &mut models {
        transform.translation.y = 1.2 + 0.08 * (t * 1.2).sin();
    }
}
