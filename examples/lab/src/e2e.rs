use bevy::{
    camera::{OrthographicProjection, ScalingMode},
    prelude::*,
};
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet,
    action::Action,
    actions::{assertions, inspect},
    init_scenario,
    scenario::Scenario,
};
use saddle_camera_orbit_camera::{OrbitCamera, OrbitCameraFollow, OrbitCameraInputTarget, OrbitCameraSystems};

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
        "snap_orbit_camera_ortho" => Some(build_orthographic_snapshot()),
        _ => None,
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec![
        "orbit_camera_smoke",
        "orbit_camera_input",
        "orbit_camera_follow_target",
        "snap_orbit_camera_ortho",
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

fn build_orthographic_snapshot() -> Scenario {
    Scenario::builder("snap_orbit_camera_ortho")
        .description(
            "Switch the lab camera into an orthographic overview, assert the shared component and projection stay aligned, then capture a snapshot.",
        )
        .then(Action::WaitFrames(30))
        .then(Action::Custom(Box::new(|world| {
            let entity = camera_entity(world);
            world.entity_mut(entity).insert(Projection::Orthographic(
                OrthographicProjection {
                    scale: 1.25,
                    scaling_mode: ScalingMode::FixedVertical {
                        viewport_height: 16.0,
                    },
                    ..OrthographicProjection::default_3d()
                },
            ));
            let mut camera = world
                .get_mut::<OrbitCamera>(entity)
                .expect("orbit camera should exist");
            camera.target_focus = Vec3::new(8.5, 0.8, -6.0);
            camera.target_yaw = 0.75;
            camera.target_pitch = -1.0;
            camera.target_orthographic_scale = 1.0;
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

fn common_focus() -> Vec3 {
    Vec3::new(0.0, 1.2, 0.0)
}
