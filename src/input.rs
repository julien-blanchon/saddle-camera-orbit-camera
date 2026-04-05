use std::collections::HashMap;

use bevy::{
    input::{
        gamepad::GamepadAxis,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
        touch::{TouchInput, TouchPhase},
    },
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    OrbitCamera, OrbitCameraFollow, OrbitCameraInputTarget, OrbitCameraInternalState,
    OrbitCameraSettings,
    math::{
        apply_exponential_zoom, orthographic_pan_translation, perspective_pan_translation,
        wrap_angle,
    },
};

#[derive(Debug, Clone, Copy, Default)]
struct TouchSample {
    current: Vec2,
    previous: Vec2,
}

#[derive(Resource, Default)]
pub(crate) struct TouchTracker {
    active: HashMap<u64, TouchSample>,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub(crate) struct TouchGestureFrame {
    pub active_touch_count: u8,
    pub orbit_delta: Vec2,
    pub pan_delta: Vec2,
    pub pinch_delta: f32,
}

impl TouchTracker {
    fn begin_frame(&mut self) {
        for sample in self.active.values_mut() {
            sample.previous = sample.current;
        }
    }

    fn apply(&mut self, event: &TouchInput) {
        match event.phase {
            TouchPhase::Started => {
                self.active.insert(
                    event.id,
                    TouchSample {
                        current: event.position,
                        previous: event.position,
                    },
                );
            }
            TouchPhase::Moved => {
                let entry = self.active.entry(event.id).or_default();
                entry.current = event.position;
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                self.active.remove(&event.id);
            }
        }
    }

