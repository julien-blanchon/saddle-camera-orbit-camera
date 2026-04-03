mod components;
mod input;
mod math;
mod systems;

pub use components::{
    OrbitAngleLimit, OrbitAxisInversion, OrbitCamera, OrbitCameraAutoRotate, OrbitCameraFollow,
    OrbitCameraHome, OrbitCameraInputTarget, OrbitCameraMouseControls, OrbitCameraPresetView,
    OrbitCameraSettings, OrbitCameraSmoothing, OrbitCameraTouchControls, OrbitZoomLimits,
};
pub use math::{
    apply_exponential_zoom, fit_orthographic_scale_for_sphere, fit_perspective_distance_for_sphere,
    orbit_rotation, orbit_state_from_eye, orbit_translation, orthographic_pan_translation,
    perspective_pan_translation, shortest_angle_delta, smooth_angle, smooth_factor, smooth_scalar,
    wrap_angle,
};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    input::{
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
        touch::TouchInput,
    },
    prelude::*,
    transform::TransformSystems,
};

use crate::{components::OrbitCameraInternalState, input::TouchTracker};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum OrbitCameraSystems {
    ReadInput,
    ApplyIntent,
    SyncTransform,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

#[derive(Resource, Default)]
struct OrbitCameraRuntimeActive(bool);

pub struct OrbitCameraPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl OrbitCameraPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for OrbitCameraPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<OrbitCameraRuntimeActive>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<AccumulatedMouseScroll>()
            .init_resource::<TouchTracker>()
            .init_resource::<input::TouchGestureFrame>()
            .add_message::<TouchInput>()
            .register_type::<OrbitAngleLimit>()
            .register_type::<OrbitAxisInversion>()
            .register_type::<OrbitCamera>()
            .register_type::<OrbitCameraAutoRotate>()
            .register_type::<OrbitCameraFollow>()
            .register_type::<OrbitCameraHome>()
            .register_type::<OrbitCameraInputTarget>()
            .register_type::<OrbitCameraMouseControls>()
            .register_type::<OrbitCameraPresetView>()
            .register_type::<OrbitCameraSettings>()
            .register_type::<OrbitCameraSmoothing>()
            .register_type::<OrbitCameraTouchControls>()
            .register_type::<OrbitZoomLimits>()
            .add_systems(self.activate_schedule, activate_runtime)
            .add_systems(self.deactivate_schedule, deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    OrbitCameraSystems::ReadInput,
                    OrbitCameraSystems::ApplyIntent,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (input::capture_touch_gestures, input::apply_pointer_input)
                    .chain()
                    .in_set(OrbitCameraSystems::ReadInput)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                (
                    systems::tick_idle_timers,
                    systems::sync_follow_targets,
                    systems::apply_auto_rotate,
                    systems::advance_state,
                )
                    .chain()
                    .in_set(OrbitCameraSystems::ApplyIntent)
                    .run_if(runtime_is_active),
            )
            .configure_sets(PostUpdate, OrbitCameraSystems::SyncTransform)
            .add_systems(
                PostUpdate,
                systems::sync_transform
                    .in_set(OrbitCameraSystems::SyncTransform)
                    .before(TransformSystems::Propagate)
                    .run_if(runtime_is_active),
            );
    }
}

fn activate_runtime(mut runtime: ResMut<OrbitCameraRuntimeActive>) {
    runtime.0 = true;
}

fn deactivate_runtime(mut runtime: ResMut<OrbitCameraRuntimeActive>) {
    runtime.0 = false;
}

fn runtime_is_active(runtime: Res<OrbitCameraRuntimeActive>) -> bool {
    runtime.0
}

#[cfg(test)]
#[path = "plugin_tests.rs"]
mod plugin_tests;
