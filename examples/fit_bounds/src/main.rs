use saddle_camera_orbit_camera_example_common as common;

use bevy::{
    camera::{PerspectiveProjection, Projection},
    prelude::*,
};
use saddle_camera_orbit_camera::{OrbitCamera, OrbitCameraPlugin};

#[derive(Component)]
struct FocusLabel;

#[derive(Resource)]
struct FocusCycle {
    camera: Entity,
    index: usize,
    timer: Timer,
    selections: Vec<SelectionBounds>,
}

#[derive(Clone, Copy)]
struct SelectionBounds {
    center: Vec3,
    half_extents: Vec3,
    label: &'static str,
}

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "orbit_camera fit bounds".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        OrbitCameraPlugin::default(),
    ));
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, cycle_selection);
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
        "Fit Bounds",
        "This example cycles between authored bounds and uses the public framing helpers.\nThe controller keeps its current yaw and pitch while fitting the new selection.",
        Color::srgb(0.34, 0.58, 0.90),
    );

    let selections = vec![
        SelectionBounds {
            center: Vec3::new(-6.0, 1.2, 0.0),
            half_extents: Vec3::new(1.0, 1.0, 1.0),
            label: "Compact prop",
        },
        SelectionBounds {
            center: Vec3::new(0.0, 2.8, 0.0),
            half_extents: Vec3::new(1.8, 2.8, 1.8),
            label: "Tall statue",
        },
        SelectionBounds {
            center: Vec3::new(7.0, 0.9, 2.5),
            half_extents: Vec3::new(3.5, 0.9, 2.2),
            label: "Wide assembly",
        },
    ];

    let shapes = [
        (
            "Compact prop",
            Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
            Vec3::new(-6.0, 1.2, 0.0),
            Color::srgb(0.84, 0.56, 0.20),
        ),
        (
            "Tall statue",
            Mesh3d(meshes.add(Capsule3d::new(1.1, 3.4).mesh().rings(8).latitudes(12))),
            Vec3::new(0.0, 2.8, 0.0),
            Color::srgb(0.28, 0.70, 0.84),
        ),
        (
            "Wide assembly",
            Mesh3d(meshes.add(Cuboid::new(7.0, 1.8, 4.4))),
            Vec3::new(7.0, 0.9, 2.5),
            Color::srgb(0.42, 0.74, 0.40),
        ),
    ];

    for (name, mesh, translation, color) in shapes {
        commands.spawn((
            Name::new(name),
            mesh,
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.42,
                ..default()
            })),
            Transform::from_translation(translation),
        ));
    }

    let settings = saddle_camera_orbit_camera::OrbitCameraSettings::default();
    let orbit = OrbitCamera::looking_at(common::DEFAULT_FOCUS, Vec3::new(-10.0, 6.0, 12.0));

    let camera = common::spawn_orbit_camera(
        &mut commands,
        "Fit Bounds Camera",
        orbit.clone(),
        settings.clone(),
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );
    common::queue_example_pane(
        &mut commands,
        common::ExampleOrbitPane::from_setup(&orbit, &settings),
    );

    commands.insert_resource(FocusCycle {
        camera,
        index: 0,
        timer: Timer::from_seconds(2.6, TimerMode::Repeating),
        selections,
    });
    commands.spawn((
        Name::new("Focus Label"),
        FocusLabel,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(18.0),
            top: Val::Px(18.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.78)),
        Text::new("Selection: Compact prop"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn cycle_selection(
    time: Res<Time>,
    mut cycle: ResMut<FocusCycle>,
    mut cameras: Query<(&Projection, &mut OrbitCamera)>,
    mut labels: Query<&mut Text, With<FocusLabel>>,
) {
    if !cycle.timer.tick(time.delta()).just_finished() {
        return;
    }

    cycle.index = (cycle.index + 1) % cycle.selections.len();
    let selection = cycle.selections[cycle.index];

    let Ok((projection, mut camera)) = cameras.get_mut(cycle.camera) else {
        return;
    };
    camera.frame_aabb(projection, selection.center, selection.half_extents, 1.25);

    if let Ok(mut label) = labels.single_mut() {
        label.0 = format!("Selection: {}", selection.label);
    }
}
