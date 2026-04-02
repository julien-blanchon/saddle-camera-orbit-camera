use bevy::{app::PostStartup, ecs::schedule::ScheduleLabel, prelude::*};

use crate::{OrbitCameraPlugin, OrbitCameraSystems};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct ActivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct DeactivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct RuntimeSchedule;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AfterApplyIntent;

#[derive(Resource, Default, Debug, PartialEq, Eq)]
struct OrderLog(Vec<&'static str>);

fn record_camera(mut log: ResMut<OrderLog>) {
    log.0.push("camera");
}

fn record_after(mut log: ResMut<OrderLog>) {
    log.0.push("after");
}

#[test]
fn plugin_builds_with_custom_schedule_labels_and_public_sets() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_schedule(ActivateSchedule)
        .init_schedule(DeactivateSchedule)
        .init_schedule(RuntimeSchedule)
        .init_resource::<OrderLog>()
        .add_plugins(OrbitCameraPlugin::new(
            ActivateSchedule,
            DeactivateSchedule,
            RuntimeSchedule,
        ))
        .configure_sets(
            RuntimeSchedule,
            OrbitCameraSystems::ApplyIntent.before(AfterApplyIntent),
        )
        .add_systems(
            RuntimeSchedule,
            (
                record_camera.in_set(OrbitCameraSystems::ApplyIntent),
                record_after.in_set(AfterApplyIntent),
            ),
        );

    app.finish();
    app.world_mut().run_schedule(ActivateSchedule);
    app.world_mut().run_schedule(RuntimeSchedule);

    assert_eq!(
        app.world().resource::<OrderLog>().0,
        vec!["camera", "after"]
    );
}

#[test]
fn always_on_constructor_activates_runtime_after_post_startup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(OrbitCameraPlugin::always_on(Update));

    app.finish();
    app.world_mut().run_schedule(PostStartup);
    app.update();
}
