use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_egui::{EguiContext, EguiPlugin};

use smooth_bevy_cameras::{
    controllers::orbit::{
        ControlEvent, OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
    },
    LookTransformPlugin,
};

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::new(true))
        .add_plugin(bevy_web_fullscreen::FullViewportPlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_system(input_map)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn_bundle(OrbitCameraBundle::new(
        OrbitCameraController::default(),
        PerspectiveCameraBundle::default(),
        Vec3::new(-2.0, 5.0, 5.0),
        Vec3::new(0., 0., 0.),
    ));
}

pub fn input_map(
    mut events: EventWriter<ControlEvent>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut egui_context: ResMut<EguiContext>,
    mouse_buttons: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    controllers: Query<&OrbitCameraController>,
) {
    let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    let OrbitCameraController {
        mouse_rotate_sensitivity,
        mouse_translate_sensitivity,
        ..
    } = *controller;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    let egui_wants_input = {
        let ctx = egui_context.ctx_mut();
        ctx.wants_keyboard_input() || ctx.wants_pointer_input()
    };

    if !egui_wants_input && mouse_buttons.pressed(MouseButton::Left) {
        if keyboard.pressed(KeyCode::LControl) {
            events.send(ControlEvent::TranslateTarget(
                mouse_translate_sensitivity * cursor_delta,
            ));
        } else {
            events.send(ControlEvent::Orbit(mouse_rotate_sensitivity * cursor_delta));
        }
    }
}
