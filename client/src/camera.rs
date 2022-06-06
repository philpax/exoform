use bevy::{input::mouse::MouseMotion, prelude::*, render::camera::Projection};
use bevy_egui::EguiContext;

use super::OccupiedScreenSpace;

/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
pub struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

pub(crate) fn pan_orbit_camera(
    occupied_screen_space: Res<OccupiedScreenSpace>,
    windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    input_mouse: Res<Input<MouseButton>>,
    mut egui_context: ResMut<EguiContext>,
    mut query: Query<(
        &mut PanOrbitCamera,
        &mut Transform,
        &Projection,
        With<Camera3d>,
    )>,
) {
    let orbit_button = MouseButton::Left;
    let pan_button = MouseButton::Middle;
    let zoom_button = MouseButton::Right;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut zoom = 0.0;
    let mut orbit_button_changed = false;

    let egui_wants_input = {
        let ctx = egui_context.ctx_mut();
        ctx.wants_keyboard_input() || ctx.wants_pointer_input()
    };
    if !egui_wants_input {
        if input_mouse.pressed(orbit_button) {
            for ev in ev_motion.iter() {
                rotation_move += ev.delta;
            }
        } else if input_mouse.pressed(pan_button) {
            // Pan only if we're not rotating at the moment
            for ev in ev_motion.iter() {
                pan += ev.delta;
            }
        } else if input_mouse.pressed(zoom_button) {
            for ev in ev_motion.iter() {
                zoom += ev.delta.x;
            }
        }
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    let window = get_primary_window_size(&windows);
    for (mut pan_orbit, mut transform, projection, _) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let projection = match projection {
            Projection::Perspective(projection) => projection,
            Projection::Orthographic(_) => continue,
        };

        if rotation_move.length_squared() > 0.0 {
            let (yaw, pitch) = {
                let delta_x = {
                    let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                    if pan_orbit.upside_down {
                        -delta
                    } else {
                        delta
                    }
                };
                let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
                (
                    Quat::from_rotation_y(-delta_x),
                    Quat::from_rotation_x(-delta_y),
                )
            };
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation *= pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            // make panning distance independent of resolution and FOV,
            let pan =
                pan * Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if zoom.abs() > 0.0 {
            let zoom = zoom * projection.fov * projection.aspect_ratio / window.x;
            pan_orbit.radius -= zoom * pan_orbit.radius;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
        // parent = x and y rotation
        // child = z-offset
        let rot_matrix = Mat3::from_quat(transform.rotation);
        let uncorrected_translation =
            pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));

        // Once the initial translation has been calculated, add in an offset to handle the
        // complications from having a side panel.
        let frustum_height = 2.0 * pan_orbit.radius * (projection.fov * 0.5).tan();
        let frustum_width = frustum_height * projection.aspect_ratio;

        let window = windows.get_primary().unwrap();

        let left_taken = occupied_screen_space.left / window.width();
        let right_taken = occupied_screen_space._right / window.width();
        let top_taken = occupied_screen_space._top / window.height();
        let bottom_taken = occupied_screen_space._bottom / window.height();
        let offset = transform.rotation.mul_vec3(Vec3::new(
            (right_taken - left_taken) * frustum_width * 0.5,
            (top_taken - bottom_taken) * frustum_height * 0.5,
            0.0,
        ));
        transform.translation = uncorrected_translation + offset;
    }
}

pub(crate) fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    Vec2::new(window.width() as f32, window.height() as f32)
}
