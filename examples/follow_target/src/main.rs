use saddle_camera_orbit_camera_example_common as common;

use bevy::{
    camera::{PerspectiveProjection, Projection},
    prelude::*,
};
use saddle_camera_orbit_camera::{
    OrbitCamera, OrbitCameraFollow, OrbitCameraPlugin, OrbitCameraSettings, OrbitCameraSystems,
};

#[derive(Component)]
struct FollowTarget;

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "orbit_camera follow target".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        OrbitCameraPlugin::default(),
    ));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        animate_target.before(OrbitCameraSystems::ApplyIntent),
    );
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
        "Follow Target",
        "Orbit and zoom still work while the focus follows the moving target.\nPanning adjusts the follow offset instead of detaching the camera.",
        Color::srgb(0.76, 0.30, 0.34),
    );

    let target = commands
        .spawn((
            Name::new("Follow Target"),
            FollowTarget,
            Mesh3d(meshes.add(Cuboid::new(1.1, 1.1, 1.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.92, 0.22, 0.32),
                perceptual_roughness: 0.35,
                ..default()
            })),
            Transform::from_xyz(5.0, 0.55, 0.0),
        ))
        .id();

    let camera = common::spawn_orbit_camera(
        &mut commands,
        "Follow Orbit Camera",
        OrbitCamera::looking_at(Vec3::new(5.0, 1.0, 0.0), Vec3::new(-3.0, 5.0, 10.0)),
        OrbitCameraSettings::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );
    commands.entity(camera).insert(OrbitCameraFollow {
        target,
        offset: Vec3::new(0.0, 0.5, 0.0),
        enabled: true,
    });
}

fn animate_target(time: Res<Time>, mut target: Query<&mut Transform, With<FollowTarget>>) {
    let Ok(mut transform) = target.single_mut() else {
        return;
    };

    let t = time.elapsed_secs() * 0.7;
    transform.translation.x = 5.0 * t.cos();
    transform.translation.z = 3.0 * t.sin();
    transform.translation.y = 0.55 + 0.4 * (t * 1.6).sin().abs();
}
