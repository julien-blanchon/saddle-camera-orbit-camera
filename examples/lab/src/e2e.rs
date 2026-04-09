use bevy::{
    camera::{OrthographicProjection, ScalingMode},
    prelude::*,
    window::PrimaryWindow,
};
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet,
    action::Action,
    actions::{assertions, inspect},
    init_scenario,
    scenario::Scenario,
};
use saddle_camera_orbit_camera::{
    OrbitCamera, OrbitCameraFocusBounds, OrbitCameraFollow, OrbitCameraInputTarget,
    OrbitCameraPresetView, OrbitCameraSettings, OrbitCameraSystems,
};

use crate::{LabCameraEntity, LabTargetEntity};

pub struct OrbitCameraLabE2EPlugin;

impl Plugin for OrbitCameraLabE2EPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(E2EPlugin);
        app.configure_sets(Update, E2ESet.before(OrbitCameraSystems::ReadInput));

        let args: Vec<String> = std::env::args().collect();
        let (scenario_name, handoff) = parse_e2e_args(&args);

        if let Some(name) = scenario_name {
            if let Some(mut scenario) = scenario_by_name(&name) {
                if handoff {
                    scenario.actions.push(Action::Handoff);
                }
                init_scenario(app, scenario);
            } else {
                error!(
                    "[orbit_camera_lab:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn parse_e2e_args(args: &[String]) -> (Option<String>, bool) {
    let mut scenario_name = None;
    let mut handoff = false;

    for arg in args.iter().skip(1) {
        if arg == "--handoff" {
            handoff = true;
        } else if !arg.starts_with('-') && scenario_name.is_none() {
            scenario_name = Some(arg.clone());
        }
    }

    if !handoff {
        handoff = std::env::var("E2E_HANDOFF").is_ok_and(|value| value == "1" || value == "true");
    }

    (scenario_name, handoff)
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "orbit_camera_smoke" => Some(build_smoke()),
        "orbit_camera_input" => Some(build_input()),
        "orbit_camera_follow_target" => Some(build_follow_target()),
        "orbit_camera_preset_views" => Some(build_preset_views()),
        "orbit_camera_zoom_to_cursor" => Some(build_zoom_to_cursor()),
        "snap_orbit_camera_ortho" => Some(build_orthographic_snapshot()),
        "orbit_camera_focus_bounds" => Some(build_focus_bounds()),
        "orbit_camera_force_update" => Some(build_force_update()),
        "orbit_camera_auto_rotate" => Some(build_auto_rotate()),
        "orbit_camera_inertia" => Some(build_inertia()),
        _ => None,
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec![
        "orbit_camera_smoke",
        "orbit_camera_input",
        "orbit_camera_follow_target",
        "orbit_camera_preset_views",
        "orbit_camera_zoom_to_cursor",
        "snap_orbit_camera_ortho",
        "orbit_camera_focus_bounds",
        "orbit_camera_force_update",
        "orbit_camera_auto_rotate",
        "orbit_camera_inertia",
    ]
}

fn camera_entity(world: &World) -> Entity {
    world.resource::<LabCameraEntity>().0
}

fn target_entity(world: &World) -> Entity {
    world.resource::<LabTargetEntity>().0
}

fn camera_state(world: &World) -> OrbitCamera {
    world
        .get::<OrbitCamera>(camera_entity(world))
        .expect("orbit camera should exist")
        .clone()
}

fn set_cursor_position(world: &mut World, logical_position: Vec2) {
    let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
    let Ok(mut window) = windows.single_mut(world) else {
        return;
    };
    window.set_cursor_position(Some(logical_position));
}

fn with_camera_settings(world: &mut World, update: impl FnOnce(&mut OrbitCameraSettings)) {
    let entity = camera_entity(world);
    if let Some(mut settings) = world.get_mut::<OrbitCameraSettings>(entity) {
        update(&mut settings);
    }
}

fn with_camera(world: &mut World, update: impl FnOnce(&mut OrbitCamera)) {
    let entity = camera_entity(world);
    if let Some(mut camera) = world.get_mut::<OrbitCamera>(entity) {
        update(&mut camera);
    }
}

fn enable_zoom_to_cursor(world: &mut World) {
    with_camera_settings(world, |settings| {
        settings.mouse.zoom_to_cursor = true;
    });
}

fn set_preset_view(world: &mut World, preset: OrbitCameraPresetView) {
    with_camera(world, |camera| camera.set_preset_view(preset));
}

fn set_focus_bounds(world: &mut World, bounds: OrbitCameraFocusBounds) {
    with_camera_settings(world, |settings| {
        settings.focus_bounds = Some(bounds);
    });
}

fn enable_force_update(world: &mut World) {
    with_camera_settings(world, |settings| {
        settings.enabled = false;
        settings.force_update = true;
    });
}

fn enable_auto_rotate(world: &mut World, wait_seconds: f32, speed: f32) {
    with_camera_settings(world, |settings| {
        settings.auto_rotate = saddle_camera_orbit_camera::OrbitCameraAutoRotate {
            enabled: true,
            wait_seconds,
            speed,
        };
    });
}

fn disable_auto_rotate(world: &mut World) {
    with_camera_settings(world, |settings| {
        settings.auto_rotate.enabled = false;
    });
}

fn enable_inertia(world: &mut World, orbit_friction: f32, pan_friction: f32, zoom_friction: f32) {
    with_camera_settings(world, |settings| {
        settings.inertia = saddle_camera_orbit_camera::OrbitCameraInertia {
            enabled: true,
            orbit_friction,
            pan_friction,
            zoom_friction,
        };
    });
}

fn configure_orthographic_snapshot(
    world: &mut World,
    scale: f32,
    focus: Vec3,
    yaw: f32,
    pitch: f32,
) {
    let entity = camera_entity(world);
    world.entity_mut(entity).insert(Projection::Orthographic(
        OrthographicProjection {
            scale,
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 16.0,
            },
            ..OrthographicProjection::default_3d()
        },
    ));
    with_camera(world, |camera| {
        camera.target_focus = focus;
        camera.target_yaw = yaw;
        camera.target_pitch = pitch;
        camera.target_orthographic_scale = 1.0;
    });
}

fn build_smoke() -> Scenario {
    Scenario::builder("orbit_camera_smoke")
        .description(
            "Boot the lab, verify the orbit camera and input marker exist, then capture a readable baseline screenshot.",
        )
        .then(Action::WaitFrames(90))
        .then(assertions::entity_exists::<OrbitCamera>("camera entity exists"))
        .then(assertions::entity_exists::<OrbitCameraInputTarget>(
            "input target marker exists",
        ))
        .then(assertions::custom("focus starts near the authored scene center", |world| {
            let orbit = camera_state(world);
            orbit.focus.distance(common_focus()) < 0.5
        }))
        .then(assertions::log_summary("orbit_camera_smoke summary"))
        .then(inspect::dump_component_json::<OrbitCamera>("orbit_camera_smoke_state"))
        .then(Action::Screenshot("orbit_camera_smoke".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_input() -> Scenario {
    Scenario::builder("orbit_camera_input")
        .description(
            "Use the real mouse-driven path to orbit, pan, and zoom, with hard checks after each input family.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Screenshot("orbit_camera_input_before".into()))
        .then(Action::WaitFrames(1))
        .then(Action::PressMouseButton(MouseButton::Left))
        .then(Action::MouseMotion {
            delta: Vec2::new(180.0, -90.0),
        })
        .then(Action::WaitFrames(2))
        .then(Action::ReleaseMouseButton(MouseButton::Left))
        .then(assertions::custom("orbit drag changes yaw and pitch", |world| {
            let orbit = camera_state(world);
            (orbit.target_yaw - orbit.home.yaw).abs() > 0.2
                && (orbit.target_pitch - orbit.home.pitch).abs() > 0.08
        }))
        .then(Action::PressMouseButton(MouseButton::Middle))
        .then(Action::MouseMotion {
            delta: Vec2::new(90.0, 36.0),
        })
        .then(Action::WaitFrames(2))
        .then(Action::ReleaseMouseButton(MouseButton::Middle))
        .then(assertions::custom("pan drag changes focus", |world| {
            let orbit = camera_state(world);
            orbit.target_focus.distance(orbit.home.focus) > 0.2
        }))
        .then(Action::MouseScroll {
            delta: Vec2::new(0.0, 3.0),
        })
        .then(Action::WaitFrames(2))
        .then(assertions::custom("scroll zooms closer in perspective", |world| {
            let orbit = camera_state(world);
            orbit.target_distance < orbit.home.distance
        }))
        .then(assertions::log_summary("orbit_camera_input summary"))
        .then(Action::Screenshot("orbit_camera_input_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_follow_target() -> Scenario {
    Scenario::builder("orbit_camera_follow_target")
        .description(
            "Attach the follow component, let the target move, and verify the focus tracks it while capturing before and after frames.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Screenshot("orbit_camera_follow_before".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            let target = target_entity(world);
            world
                .entity_mut(camera_entity(world))
                .insert(OrbitCameraFollow {
                    target,
                    offset: Vec3::new(0.0, 0.6, 0.0),
                    enabled: true,
                });
        })))
        .then(Action::WaitFrames(90))
        .then(assertions::custom("camera focus tracks the moving target", |world| {
            let orbit = camera_state(world);
            let target_translation = world
                .get::<Transform>(target_entity(world))
                .expect("target transform exists")
                .translation;
            orbit.target_focus.distance(target_translation + Vec3::new(0.0, 0.6, 0.0)) < 0.2
        }))
        .then(assertions::log_summary("orbit_camera_follow_target summary"))
        .then(Action::Screenshot("orbit_camera_follow_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_preset_views() -> Scenario {
    Scenario::builder("orbit_camera_preset_views")
        .description(
            "Jump the shared orbit camera through the new preset-view helpers and verify the authored target angles land on the expected axes.",
        )
        .then(Action::WaitFrames(30))
        .then(Action::Screenshot("orbit_camera_preset_views_before".into()))
        .then(Action::Custom(Box::new(|world| {
            set_preset_view(world, OrbitCameraPresetView::Top);
        })))
        .then(Action::WaitFrames(10))
        .then(assertions::custom("top preset drives pitch toward overhead", |world| {
            let orbit = camera_state(world);
            orbit.target_pitch > 1.4 && orbit.pitch > 0.8
        }))
        .then(Action::Custom(Box::new(|world| {
            set_preset_view(world, OrbitCameraPresetView::Right);
        })))
        .then(Action::WaitFrames(12))
        .then(assertions::custom("right preset drives yaw onto the side view", |world| {
            let orbit = camera_state(world);
            (orbit.target_yaw - std::f32::consts::FRAC_PI_2).abs() < 0.01
                && orbit.target_pitch.abs() < 0.05
        }))
        .then(assertions::log_summary("orbit_camera_preset_views summary"))
        .then(Action::Screenshot("orbit_camera_preset_views_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_zoom_to_cursor() -> Scenario {
    Scenario::builder("orbit_camera_zoom_to_cursor")
        .description(
            "Enable cursor-aware zoom, place the cursor away from center, and assert a wheel zoom changes both distance and focus instead of only dollying straight in.",
        )
        .then(Action::WaitFrames(30))
        .then(Action::Custom(Box::new(|world| {
            enable_zoom_to_cursor(world);
            set_cursor_position(world, Vec2::new(1120.0, 260.0));
        })))
        .then(Action::Screenshot("orbit_camera_zoom_to_cursor_before".into()))
        .then(Action::WaitFrames(4))
        .then(Action::MouseScroll {
            delta: Vec2::new(0.0, 4.0),
        })
        .then(Action::WaitFrames(8))
        .then(assertions::custom(
            "zoom-to-cursor moves focus as well as distance",
            |world| {
                let orbit = camera_state(world);
                orbit.target_distance < orbit.home.distance
                    && orbit.target_focus.distance(orbit.home.focus) > 0.25
            },
        ))
        .then(assertions::log_summary("orbit_camera_zoom_to_cursor summary"))
        .then(Action::Screenshot("orbit_camera_zoom_to_cursor_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_orthographic_snapshot() -> Scenario {
    Scenario::builder("snap_orbit_camera_ortho")
        .description(
            "Switch the lab camera into an orthographic overview, assert the shared component and projection stay aligned, then capture a snapshot.",
        )
        .then(Action::WaitFrames(30))
        .then(Action::Custom(Box::new(|world| {
            configure_orthographic_snapshot(world, 1.25, Vec3::new(8.5, 0.8, -6.0), 0.75, -1.0);
        })))
        .then(Action::WaitFrames(10))
        .then(assertions::custom("orthographic projection scale matches camera state", |world| {
            let entity = camera_entity(world);
            let orbit = world
                .get::<OrbitCamera>(entity)
                .expect("orbit camera should exist");
            let projection = world
                .get::<Projection>(entity)
                .expect("projection should exist");
            let Projection::Orthographic(orthographic) = projection else {
                return false;
            };
            (orthographic.scale - orbit.orthographic_scale).abs() < 0.05
        }))
        .then(assertions::log_summary("snap_orbit_camera_ortho summary"))
        .then(Action::Screenshot("snap_orbit_camera_ortho".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_focus_bounds() -> Scenario {
    Scenario::builder("orbit_camera_focus_bounds")
        .description(
            "Enable focus bounds on the camera, push the target focus far outside, and verify the advance_state system clamps it within the authored boundary.",
        )
        .then(Action::WaitFrames(30))
        .then(Action::Custom(Box::new(|world| {
            set_focus_bounds(
                world,
                OrbitCameraFocusBounds::Cuboid {
                    min: Vec3::new(-5.0, -5.0, -5.0),
                    max: Vec3::new(5.0, 5.0, 5.0),
                },
            );
            with_camera(world, |camera| {
                camera.target_focus = Vec3::new(50.0, 50.0, 50.0);
            });
        })))
        .then(Action::WaitFrames(4))
        .then(assertions::custom("focus is clamped within cuboid bounds", |world| {
            let orbit = camera_state(world);
            orbit.target_focus.x <= 5.0
                && orbit.target_focus.y <= 5.0
                && orbit.target_focus.z <= 5.0
        }))
        .then(assertions::log_summary("orbit_camera_focus_bounds summary"))
        .then(Action::Screenshot("orbit_camera_focus_bounds".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_force_update() -> Scenario {
    Scenario::builder("orbit_camera_force_update")
        .description(
            "Disable the camera, set force_update=true, move the target, and verify the camera still advances toward its target state for one frame.",
        )
        .then(Action::WaitFrames(30))
        .then(Action::Custom(Box::new(|world| {
            enable_force_update(world);
            with_camera(world, |camera| {
                camera.target_yaw = 2.0;
            });
        })))
        .then(Action::WaitFrames(2))
        .then(assertions::custom("force_update advances disabled camera", |world| {
            let orbit = camera_state(world);
            orbit.yaw.abs() > 0.01
        }))
        .then(assertions::custom("force_update auto-resets", |world| {
            let entity = camera_entity(world);
            let settings = world
                .get::<OrbitCameraSettings>(entity)
                .expect("settings should exist");
            !settings.force_update
        }))
        .then(assertions::log_summary("orbit_camera_force_update summary"))
        .then(Action::Screenshot("orbit_camera_force_update".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_auto_rotate() -> Scenario {
    Scenario::builder("orbit_camera_auto_rotate")
        .description(
            "Enable auto-rotate with a short idle wait, then verify the camera yaw advances \
             autonomously without any user input. Disable auto-rotate and confirm yaw stabilises.",
        )
        .then(Action::WaitFrames(30))
        // Enable auto-rotate: 0.5 s idle wait, 1.2 rad/s speed.
        .then(Action::Custom(Box::new(|world| {
            enable_auto_rotate(world, 0.5, 1.2);
        })))
        // Wait past the idle threshold (0.5 s = 30 frames) then an extra 60 frames for movement.
        .then(Action::WaitFrames(90))
        .then(assertions::custom("yaw changed due to auto-rotate", |world| {
            let orbit = camera_state(world);
            (orbit.yaw - orbit.home.yaw).abs() > 0.08
        }))
        .then(assertions::log_summary("orbit_camera_auto_rotate active summary"))
        .then(Action::Screenshot("orbit_camera_auto_rotate_active".into()))
        .then(Action::WaitFrames(1))
        // Disable auto-rotate and capture a stable yaw.
        .then(Action::Custom(Box::new(|world| {
            disable_auto_rotate(world);
        })))
        .then(Action::WaitFrames(30))
        .then(assertions::custom("yaw no longer advances after disable", |world| {
            let orbit = camera_state(world);
            // Yaw should be stable: target_yaw and yaw should be converging.
            (orbit.target_yaw - orbit.yaw).abs() < 0.3
        }))
        .then(assertions::log_summary("orbit_camera_auto_rotate summary"))
        .then(inspect::dump_component_json::<OrbitCamera>("orbit_camera_auto_rotate_state"))
        .then(Action::Screenshot("orbit_camera_auto_rotate_disabled".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_inertia() -> Scenario {
    Scenario::builder("orbit_camera_inertia")
        .description(
            "Enable inertia, apply a quick orbit drag via mouse motion, release, then verify \
             that the yaw target continues to coast past the drag endpoint (momentum carry-through) \
             before friction brings it to rest.",
        )
        .then(Action::WaitFrames(30))
        // Enable inertia with low friction so momentum is clearly visible.
        .then(Action::Custom(Box::new(|world| {
            enable_inertia(world, 2.0, 3.0, 4.0);
        })))
        .then(Action::Screenshot("orbit_camera_inertia_before".into()))
        .then(Action::WaitFrames(1))
        // Record yaw before the drag.
        .then(Action::Custom(Box::new(|world| {
            let orbit = camera_state(world);
            world.insert_resource(InertiaCheckpoint { yaw_after_drag: orbit.target_yaw });
        })))
        // Apply a quick, decisive drag then immediately release.
        .then(Action::PressMouseButton(MouseButton::Left))
        .then(Action::MouseMotion {
            delta: Vec2::new(150.0, 0.0),
        })
        .then(Action::WaitFrames(1))
        .then(Action::ReleaseMouseButton(MouseButton::Left))
        // Update the checkpoint to the yaw at the moment of release.
        .then(Action::Custom(Box::new(|world| {
            let orbit = camera_state(world);
            world.insert_resource(InertiaCheckpoint { yaw_after_drag: orbit.target_yaw });
        })))
        // Wait a few frames — with inertia the yaw target should still be moving.
        .then(Action::WaitFrames(10))
        .then(assertions::custom(
            "inertia carries yaw past the drag endpoint",
            |world| {
                let orbit = camera_state(world);
                let checkpoint = world.resource::<InertiaCheckpoint>();
                // The target_yaw should have advanced past what it was immediately at release.
                (orbit.target_yaw - checkpoint.yaw_after_drag).abs() > 0.02
            },
        ))
        .then(assertions::log_summary("orbit_camera_inertia summary"))
        .then(inspect::dump_component_json::<OrbitCamera>("orbit_camera_inertia_state"))
        .then(Action::Screenshot("orbit_camera_inertia_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

// ── checkpoint resources ──────────────────────────────────────────────────────

#[derive(Resource, Clone, Copy)]
struct InertiaCheckpoint {
    yaw_after_drag: f32,
}

fn common_focus() -> Vec3 {
    Vec3::new(0.0, 1.2, 0.0)
}
