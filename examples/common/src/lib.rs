use bevy::{app::AppExit, camera::Projection, light::GlobalAmbientLight, prelude::*};
use saddle_camera_orbit_camera::{OrbitCamera, OrbitCameraInputTarget, OrbitCameraSettings};

pub const DEFAULT_FOCUS: Vec3 = Vec3::new(0.0, 1.2, 0.0);

#[derive(Resource)]
struct AutoExitAfter(Timer);

pub fn apply_example_defaults(app: &mut App) {
    app.insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.07)));

    if let Some(timer) = auto_exit_from_env() {
        app.insert_resource(timer);
        app.add_systems(Update, auto_exit_after);
    }
}

pub fn spawn_reference_world(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    title: &str,
    instructions: &str,
    accent: Color,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.58, 0.60, 0.68),
        brightness: 120.0,
        affects_lightmapped_meshes: true,
    });

    commands.spawn((
        Name::new("Reference Sun"),
        DirectionalLight {
            illuminance: 32_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.7, 0.0)),
    ));

    commands.spawn((
        Name::new("Reference Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(48.0, 48.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.13, 0.16),
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));

    commands.spawn((
        Name::new("Reference Plinth"),
        Mesh3d(meshes.add(Cuboid::new(3.0, 0.6, 3.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.28, 0.30, 0.34),
            metallic: 0.08,
            perceptual_roughness: 0.4,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, 0.3, 0.0)),
    ));

    commands.spawn((
        Name::new("Reference Accent"),
        Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(5).expect("icosphere"))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: accent,
            metallic: 0.04,
            perceptual_roughness: 0.28,
            ..default()
        })),
        Transform::from_translation(DEFAULT_FOCUS),
    ));

    let pillar_mesh = meshes.add(Cuboid::new(0.45, 3.2, 0.45));
    for (index, (x, z)) in [(-6.0, -6.0), (6.0, -6.0), (-6.0, 6.0), (6.0, 6.0)]
        .into_iter()
        .enumerate()
    {
        commands.spawn((
            Name::new(format!("Reference Pillar {}", index + 1)),
            Mesh3d(pillar_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.48, 0.50, 0.58),
                perceptual_roughness: 0.55,
                ..default()
            })),
            Transform::from_xyz(x, 1.6, z),
        ));
    }

    commands.spawn((
        Name::new("Reference Overlay"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(420.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.78)),
        Text::new(format!("{title}\n{instructions}")),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub fn spawn_orbit_camera(
    commands: &mut Commands,
    name: &str,
    camera: OrbitCamera,
    settings: OrbitCameraSettings,
    projection: Projection,
    input_target: bool,
) -> Entity {
    let mut entity = commands.spawn((Name::new(name.to_owned()), camera, settings, projection));
    if input_target {
        entity.insert(OrbitCameraInputTarget);
    }
    entity.id()
}

fn auto_exit_from_env() -> Option<AutoExitAfter> {
    let seconds = std::env::var("ORBIT_CAMERA_AUTO_EXIT_SECONDS")
        .ok()?
        .parse::<f32>()
        .ok()?;
    Some(AutoExitAfter(Timer::from_seconds(
        seconds.max(0.1),
        TimerMode::Once,
    )))
}

fn auto_exit_after(
    time: Res<Time>,
    mut timer: ResMut<AutoExitAfter>,
    mut exit: MessageWriter<AppExit>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        exit.write(AppExit::Success);
    }
}
