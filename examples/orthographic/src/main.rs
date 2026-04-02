use saddle_camera_orbit_camera_example_common as common;

use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;
use saddle_camera_orbit_camera::{OrbitAngleLimit, OrbitCamera, OrbitCameraPlugin, OrbitCameraSettings};

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "orbit_camera orthographic".into(),
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
        "Orthographic Overview",
        "The same controller works with an orthographic projection.\nPitch is constrained for a tactics-style board view.",
        Color::srgb(0.24, 0.76, 0.58),
    );

    let tile_mesh = meshes.add(Cuboid::new(1.8, 0.2, 1.8));
    for x in -3..=3 {
        for z in -3..=3 {
            let hue = if (x + z) % 2 == 0 { 0.18 } else { 0.28 };
            commands.spawn((
                Name::new(format!("Board Tile {x} {z}")),
                Mesh3d(tile_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(hue, hue + 0.03, hue + 0.06),
                    perceptual_roughness: 0.9,
                    ..default()
                })),
                Transform::from_xyz(x as f32 * 2.2, 0.12, z as f32 * 2.2),
            ));
        }
    }

    let settings = OrbitCameraSettings {
        pitch_limits: OrbitAngleLimit::new(-1.35, -0.35),
        ..OrbitCameraSettings::default()
    };

    common::spawn_orbit_camera(
        &mut commands,
        "Orthographic Orbit Camera",
        OrbitCamera::looking_at(Vec3::new(0.0, 0.6, 0.0), Vec3::new(0.0, 18.0, 18.0))
            .with_orthographic_scale(1.35),
        settings,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.35,
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 18.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        true,
    );
}
