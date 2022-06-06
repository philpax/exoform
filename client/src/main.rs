use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use shared::{Node, NodeData};

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
#[derive(Debug)]
pub struct Graph(Node);
pub struct CurrentEntity(Option<Entity>);
pub struct RebuildTimer(Timer);

fn build_sample_graph() -> Node {
    Node::default_with_data(NodeData::Union(0.0, vec![]))
}

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .insert_resource(Graph(build_sample_graph()))
        .insert_resource(CurrentEntity(None))
        .insert_resource(RebuildTimer(Timer::new(
            std::time::Duration::from_secs_f32(0.2),
            true,
        )))
        .insert_resource(WindowDescriptor {
            width: 1600.,
            height: 900.,
            ..Default::default()
        })
        .init_resource::<OccupiedScreenSpace>()
        .add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_web_fullscreen::FullViewportPlugin);

    app.add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_startup_system(mesh_generation::rebuild_mesh)
        .add_system(ui::sdf_code_editor)
        .add_system(mesh_generation::keep_rebuilding_mesh)
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
        .insert(camera::PanOrbitCamera {
            radius: eye.distance(target),
            ..Default::default()
        });
}
