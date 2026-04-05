use bevy::{prelude::*, transform::helper::TransformHelper};

use crate::{
    OrbitCamera, OrbitCameraCollision, OrbitCameraDollyZoom, OrbitCameraFollow,
    OrbitCameraInternalState, OrbitCameraSettings,
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

pub(crate) fn apply_inertia(
    time: Res<Time>,
    mut query: Query<(
        &OrbitCameraSettings,
        &mut OrbitCameraInternalState,
        &mut OrbitCamera,
        Option<&mut OrbitCameraFollow>,
    )>,
) {
    let dt = time.delta_secs();
    for (settings, mut internal, mut orbit, follow) in &mut query {
        if !settings.enabled || !settings.inertia.enabled || dt <= 0.0 {
            continue;
        }

        if internal.manual_interaction_this_frame {
            continue;
        }

        let orbit_vel = internal.orbit_velocity;
        if orbit_vel.length_squared() > 1e-6 {
            orbit.target_yaw = wrap_angle(orbit.target_yaw - orbit_vel.x);
            orbit.target_pitch += orbit_vel.y;
        }

        let pan_vel = internal.pan_velocity;
        if pan_vel.length_squared() > 1e-6 {
            if let Some(follow) = follow
                && follow.enabled
            {
                follow.into_inner().offset += pan_vel * dt;
            } else {
                orbit.target_focus += pan_vel * dt;
            }
        }

        let zoom_vel = internal.zoom_velocity;
        if zoom_vel.abs() > 1e-6 {
            orbit.target_distance = (orbit.target_distance * (-zoom_vel * dt).exp()).max(0.001);
        }

        let orbit_decay = (-settings.inertia.orbit_friction * dt).exp();
        internal.orbit_velocity *= orbit_decay;

        let pan_decay = (-settings.inertia.pan_friction * dt).exp();
        internal.pan_velocity *= pan_decay;

        let zoom_decay = (-settings.inertia.zoom_friction * dt).exp();
        internal.zoom_velocity *= zoom_decay;

        if internal.orbit_velocity.length_squared() < 1e-8 {
            internal.orbit_velocity = Vec2::ZERO;
        }
        if internal.pan_velocity.length_squared() < 1e-8 {
            internal.pan_velocity = Vec3::ZERO;
        }
        if internal.zoom_velocity.abs() < 1e-8 {
            internal.zoom_velocity = 0.0;
        }
    }
}

pub(crate) fn advance_state(
    time: Res<Time>,
    mut query: Query<(&mut OrbitCameraSettings, &mut OrbitCamera)>,
) {
    let dt = time.delta_secs();
    for (mut settings, mut orbit) in &mut query {
        if settings.allow_upside_down {
            // No pitch clamping when upside-down is allowed
        } else {
            orbit.target_pitch = settings.pitch_limits.clamp(orbit.target_pitch);
        }
        if let Some(yaw_limits) = settings.yaw_limits {
            orbit.target_yaw = yaw_limits.clamp(orbit.target_yaw);
        }
        orbit.target_distance = settings.zoom_limits.clamp_distance(orbit.target_distance);
        orbit.target_orthographic_scale = settings
            .zoom_limits
            .clamp_orthographic_scale(orbit.target_orthographic_scale);

        if let Some(focus_bounds) = settings.focus_bounds {
            orbit.target_focus = focus_bounds.clamp_focus(orbit.target_focus);
        }

        if !settings.enabled && !settings.force_update {
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

        if settings.force_update {
            settings.force_update = false;
        }
    }
}

pub(crate) fn apply_dolly_zoom(
    mut query: Query<(&OrbitCamera, &OrbitCameraDollyZoom, &mut Projection)>,
) {
    for (orbit, dolly, mut projection) in &mut query {
        if !dolly.enabled {
            continue;
        }

        if let Projection::Perspective(perspective) = projection.as_mut() {
            let half_width = dolly.reference_width * 0.5;
            let distance = orbit.distance.max(0.01);
            let fov = 2.0 * (half_width / distance).atan();
            perspective.fov = fov.clamp(0.01, std::f32::consts::PI - 0.01);
        }
    }
}

pub(crate) fn sync_transform(
    mut query: Query<(
        &OrbitCamera,
        &OrbitCameraInternalState,
        &OrbitCameraSettings,
        &mut Transform,
        &mut Projection,
        Option<&OrbitCameraDollyZoom>,
    )>,
) {
    for (orbit, internal, settings, mut transform, mut projection, dolly) in &mut query {
        let effective_distance = if let Some(collision_dist) = internal.collision_distance
            && settings.enabled
        {
            orbit.distance.min(collision_dist)
        } else {
            orbit.distance
        };

        transform.rotation = orbit_rotation(orbit.yaw, orbit.pitch);
        transform.translation =
            orbit_translation(orbit.focus, orbit.yaw, orbit.pitch, effective_distance);

        if let Projection::Orthographic(orthographic) = projection.as_mut() {
            orthographic.scale = orbit.orthographic_scale;
        }

        if dolly.is_none() || dolly.is_some_and(|d| !d.enabled) {
            // dolly_zoom system handles FOV when active
        }
    }
}

pub(crate) fn update_collision(
    time: Res<Time>,
    mut query: Query<(
        &OrbitCamera,
        &OrbitCameraCollision,
        &OrbitCameraSettings,
        &mut OrbitCameraInternalState,
    )>,
) {
    let dt = time.delta_secs();
    for (orbit, collision, settings, mut internal) in &mut query {
        if !collision.enabled || !settings.enabled {
            internal.collision_distance = None;
            continue;
        }

        // Without a physics world, we provide the infrastructure for consumers
        // to set collision_distance externally via the internal state.
        // The collision_distance field smoothly returns to None when not actively set.
        if let Some(current) = internal.collision_distance {
            let target = orbit.distance;
            let smoothed = smooth_scalar(current, target, collision.smooth_speed, dt);
            if (smoothed - target).abs() < 0.01 {
                internal.collision_distance = None;
            } else {
                internal.collision_distance = Some(smoothed);
            }
        }
    }
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
