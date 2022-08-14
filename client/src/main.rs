use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use clap::Parser;

use shared::Graph;
use tokio::net::TcpStream;

mod camera;
mod mesh_generation;
mod resources;
mod ui;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        compile_error!("Argument parsing doesn't work on WASM. This is intentional as we're using a TCP server. Once the network protocol actually works, we can investigate using WebSocket or WebTransport.");
    }

    #[derive(Parser)]
    #[clap(author, version, about, long_about = None)]
    struct Args {
        #[clap(short, long)]
        host: String,
        #[clap(short, long)]
        port: Option<u16>,
        #[clap(short, long)]
        room: String,
    }

    let args = Args::parse();
    let port = args.port.unwrap_or(shared::DEFAULT_PORT);

    let shutdown = Arc::new(AtomicBool::new(false));

    let (rx, tx) = (Arc::new(Mutex::new(vec![])), Arc::new(Mutex::new(vec![])));
    let _network_tasks =
        create_network_tasks(&args.host, port, rx.clone(), tx.clone(), shutdown.clone()).await?;

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

    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(winit_settings)
        .insert_resource(shared::Graph::new_client())
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
        .insert_resource(resources::NetworkState::new(shutdown.clone(), tx, rx))
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy::pbr::wireframe::WireframePlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_system(synchronise_network_to_local);

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

async fn create_network_tasks(
    host: &str,
    port: u16,
    rx: Arc<Mutex<Vec<shared::GraphChange>>>,
    tx: Arc<Mutex<Vec<shared::protocol::Message>>>,
    shutdown: Arc<AtomicBool>,
) -> anyhow::Result<(
    tokio::task::JoinHandle<anyhow::Result<()>>,
    tokio::task::JoinHandle<anyhow::Result<()>>,
)> {
    let (socket_rx, socket_tx) = TcpStream::connect((host, port)).await?.into_split();

    let read_task = tokio::spawn({
        let mut socket_rx = socket_rx;
        let shutdown = shutdown.clone();
        let rx = rx.clone();

        async move {
            loop {
                use shared::protocol::Message;

                if shutdown.load(Ordering::SeqCst) {
                    break;
                }

                let message = match shared::protocol::read(&mut socket_rx).await {
                    Some(Ok(Message::GraphChange(cmd))) => cmd,
                    Some(Ok(msg)) => panic!("unexpected message: {msg:?}"),
                    Some(Err(err)) => return Err(err),
                    None => break,
                };
                rx.lock().unwrap().push(message);
            }

            anyhow::Ok(())
        }
    });
    let write_task = tokio::spawn({
        let mut socket_tx = socket_tx;
        let shutdown = shutdown.clone();
        let tx = tx.clone();

        async move {
            loop {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }

                let to_send: Vec<_> = tx
                    .lock()
                    .map(|mut ms| ms.drain(..).collect())
                    .unwrap_or_default();
                for message in to_send {
                    shared::protocol::write(&mut socket_tx, message).await?;
                }
            }

            anyhow::Ok(())
        }
    });

    Ok((read_task, write_task))
}

fn synchronise_network_to_local(
    mut graph: ResMut<Graph>,
    network_state: Res<resources::NetworkState>,
) {
    let changes = &mut network_state.rx.lock().unwrap();
    if !changes.is_empty() {
        graph.apply_changes(changes);
        changes.clear();
    }
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
