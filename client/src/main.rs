use std::path::PathBuf;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use clap::Parser;

mod camera;
mod mesh_generation;
mod resources;
mod ui;

pub fn main() -> anyhow::Result<()> {
    #[derive(Parser)]
    #[clap(author, version, about, long_about = None)]
    struct Args {
        #[clap()]
        path: PathBuf,
    }

    let args = Args::parse();

    let mut app = App::new();
    #[cfg(target_arch = "wasm32")]
    let winit_settings = bevy::winit::WinitSettings::desktop_app();
    #[cfg(not(target_arch = "wasm32"))]
    let winit_settings = {
        // Temporary workarond:
        // https://github.com/bevyengine/bevy/issues/5384
        use bevy::winit::{UpdateMode, WinitSettings};
        use std::time::Duration;
        WinitSettings {
            focused_mode: UpdateMode::Reactive {
                max_wait: Duration::from_secs(5),
            },
            unfocused_mode: UpdateMode::Reactive {
                max_wait: Duration::from_secs(5),
            },
            ..Default::default()
        }
    };

    let graph: shared::Graph = serde_json::from_str(&std::fs::read_to_string(args.path)?)?;

    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(winit_settings)
        .insert_resource(graph)
        .insert_resource(WindowDescriptor {
            width: 1600.,
            height: 900.,
            title: format!("Exoform {}", env!("CARGO_PKG_VERSION")),
            ..Default::default()
        })
        .insert_resource(resources::RenderParameters {
            wireframe: false,
            colours: true,
        })
        .insert_resource(resources::MeshGenerationResult::Unbuilt)
        .insert_resource(resources::OccupiedScreenSpace::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy::pbr::wireframe::WireframePlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_web_fullscreen::FullViewportPlugin);

    app.add_plugin(EguiPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(mesh_generation::MeshGenerationPlugin)
        .add_startup_system(setup)
        .add_system(camera::pan_orbit_camera)
        .run();

    Ok(())
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
