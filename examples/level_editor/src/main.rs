//! # Orbit Camera — Level Editor Example
//!
//! Demonstrates a level-editor style camera with focus bounds, dolly zoom,
//! keyboard preset views, and bounded panning.

use saddle_camera_orbit_camera_example_common as common;

use bevy::{
    camera::{PerspectiveProjection, Projection},
    prelude::*,
};
use saddle_camera_orbit_camera::{
    OrbitCamera, OrbitCameraDollyZoom, OrbitCameraFocusBounds, OrbitCameraPlugin,
    OrbitCameraPresetView, OrbitCameraSettings,
};

#[derive(Component)]
struct DollyZoomLabel;

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "orbit_camera level editor".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        OrbitCameraPlugin::default(),
    ));
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, handle_keyboard);
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
        "Level Editor Camera",
        "1-6: preset views (Front/Back/Left/Right/Top/Bottom)\n\
         H: reset to home view\n\
         D: toggle dolly zoom effect\n\
         Focus is bounded to a 20-unit cuboid\n\
         Left drag: orbit  Middle drag: pan  Wheel: zoom",
        Color::srgb(0.42, 0.72, 0.88),
    );

    // Scatter some editor-style props
    let cube_mesh = meshes.add(Cuboid::new(1.4, 1.4, 1.4));
    let colors = [
        Color::srgb(0.82, 0.38, 0.28),
        Color::srgb(0.28, 0.72, 0.42),
        Color::srgb(0.38, 0.48, 0.88),
        Color::srgb(0.88, 0.78, 0.28),
    ];
    for (i, &color) in colors.iter().enumerate() {
        let angle = i as f32 * std::f32::consts::TAU / colors.len() as f32;
        let pos = Vec3::new(angle.cos() * 5.0, 0.7, angle.sin() * 5.0);
        commands.spawn((
            Name::new(format!("Editor Prop {}", i + 1)),
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.45,
                ..default()
            })),
            Transform::from_translation(pos),
        ));
    }

    let mut settings = OrbitCameraSettings::default();
    settings.focus_bounds = Some(OrbitCameraFocusBounds::Cuboid {
        min: Vec3::new(-10.0, -2.0, -10.0),
        max: Vec3::new(10.0, 12.0, 10.0),
    });
    settings.mouse.zoom_to_cursor = true;

    let orbit = OrbitCamera::looking_at(common::DEFAULT_FOCUS, Vec3::new(-8.0, 6.0, 10.0));

    let camera_entity = common::spawn_orbit_camera(
        &mut commands,
        "Level Editor Camera",
        orbit.clone(),
        settings.clone(),
        Projection::Perspective(PerspectiveProjection::default()),
        true,
    );
    common::queue_example_pane(
        &mut commands,
        common::ExampleOrbitPane::from_setup(&orbit, &settings),
    );

    // Add dolly zoom component (starts disabled)
    commands
        .entity(camera_entity)
        .insert(OrbitCameraDollyZoom::default());

    commands.spawn((
        Name::new("Dolly Zoom Label"),
        DollyZoomLabel,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(18.0),
            top: Val::Px(18.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.78)),
        Text::new("Dolly Zoom: OFF"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn handle_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut cameras: Query<(&mut OrbitCamera, Option<&mut OrbitCameraDollyZoom>)>,
    mut labels: Query<&mut Text, With<DollyZoomLabel>>,
) {
    let presets = [
        (KeyCode::Digit1, OrbitCameraPresetView::Front),
        (KeyCode::Digit2, OrbitCameraPresetView::Back),
        (KeyCode::Digit3, OrbitCameraPresetView::Left),
        (KeyCode::Digit4, OrbitCameraPresetView::Right),
        (KeyCode::Digit5, OrbitCameraPresetView::Top),
        (KeyCode::Digit6, OrbitCameraPresetView::Bottom),
    ];

    for (mut orbit, dolly) in &mut cameras {
        for &(key, preset) in &presets {
            if keys.just_pressed(key) {
                orbit.set_preset_view(preset);
            }
        }

        if keys.just_pressed(KeyCode::KeyH) {
            orbit.reset_to_home();
        }

        if keys.just_pressed(KeyCode::KeyD) {
            if let Some(mut dolly) = dolly {
                dolly.enabled = !dolly.enabled;
                if let Ok(mut label) = labels.single_mut() {
                    label.0 = format!(
                        "Dolly Zoom: {}",
                        if dolly.enabled { "ON" } else { "OFF" }
                    );
                }
            }
        }
    }
}
