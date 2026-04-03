use bevy::{app::AppExit, camera::Projection, light::GlobalAmbientLight, prelude::*};
use bevy_flair::prelude::InlineStyle;
use saddle_camera_orbit_camera::{
    OrbitCamera, OrbitCameraInputTarget, OrbitCameraSettings, OrbitCameraSystems,
};
use saddle_pane::prelude::*;

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

pub const DEFAULT_FOCUS: Vec3 = Vec3::new(0.0, 1.2, 0.0);

#[derive(Resource)]
struct AutoExitAfter(Timer);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Pane)]
#[pane(title = "Orbit Camera", position = "top-right")]
pub struct ExampleOrbitPane {
    #[pane(slider, min = 0.001, max = 0.03, step = 0.001)]
    pub orbit_sensitivity: f32,
    #[pane(slider, min = 0.2, max = 3.0, step = 0.05)]
    pub pan_sensitivity: f32,
    #[pane(slider, min = 0.02, max = 0.4, step = 0.01)]
    pub wheel_zoom_sensitivity: f32,
    #[pane(toggle)]
    pub zoom_to_cursor: bool,
    #[pane(toggle)]
    pub auto_rotate_enabled: bool,
    #[pane(slider, min = 0.0, max = 6.0, step = 0.1)]
    pub auto_rotate_wait_seconds: f32,
    #[pane(slider, min = 0.0, max = 1.5, step = 0.01)]
    pub auto_rotate_speed: f32,
    #[pane(slider, min = 0.5, max = 80.0, step = 0.1)]
    pub distance: f32,
    #[pane(slider, min = 0.1, max = 12.0, step = 0.05)]
    pub orthographic_scale: f32,
    #[pane(slider, min = 0.1, max = 80.0, step = 0.1)]
    pub min_distance: f32,
    #[pane(slider, min = 1.0, max = 250.0, step = 0.5)]
    pub max_distance: f32,
}

impl Default for ExampleOrbitPane {
    fn default() -> Self {
        Self {
            orbit_sensitivity: 0.008,
            pan_sensitivity: 1.0,
            wheel_zoom_sensitivity: 0.14,
            zoom_to_cursor: false,
            auto_rotate_enabled: false,
            auto_rotate_wait_seconds: 2.0,
            auto_rotate_speed: 0.45,
            distance: 12.0,
            orthographic_scale: 1.0,
            min_distance: 0.5,
            max_distance: 250.0,
        }
    }
}

impl ExampleOrbitPane {
    pub fn from_setup(orbit: &OrbitCamera, settings: &OrbitCameraSettings) -> Self {
        Self {
            orbit_sensitivity: settings.mouse.orbit_sensitivity.x,
            pan_sensitivity: settings.mouse.pan_sensitivity,
            wheel_zoom_sensitivity: settings.mouse.wheel_zoom_sensitivity,
            zoom_to_cursor: settings.mouse.zoom_to_cursor,
            auto_rotate_enabled: settings.auto_rotate.enabled,
            auto_rotate_wait_seconds: settings.auto_rotate.wait_seconds,
            auto_rotate_speed: settings.auto_rotate.speed,
            distance: orbit.target_distance,
            orthographic_scale: orbit.target_orthographic_scale,
            min_distance: settings.zoom_limits.min_distance,
            max_distance: settings.zoom_limits.max_distance,
        }
    }
}

#[derive(Resource, Clone, Copy)]
struct ExampleOrbitPaneBootstrap(ExampleOrbitPane);

pub fn queue_example_pane(commands: &mut Commands, pane: ExampleOrbitPane) {
    commands.insert_resource(ExampleOrbitPaneBootstrap(pane));
}

pub fn apply_example_defaults(app: &mut App) {
    app.insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.07)));

    if let Some(timer) = auto_exit_from_env() {
        app.insert_resource(timer);
        app.add_systems(Update, auto_exit_after);
    }
}

pub fn install_pane(app: &mut App) {
    app.add_plugins((
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        PanePlugin,
    ))
    .register_pane::<ExampleOrbitPane>()
    .add_systems(
        PreUpdate,
        (
            prime_pane_theme_vars,
            apply_bootstrapped_pane,
            sync_example_pane,
        )
            .chain(),
    )
    .add_systems(
        Update,
        reflect_example_pane.after(OrbitCameraSystems::ApplyIntent),
    );
}

fn prime_pane_theme_vars(mut panes: Query<&mut InlineStyle, Added<PaneRoot>>) {
    for mut style in &mut panes {
        for &(key, value) in PANE_DARK_THEME_VARS {
            style.set(key, value.to_owned());
        }
    }
}

fn apply_bootstrapped_pane(
    bootstrap: Option<Res<ExampleOrbitPaneBootstrap>>,
    mut pane: ResMut<ExampleOrbitPane>,
) {
    let Some(bootstrap) = bootstrap else {
        return;
    };

    if *pane == ExampleOrbitPane::default() {
        *pane = bootstrap.0;
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

fn sync_example_pane(
    mut pane: ResMut<ExampleOrbitPane>,
    bootstrap: Option<Res<ExampleOrbitPaneBootstrap>>,
    mut cameras: Query<(&mut OrbitCamera, &mut OrbitCameraSettings)>,
) {
    let has_bootstrap = bootstrap.is_some();
    if let Some(bootstrap) = bootstrap {
        if *pane == ExampleOrbitPane::default() && bootstrap.0 != *pane {
            *pane = bootstrap.0;
        }
    }

    for (mut orbit, mut settings) in &mut cameras {
        let scene_pane = ExampleOrbitPane::from_setup(&orbit, &settings);
        if !has_bootstrap && *pane == ExampleOrbitPane::default() && scene_pane != *pane {
            *pane = scene_pane;
            return;
        }

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
        orbit.target_orthographic_scale = pane.orthographic_scale.clamp(
            settings.zoom_limits.min_orthographic_scale,
            settings.zoom_limits.max_orthographic_scale,
        );
    }
}

fn reflect_example_pane(
    mut pane: ResMut<ExampleOrbitPane>,
    cameras: Query<(&OrbitCamera, &OrbitCameraSettings)>,
) {
    let Some((orbit, settings)) = cameras.iter().next() else {
        return;
    };

    pane.zoom_to_cursor = settings.mouse.zoom_to_cursor;
    pane.distance = orbit.target_distance;
    pane.orthographic_scale = orbit.target_orthographic_scale;
}
