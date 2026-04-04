//! # Orbit Camera — Basic Example
//!
//! Simplest orbit camera: left-drag orbit, middle-drag pan, wheel zoom,
//! with idle auto-rotate. Fully self-contained — copy into your own project.

use bevy::{
    camera::{PerspectiveProjection, Projection},
    light::GlobalAmbientLight,
    prelude::*,
};
use bevy_flair::prelude::InlineStyle;
use saddle_camera_orbit_camera::{
    OrbitCamera, OrbitCameraAutoRotate, OrbitCameraInputTarget, OrbitCameraPlugin,
    OrbitCameraSettings, OrbitCameraSystems,
};
use saddle_pane::prelude::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DEFAULT_FOCUS: Vec3 = Vec3::new(0.0, 1.2, 0.0);

// ---------------------------------------------------------------------------
// Pane — live-editable orbit camera parameters
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, Copy, PartialEq, Pane)]
#[pane(title = "Orbit Camera", position = "top-right")]
struct OrbitPane {
    #[pane(slider, min = 0.001, max = 0.03, step = 0.001)]
    orbit_sensitivity: f32,
    #[pane(slider, min = 0.2, max = 3.0, step = 0.05)]
    pan_sensitivity: f32,
    #[pane(slider, min = 0.02, max = 0.4, step = 0.01)]
    wheel_zoom_sensitivity: f32,
    #[pane(toggle)]
    zoom_to_cursor: bool,
    #[pane(toggle)]
    auto_rotate_enabled: bool,
    #[pane(slider, min = 0.0, max = 6.0, step = 0.1)]
    auto_rotate_wait_seconds: f32,
    #[pane(slider, min = 0.0, max = 1.5, step = 0.01)]
    auto_rotate_speed: f32,
    #[pane(slider, min = 0.5, max = 80.0, step = 0.1)]
    distance: f32,
    #[pane(slider, min = 0.1, max = 80.0, step = 0.1)]
    min_distance: f32,
    #[pane(slider, min = 1.0, max = 250.0, step = 0.5)]
    max_distance: f32,
}

impl Default for OrbitPane {
    fn default() -> Self {
        Self {
            orbit_sensitivity: 0.008,
            pan_sensitivity: 1.0,
            wheel_zoom_sensitivity: 0.14,
            zoom_to_cursor: false,
            auto_rotate_enabled: true,
            auto_rotate_wait_seconds: 1.5,
            auto_rotate_speed: 0.22,
            distance: 12.0,
            min_distance: 0.5,
            max_distance: 250.0,
        }
    }
}

// Dark theme CSS variables for the pane.
const PANE_DARK_THEME_VARS: &[(&str, &str)] = &[
    ("--pane-elevation-1", "#28292e"),
    ("--pane-elevation-2", "#222327"),
    ("--pane-elevation-3", "rgba(187, 188, 196, 0.10)"),
    ("--pane-border", "#3c3d44"),
    ("--pane-border-focus", "#7090b0"),
    ("--pane-border-subtle", "#333438"),
    ("--pane-text-primary", "#bbbcc4"),
    ("--pane-text-secondary", "#78797f"),
    ("--pane-text-muted", "#5c5d64"),
    ("--pane-text-on-accent", "#ffffff"),
    ("--pane-text-brighter", "#d0d1d8"),
    ("--pane-text-monitor", "#9a9ba2"),
    ("--pane-text-log", "#8a8b92"),
    ("--pane-accent", "#4a6fa5"),
    ("--pane-accent-hover", "#5a8fd5"),
    ("--pane-accent-active", "#3a5f95"),
    ("--pane-accent-subtle", "rgba(74, 111, 165, 0.15)"),
    ("--pane-accent-fill", "rgba(74, 111, 165, 0.60)"),
    ("--pane-accent-fill-hover", "rgba(90, 143, 213, 0.70)"),
    ("--pane-accent-fill-active", "rgba(90, 143, 213, 0.80)"),
    ("--pane-accent-checked", "rgba(74, 111, 165, 0.25)"),
    ("--pane-accent-checked-hover", "rgba(74, 111, 165, 0.35)"),
    ("--pane-accent-indicator", "rgba(74, 111, 165, 0.80)"),
    ("--pane-accent-knob", "#7aacdf"),
    ("--pane-widget-bg", "rgba(187, 188, 196, 0.10)"),
    ("--pane-widget-hover", "rgba(187, 188, 196, 0.15)"),
    ("--pane-widget-focus", "rgba(187, 188, 196, 0.20)"),
    ("--pane-widget-active", "rgba(187, 188, 196, 0.25)"),
    ("--pane-widget-bg-muted", "rgba(187, 188, 196, 0.06)"),
    ("--pane-tab-hover-bg", "rgba(187, 188, 196, 0.06)"),
    ("--pane-hover-bg", "rgba(255, 255, 255, 0.03)"),
    ("--pane-active-bg", "rgba(255, 255, 255, 0.05)"),
    ("--pane-popup-bg", "#1e1f24"),
    ("--pane-bg-dark", "rgba(0, 0, 0, 0.25)"),
];

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.07)));
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
        // Pane UI stack
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        PanePlugin,
    ))
    .register_pane::<OrbitPane>()
    .add_systems(Startup, setup)
    .add_systems(PreUpdate, prime_pane_theme)
    .add_systems(
        Update,
        (
            sync_pane_to_camera.after(OrbitCameraSystems::ApplyIntent),
            reflect_camera_to_pane.after(OrbitCameraSystems::ApplyIntent),
        ),
    );
    app.run();
}