    fn first_two(&self) -> [Option<TouchSample>; 2] {
        let mut touches: Vec<(u64, TouchSample)> = self
            .active
            .iter()
            .map(|(id, sample)| (*id, *sample))
            .collect();
        touches.sort_by_key(|(id, _)| *id);
        [
            touches.first().map(|(_, sample)| *sample),
            touches.get(1).map(|(_, sample)| *sample),
        ]
    }
}

pub(crate) fn capture_touch_gestures(
    mut touch_events: MessageReader<TouchInput>,
    mut tracker: ResMut<TouchTracker>,
    mut frame: ResMut<TouchGestureFrame>,
) {
    tracker.begin_frame();
    *frame = TouchGestureFrame::default();

    for event in touch_events.read() {
        tracker.apply(event);
    }

    let [first, second] = tracker.first_two();
    frame.active_touch_count = tracker.active.len().min(255) as u8;

    match (first, second) {
        (Some(one), None) => {
            frame.orbit_delta = one.current - one.previous;
        }
        (Some(a), Some(b)) => {
            let current_center = (a.current + b.current) * 0.5;
            let previous_center = (a.previous + b.previous) * 0.5;
            frame.pan_delta = current_center - previous_center;
            frame.pinch_delta = a.current.distance(b.current) - a.previous.distance(b.previous);
        }
        _ => {}
    }
}

pub(crate) fn apply_pointer_input(
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    mouse_motion: Option<Res<AccumulatedMouseMotion>>,
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    touch_frame: Res<TouchGestureFrame>,
    mut cameras: ParamSet<(
        Query<
            (Entity, &Camera, &OrbitCameraSettings),
            (With<OrbitCamera>, With<OrbitCameraInputTarget>),
        >,
        Query<
            (
                &Camera,
                &GlobalTransform,
                &Projection,
                &mut OrbitCamera,
                &OrbitCameraSettings,
                Option<&mut OrbitCameraFollow>,
                &mut OrbitCameraInternalState,
            ),
            (With<OrbitCamera>, With<OrbitCameraInputTarget>),
        >,
    )>,
) {
    let Some(entity) = ({
        let selection = cameras.p0();
        select_input_target(&selection)
    }) else {
        return;
    };

    let mut controlled = cameras.p1();
    let Ok((camera, camera_transform, projection, mut orbit, settings, follow, mut internal)) =
        controlled.get_mut(entity)
    else {
        return;
    };

    if !settings.enabled {
        return;
    }

    let mut follow = follow;

    let motion = mouse_motion.map_or(Vec2::ZERO, |delta| delta.delta);
    let scroll = mouse_scroll.map_or(Vec2::ZERO, |delta| delta.delta);
    let buttons = mouse_buttons.as_deref();

    let mut mouse_orbit = Vec2::ZERO;
    let mut mouse_pan = Vec2::ZERO;
    if buttons.is_some_and(|pressed| pressed.pressed(settings.mouse.orbit_button)) {
        mouse_orbit = motion;
    } else if buttons.is_some_and(|pressed| pressed.pressed(settings.mouse.pan_button)) {
        mouse_pan = motion;
    }

    let touch_orbit = if settings.touch.enabled {
        touch_frame.orbit_delta
    } else {
        Vec2::ZERO
    };
    let touch_pan = if settings.touch.enabled {
        touch_frame.pan_delta
    } else {
        Vec2::ZERO
    };
    let touch_pinch = if settings.touch.enabled {
        touch_frame.pinch_delta
    } else {
        0.0
    };

    let orbit_delta = Vec2::new(
        apply_inversion(mouse_orbit.x, settings.inversion.orbit_x)
            * settings.mouse.orbit_sensitivity.x
            + apply_inversion(touch_orbit.x, settings.inversion.orbit_x)
                * settings.touch.orbit_sensitivity.x,
        apply_inversion(mouse_orbit.y, settings.inversion.orbit_y)
            * settings.mouse.orbit_sensitivity.y
            + apply_inversion(touch_orbit.y, settings.inversion.orbit_y)
                * settings.touch.orbit_sensitivity.y,
    );

    let pan_pixels = Vec2::new(
        apply_inversion(mouse_pan.x, settings.inversion.pan_x) * settings.mouse.pan_sensitivity
            + apply_inversion(touch_pan.x, settings.inversion.pan_x)
                * settings.touch.pan_sensitivity,
        apply_inversion(mouse_pan.y, settings.inversion.pan_y) * settings.mouse.pan_sensitivity
            + apply_inversion(touch_pan.y, settings.inversion.pan_y)
                * settings.touch.pan_sensitivity,
    );

    let zoom_sign = if settings.reversed_zoom { -1.0 } else { 1.0 };
    let zoom_delta = zoom_sign
        * (apply_inversion(
            scroll.y * settings.mouse.wheel_zoom_sensitivity,
            settings.inversion.zoom,
        ) + apply_inversion(
            touch_pinch * settings.touch.pinch_zoom_sensitivity,
            settings.inversion.zoom,
        ));

    if orbit_delta.length_squared() > 0.0 {
        let effective_orbit = if settings.allow_upside_down {
            let is_upside_down =
                orbit.pitch.rem_euclid(std::f32::consts::TAU) > std::f32::consts::PI;
            let x = if is_upside_down {
                -orbit_delta.x
            } else {
                orbit_delta.x
            };
            Vec2::new(x, orbit_delta.y)
        } else {
            orbit_delta
        };
        orbit.target_yaw = wrap_angle(orbit.target_yaw - effective_orbit.x);
        orbit.target_pitch += effective_orbit.y;

        if settings.inertia.enabled {
            internal.orbit_velocity = effective_orbit;
        }
    } else if settings.inertia.enabled
        && buttons.is_some_and(|pressed| !pressed.pressed(settings.mouse.orbit_button))
        && touch_frame.active_touch_count == 0
    {
        // Pointer released — inertia will take over in the inertia system
    }

    if pan_pixels.length_squared() > 0.0 {
        let viewport_size = camera
            .logical_viewport_size()
            .or_else(|| primary_window.single().ok().map(Window::size))
            .unwrap_or(Vec2::new(1280.0, 720.0));
        let translation = match projection {
            Projection::Perspective(perspective) => perspective_pan_translation(
                viewport_size,
                perspective,
                camera_transform.compute_transform().rotation,
                orbit.distance,
                pan_pixels,
            ),
            Projection::Orthographic(orthographic) => orthographic_pan_translation(
                viewport_size,
                orthographic,
                camera_transform.compute_transform().rotation,
                pan_pixels,
            ),
            _ => Vec3::ZERO,
        };

        if let Some(follow) = follow.as_deref_mut() {
            if follow.enabled {
                follow.offset += translation;
            } else {
                orbit.target_focus += translation;
            }
        } else {
            orbit.target_focus += translation;
        }

        if settings.inertia.enabled {
            internal.pan_velocity = translation / time_delta_or_default(0.016);
        }
    }

    if zoom_delta.abs() > f32::EPSILON {
        let cursor_anchor_before = (settings.mouse.zoom_to_cursor
            && matches!(projection, Projection::Perspective(_)))
        .then(|| current_cursor_position(&primary_window))
        .flatten()
        .and_then(|cursor| {
            cursor_focus_anchor(camera, camera_transform, cursor, orbit.target_focus)
        });
        match projection {
            Projection::Perspective(_) => {
                orbit.target_distance =
                    apply_exponential_zoom(orbit.target_distance, zoom_delta, 1.0);
            }
            Projection::Orthographic(_) => {
                orbit.target_orthographic_scale =
                    apply_exponential_zoom(orbit.target_orthographic_scale, zoom_delta, 1.0);
            }
            _ => {}
        }

        if let (Some(cursor), Some(anchor_before)) = (
            current_cursor_position(&primary_window),
            cursor_anchor_before,
        ) && let Some(anchor_after) = predicted_cursor_focus_anchor(camera, &orbit, cursor)
        {
            apply_focus_translation(
                &mut orbit,
                follow.as_deref_mut(),
                anchor_before - anchor_after,
            );
        }

        if settings.inertia.enabled {
            internal.zoom_velocity = zoom_delta * 4.0;
        }
    }

    if orbit_delta.length_squared() > 0.0
        || pan_pixels.length_squared() > 0.0
        || zoom_delta.abs() > f32::EPSILON
    {
        internal.manual_interaction_this_frame = true;
        // Clear inertia velocities that conflict with current active input
        if orbit_delta.length_squared() > 0.0 {
            // Orbit velocity is set above, don't clear
        }
        if pan_pixels.length_squared() > 0.0 {
            // Pan velocity is set above, don't clear
        }
    }
}

pub(crate) fn apply_gamepad_input(
    time: Res<Time>,
    gamepads: Query<&Gamepad>,
    mut cameras: Query<
        (
            &mut OrbitCamera,
            &OrbitCameraSettings,
            &mut OrbitCameraInternalState,
        ),
        With<OrbitCameraInputTarget>,
    >,
) {
    let dt = time.delta_secs();
    let Some(gamepad) = gamepads.iter().next() else {
        return;
    };

    for (mut orbit, settings, mut internal) in &mut cameras {
        if !settings.enabled || !settings.gamepad.enabled {
            continue;
        }

        let deadzone = settings.gamepad.deadzone;

        let right_x = apply_deadzone(
            gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0),
            deadzone,
        );
        let right_y = apply_deadzone(
            gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0),
            deadzone,
        );

