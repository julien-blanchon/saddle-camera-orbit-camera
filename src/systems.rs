use bevy::{prelude::*, transform::helper::TransformHelper};

use crate::{
    OrbitCamera, OrbitCameraFollow, OrbitCameraInternalState, OrbitCameraSettings,
    math::{
        orbit_rotation, orbit_translation, smooth_angle, smooth_scalar, smooth_vec3, wrap_angle,
    },
};

pub(crate) fn tick_idle_timers(
    time: Res<Time>,
    mut query: Query<(&OrbitCameraSettings, &mut OrbitCameraInternalState), With<OrbitCamera>>,
) {
    let dt = time.delta_secs();
    for (settings, mut internal) in &mut query {
        if !settings.enabled {
            internal.manual_interaction_this_frame = false;
            continue;
        }

        if internal.manual_interaction_this_frame {
            internal.idle_seconds = 0.0;
            internal.manual_interaction_this_frame = false;
        } else {
            internal.idle_seconds += dt;
        }
    }
}

pub(crate) fn sync_follow_targets(
    helper: TransformHelper,
    mut cameras: Query<(&mut OrbitCamera, &OrbitCameraFollow, &OrbitCameraSettings)>,
) {
    for (mut orbit, follow, settings) in &mut cameras {
        if !settings.enabled || !follow.enabled {
            continue;
        }

        let Ok(global) = helper.compute_global_transform(follow.target) else {
            continue;
        };

        orbit.target_focus = global.translation() + follow.offset;
    }
}

pub(crate) fn apply_auto_rotate(
    time: Res<Time>,
    mut query: Query<(
        &OrbitCameraSettings,
        &OrbitCameraInternalState,
        &mut OrbitCamera,
    )>,
) {
    let dt = time.delta_secs();
    for (settings, internal, mut orbit) in &mut query {
        if !settings.enabled || !settings.auto_rotate.enabled {
            continue;
        }

        if internal.idle_seconds < settings.auto_rotate.wait_seconds {
            continue;
        }

        orbit.target_yaw = wrap_angle(orbit.target_yaw + settings.auto_rotate.speed * dt);
    }
}

pub(crate) fn advance_state(
    time: Res<Time>,
    mut query: Query<(&OrbitCameraSettings, &mut OrbitCamera)>,
) {
    let dt = time.delta_secs();
    for (settings, mut orbit) in &mut query {
        orbit.target_pitch = settings.pitch_limits.clamp(orbit.target_pitch);
        if let Some(yaw_limits) = settings.yaw_limits {
            orbit.target_yaw = yaw_limits.clamp(orbit.target_yaw);
        }
        orbit.target_distance = settings.zoom_limits.clamp_distance(orbit.target_distance);
        orbit.target_orthographic_scale = settings
            .zoom_limits
            .clamp_orthographic_scale(orbit.target_orthographic_scale);

        if !settings.enabled {
            continue;
        }

        orbit.yaw = smooth_angle(
            orbit.yaw,
            orbit.target_yaw,
            settings.smoothing.rotation_decay,
            dt,
        );
        orbit.pitch = smooth_angle(
            orbit.pitch,
            orbit.target_pitch,
            settings.smoothing.rotation_decay,
            dt,
        );
        orbit.distance = smooth_scalar(
            orbit.distance,
            orbit.target_distance,
            settings.smoothing.zoom_decay,
            dt,
        );
        orbit.orthographic_scale = smooth_scalar(
            orbit.orthographic_scale,
            orbit.target_orthographic_scale,
            settings.smoothing.zoom_decay,
            dt,
        );
        orbit.focus = smooth_vec3(
            orbit.focus,
            orbit.target_focus,
            settings.smoothing.focus_decay,
            dt,
        );
    }
}

pub(crate) fn sync_transform(mut query: Query<(&OrbitCamera, &mut Transform, &mut Projection)>) {
    for (orbit, mut transform, mut projection) in &mut query {
        transform.rotation = orbit_rotation(orbit.yaw, orbit.pitch);
        transform.translation =
            orbit_translation(orbit.focus, orbit.yaw, orbit.pitch, orbit.distance);

        if let Projection::Orthographic(orthographic) = projection.as_mut() {
            orthographic.scale = orbit.orthographic_scale;
        }
    }
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
