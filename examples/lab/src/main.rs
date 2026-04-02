use saddle_camera_orbit_camera_example_common as common;
#[cfg(feature = "e2e")]
mod e2e;

use bevy::{
    prelude::*,
    remote::{RemotePlugin, http::RemoteHttpPlugin},
};
#[cfg(feature = "brp")]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_camera_orbit_camera::{
    OrbitCamera, OrbitCameraFollow, OrbitCameraInputTarget, OrbitCameraPlugin, OrbitCameraSettings,
    OrbitCameraSystems,
};

#[derive(Component)]
struct LabFollowTarget;

#[derive(Component)]
struct LabOverlay;

#[derive(Resource, Clone, Copy)]
pub struct LabCameraEntity(pub Entity);

#[derive(Resource, Clone, Copy)]
pub struct LabTargetEntity(pub Entity);

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "orbit_camera_lab".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        OrbitCameraPlugin::default(),
        RemotePlugin::default(),
    ));
    #[cfg(feature = "brp")]
    app.add_plugins(BrpExtrasPlugin::with_http_plugin(
        RemoteHttpPlugin::default(),
    ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::OrbitCameraLabE2EPlugin);

    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            animate_target.before(OrbitCameraSystems::ApplyIntent),
            update_overlay.after(OrbitCameraSystems::ApplyIntent),
        ),
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
        "Orbit Camera Lab",
        "LMB orbit, MMB pan, wheel zoom.\nThe overlay mirrors the shared component state for BRP and E2E.",
        Color::srgb(0.92, 0.58, 0.26),
    );

    let target = commands
        .spawn((
            Name::new("Lab Follow Target"),
            LabFollowTarget,
            Mesh3d(meshes.add(Capsule3d::new(0.55, 1.6).mesh().rings(10).latitudes(14))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.92, 0.22, 0.34),
                perceptual_roughness: 0.32,
                ..default()
            })),
            Transform::from_xyz(4.5, 1.0, 0.0),
        ))
        .id();

    let camera = common::spawn_orbit_camera(
        &mut commands,
        "Lab Orbit Camera",
        OrbitCamera::looking_at(common::DEFAULT_FOCUS, Vec3::new(-8.0, 5.5, 12.0))
            .with_orthographic_scale(1.25),
        OrbitCameraSettings::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );
    commands.insert_resource(LabCameraEntity(camera));
    commands.insert_resource(LabTargetEntity(target));

    commands.spawn((
        Name::new("Lab Overlay"),
        LabOverlay,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(420.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.82)),
        Text::default(),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));

    commands.spawn((
        Name::new("Lab Orthographic Board"),
        Mesh3d(meshes.add(Cuboid::new(7.0, 0.2, 4.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.28, 0.34),
            perceptual_roughness: 0.92,
            ..default()
        })),
        Transform::from_xyz(8.5, 0.1, -6.0),
    ));
}

fn animate_target(time: Res<Time>, mut targets: Query<&mut Transform, With<LabFollowTarget>>) {
    let Ok(mut transform) = targets.single_mut() else {
        return;
    };

    let t = time.elapsed_secs() * 0.7;
    transform.translation.x = 4.0 * t.cos();
    transform.translation.z = 3.2 * t.sin();
    transform.translation.y = 1.0 + 0.35 * (t * 1.8).sin().abs();
}

fn update_overlay(
    camera_entity: Res<LabCameraEntity>,
    target_entity: Res<LabTargetEntity>,
    cameras: Query<(
        &OrbitCamera,
        Option<&OrbitCameraFollow>,
        Option<&OrbitCameraInputTarget>,
    )>,
    targets: Query<&Transform, With<LabFollowTarget>>,
    mut overlays: Query<&mut Text, With<LabOverlay>>,
) {
    let Ok((orbit, follow, input_target)) = cameras.get(camera_entity.0) else {
        return;
    };
    let Ok(target_transform) = targets.get(target_entity.0) else {
        return;
    };
    let Ok(mut text) = overlays.single_mut() else {
        return;
    };

    text.0 = format!(
        "Orbit Camera Lab\nfocus {:.2?}\nangles yaw {:.2} pitch {:.2}\ndistance {:.2} ortho {:.2}\ntarget focus {:.2?}\nfollow {:?}\ninput target {}\ntracked entity {:.2?}",
        orbit.focus,
        orbit.yaw,
        orbit.pitch,
        orbit.distance,
        orbit.orthographic_scale,
        orbit.target_focus,
        follow.map(|follow| (follow.enabled, follow.offset)),
        input_target.is_some(),
        target_transform.translation,
    );
}