// ---------------------------------------------------------------------------
// Setup — camera, scene, overlay
// ---------------------------------------------------------------------------

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // -- Reference world --
    spawn_reference_scene(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Orbit Camera",
        "Left drag: orbit\nMiddle drag: pan\nWheel: zoom\n\
        Idle auto-rotate resumes after a short pause.",
        Color::srgb(0.88, 0.56, 0.20),
    );

    // -- Orbit camera with auto-rotate enabled --
    let settings = OrbitCameraSettings {
        auto_rotate: OrbitCameraAutoRotate {
            enabled: true,
            wait_seconds: 1.5,
            speed: 0.22,
        },
        ..OrbitCameraSettings::default()
    };
    let orbit = OrbitCamera::looking_at(DEFAULT_FOCUS, Vec3::new(-7.5, 5.2, 10.0));

    commands.spawn((
        Name::new("Basic Orbit Camera"),
        orbit.clone(),
        settings.clone(),
        Projection::Perspective(PerspectiveProjection::default()),
        OrbitCameraInputTarget,
    ));

    // Seed the pane with initial values from the camera.
    commands.insert_resource(OrbitPane {
        orbit_sensitivity: settings.mouse.orbit_sensitivity.x,
        pan_sensitivity: settings.mouse.pan_sensitivity,
        wheel_zoom_sensitivity: settings.mouse.wheel_zoom_sensitivity,
        zoom_to_cursor: settings.mouse.zoom_to_cursor,
        auto_rotate_enabled: settings.auto_rotate.enabled,
        auto_rotate_wait_seconds: settings.auto_rotate.wait_seconds,
        auto_rotate_speed: settings.auto_rotate.speed,
        distance: orbit.target_distance,
        min_distance: settings.zoom_limits.min_distance,
        max_distance: settings.zoom_limits.max_distance,
    });
}

// ---------------------------------------------------------------------------
// Reference scene — ground, plinth, accent sphere, pillars, overlay
// ---------------------------------------------------------------------------

fn spawn_reference_scene(
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
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 32_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.7, 0.0)),
    ));

    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(48.0, 48.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.13, 0.16),
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));

    commands.spawn((
        Name::new("Plinth"),
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
        Name::new("Accent Sphere"),
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
    for (idx, (x, z)) in [(-6.0, -6.0), (6.0, -6.0), (-6.0, 6.0), (6.0, 6.0)]
        .into_iter()
        .enumerate()
    {
        commands.spawn((
            Name::new(format!("Pillar {}", idx + 1)),
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
        Name::new("Overlay"),
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

// ---------------------------------------------------------------------------
// Pane dark theme
// ---------------------------------------------------------------------------

fn prime_pane_theme(mut panes: Query<&mut InlineStyle, Added<PaneRoot>>) {
    for mut style in &mut panes {
        for &(key, value) in PANE_DARK_THEME_VARS {
            style.set(key, value.to_owned());
        }
    }
}

// ---------------------------------------------------------------------------
// Pane <-> camera sync — push pane edits into settings, pull runtime back
// ---------------------------------------------------------------------------

fn sync_pane_to_camera(
    pane: Res<OrbitPane>,
    mut cameras: Query<(&mut OrbitCamera, &mut OrbitCameraSettings)>,
) {
    if !pane.is_changed() {
        return;
    }

    for (mut orbit, mut settings) in &mut cameras {
        let min_distance = pane.min_distance.max(0.1);
        let max_distance = pane.max_distance.max(min_distance + 0.1);

        settings.mouse.orbit_sensitivity = Vec2::splat(pane.orbit_sensitivity);
        settings.mouse.pan_sensitivity = pane.pan_sensitivity;
        settings.mouse.wheel_zoom_sensitivity = pane.wheel_zoom_sensitivity;
        settings.mouse.zoom_to_cursor = pane.zoom_to_cursor;
        settings.auto_rotate.enabled = pane.auto_rotate_enabled;
        settings.auto_rotate.wait_seconds = pane.auto_rotate_wait_seconds;
        settings.auto_rotate.speed = pane.auto_rotate_speed;
        settings.zoom_limits.min_distance = min_distance;
        settings.zoom_limits.max_distance = max_distance;

        orbit.target_distance = pane.distance.clamp(min_distance, max_distance);
    }
}

fn reflect_camera_to_pane(
    mut pane: ResMut<OrbitPane>,
    cameras: Query<(&OrbitCamera, &OrbitCameraSettings)>,
) {
    let Some((orbit, settings)) = cameras.iter().next() else {
        return;
    };
    pane.zoom_to_cursor = settings.mouse.zoom_to_cursor;
    pane.distance = orbit.target_distance;
}
