use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use shared::NodeData;

mod camera;
mod mesh_generation;
mod ui;

#[derive(Default)]
pub struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    _bottom: f32,
}

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .insert_resource(shared::Graph::new(NodeData::Union(shared::Union::new())))
        .insert_resource(WindowDescriptor {
            width: 1600.,
            height: 900.,
            title: format!("Exoform {}", env!("CARGO_PKG_VERSION")),
            ..Default::default()
        })
        .init_resource::<OccupiedScreenSpace>()
        .add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_web_fullscreen::FullViewportPlugin);

    app.add_plugin(EguiPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(mesh_generation::MeshGenerationPlugin)
        .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
        .add_plugin(bevy_transform_gizmo::TransformGizmoPlugin::default())
        .add_startup_system(setup)
        .add_system(camera::pan_orbit_camera)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    let eye = Vec3::new(-2.0, 5.0, 5.0);
    let target = Vec3::new(0., 0., 0.);
    let transform = Transform::from_translation(eye).looking_at(target, Vec3::Y);
    commands
        .spawn_bundle(Camera3dBundle {
            transform,
            ..Default::default()
        })
        // .insert(camera::PanOrbitCamera {
        //     radius: eye.distance(target),
        //     ..Default::default()
        // })
        .insert_bundle(bevy_mod_picking::PickingCameraBundle::default())
        .insert(bevy_transform_gizmo::GizmoPickSource::default());
}