        let left_x = apply_deadzone(
            gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0),
            deadzone,
        );
        let left_y = apply_deadzone(
            gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0),
            deadzone,
        );

        let orbit_input = Vec2::new(right_x, -right_y);
        if orbit_input.length_squared() > 0.0 {
            let scaled = orbit_input * settings.gamepad.orbit_sensitivity * dt;
            let effective_x = apply_inversion(scaled.x, settings.inversion.orbit_x);
            let effective_y = apply_inversion(scaled.y, settings.inversion.orbit_y);
            orbit.target_yaw = wrap_angle(orbit.target_yaw - effective_x);
            orbit.target_pitch += effective_y;
            internal.manual_interaction_this_frame = true;
        }

        let pan_input = Vec2::new(left_x, -left_y);
        if pan_input.length_squared() > 0.0 {
            let rotation = crate::orbit_rotation(orbit.yaw, orbit.pitch);
            let scaled = pan_input * settings.gamepad.pan_sensitivity * dt;
            let translation = rotation * Vec3::new(-scaled.x, scaled.y, 0.0);
            orbit.target_focus += translation;
            internal.manual_interaction_this_frame = true;
        }

        let left_trigger = gamepad.get(GamepadAxis::LeftZ).unwrap_or(0.0).max(0.0);
        let right_trigger = gamepad.get(GamepadAxis::RightZ).unwrap_or(0.0).max(0.0);
        let zoom_input = right_trigger - left_trigger;
        if zoom_input.abs() > 0.01 {
            let zoom_sign = if settings.reversed_zoom { -1.0 } else { 1.0 };
            let zoom_delta = apply_inversion(
                zoom_input * settings.gamepad.zoom_sensitivity * dt * zoom_sign,
                settings.inversion.zoom,
            );
            orbit.target_distance = apply_exponential_zoom(orbit.target_distance, zoom_delta, 1.0);
            internal.manual_interaction_this_frame = true;
        }
    }
}

