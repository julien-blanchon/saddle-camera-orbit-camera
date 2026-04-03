use saddle_camera_orbit_camera_example_common as common;

use bevy::{
    camera::{PerspectiveProjection, Projection},
    prelude::*,
};
use saddle_camera_orbit_camera::{
    OrbitCamera, OrbitCameraPlugin, OrbitCameraPresetView, OrbitCameraSettings,
};

#[derive(Component)]
struct ViewLabel;

#[derive(Resource)]
struct ViewCycle {
    camera: Entity,
    index: usize,
    timer: Timer,
}

const PRESET_VIEWS: &[(OrbitCameraPresetView, &str)] = &[
    (OrbitCameraPresetView::Front, "Front"),
    (OrbitCameraPresetView::Right, "Right"),
    (OrbitCameraPresetView::Top, "Top"),
    (OrbitCameraPresetView::Left, "Left"),
    (OrbitCameraPresetView::Back, "Back"),
];

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "orbit_camera preset views".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        OrbitCameraPlugin::default(),
    ));
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, cycle_views);
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
        "Preset Views",
        "The camera snaps through authored axis-aligned views.\nThis is useful for model viewers, editor-style inspectors, and CAD-inspired navigation.",
        Color::srgb(0.32, 0.72, 0.96),
    );

    commands.spawn((
        Name::new("Hero Bust"),
        Mesh3d(meshes.add(Capsule3d::new(1.0, 2.8).mesh().rings(10).latitudes(14))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.92, 0.72, 0.22),
            perceptual_roughness: 0.32,
            ..default()
        })),
        Transform::from_xyz(0.0, 2.0, 0.0),
    ));

    let settings = OrbitCameraSettings::default();
    let orbit = OrbitCamera::looking_at(common::DEFAULT_FOCUS, Vec3::new(0.0, 6.5, 11.0));
    let camera = common::spawn_orbit_camera(
        &mut commands,
        "Preset View Camera",
        orbit.clone(),
        settings.clone(),
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );
    common::queue_example_pane(
        &mut commands,
        common::ExampleOrbitPane::from_setup(&orbit, &settings),
    );
    commands.insert_resource(ViewCycle {
        camera,
        index: 0,
        timer: Timer::from_seconds(2.0, TimerMode::Repeating),
    });

    commands.spawn((
        Name::new("View Label"),
        ViewLabel,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(18.0),
            top: Val::Px(18.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.78)),
        Text::new("Preset View: Front"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn cycle_views(
    time: Res<Time>,
    mut cycle: ResMut<ViewCycle>,
    mut cameras: Query<&mut OrbitCamera>,
    mut labels: Query<&mut Text, With<ViewLabel>>,
) {
    if !cycle.timer.tick(time.delta()).just_finished() {
        return;
    }

    cycle.index = (cycle.index + 1) % PRESET_VIEWS.len();
    let (preset, label) = PRESET_VIEWS[cycle.index];

    let Ok(mut camera) = cameras.get_mut(cycle.camera) else {
        return;
    };
    camera.set_preset_view(preset);

    if let Ok(mut text) = labels.single_mut() {
        text.0 = format!("Preset View: {label}");
    }
}