fn select_input_target(
    query: &Query<
        (Entity, &Camera, &OrbitCameraSettings),
        (With<OrbitCamera>, With<OrbitCameraInputTarget>),
    >,
) -> Option<Entity> {
    query
        .iter()
        .filter(|(_, camera, settings)| settings.enabled && camera.is_active)
        .max_by_key(|(entity, camera, _)| (camera.order, entity.to_bits()))
        .map(|(entity, _, _)| entity)
}

fn apply_inversion(value: f32, inverted: bool) -> f32 {
    if inverted { -value } else { value }
}

fn apply_deadzone(value: f32, deadzone: f32) -> f32 {
    if value.abs() < deadzone {
        0.0
    } else {
        (value - value.signum() * deadzone) / (1.0 - deadzone)
    }
}

fn current_cursor_position(windows: &Query<&Window, With<PrimaryWindow>>) -> Option<Vec2> {
    windows.single().ok()?.cursor_position()
}

fn cursor_focus_anchor(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor_position: Vec2,
    focus: Vec3,
) -> Option<Vec3> {
    let ray = camera
        .viewport_to_world(camera_transform, cursor_position)
        .ok()?;
    let normal = camera_transform.forward().as_vec3();
    ray.plane_intersection_point(focus, InfinitePlane3d::new(normal))
}

fn predicted_cursor_focus_anchor(
    camera: &Camera,
    orbit: &OrbitCamera,
    cursor_position: Vec2,
) -> Option<Vec3> {
    let predicted_transform = Transform {
        rotation: crate::orbit_rotation(orbit.target_yaw, orbit.target_pitch),
        translation: crate::orbit_translation(
            orbit.target_focus,
            orbit.target_yaw,
            orbit.target_pitch,
            orbit.target_distance,
        ),
        ..default()
    };

    cursor_focus_anchor(
        camera,
        &GlobalTransform::from(predicted_transform),
        cursor_position,
        orbit.target_focus,
    )
}

fn apply_focus_translation(
    orbit: &mut OrbitCamera,
    follow: Option<&mut OrbitCameraFollow>,
    translation: Vec3,
) {
    if translation.length_squared() <= f32::EPSILON {
        return;
    }

    if let Some(follow) = follow
        && follow.enabled
    {
        follow.offset += translation;
        return;
    }

    orbit.target_focus += translation;
}

fn time_delta_or_default(default: f32) -> f32 {
    default
}
