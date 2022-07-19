use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use clap::Parser;

use shared::Graph;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

mod camera;
mod mesh_generation;
mod ui;

#[derive(Default)]
pub struct RenderParameters {
    pub wireframe: bool,
}

#[derive(Default)]
pub struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    _bottom: f32,
}

pub struct NetworkState {
    shutdown: Arc<AtomicBool>,
    tx: Arc<Mutex<Vec<shared::GraphCommand>>>,
    rx: Arc<Mutex<Vec<shared::GraphChange>>>,
}
impl NetworkState {
    pub fn send(&mut self, commands: &[shared::GraphCommand]) {
        self.tx.lock().unwrap().extend_from_slice(commands);
    }
}
impl Drop for NetworkState {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

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
    }

    let args = Args::parse();
    let port = args.port.unwrap_or(shared::DEFAULT_PORT);

    let shutdown = Arc::new(AtomicBool::new(false));
    let (socket_rx, mut socket_tx) = TcpStream::connect((args.host.as_ref(), port))
        .await?
        .into_split();

    let (rx, tx) = (Arc::new(Mutex::new(vec![])), Arc::new(Mutex::new(vec![])));
    let _read_task = tokio::spawn({
        let shutdown = shutdown.clone();
        let mut reader = BufReader::new(socket_rx);
        let rx = rx.clone();

        async move {
            loop {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }

                let mut buf = String::new();
                let n = reader.read_line(&mut buf).await?;
                if n == 0 {
                    break;
                }
                rx.lock()
                    .unwrap()
                    .append(&mut serde_json::from_str(buf.trim())?);
            }

            anyhow::Ok(())
        }
    });
    let _write_task = tokio::spawn({
        let shutdown = shutdown.clone();
        let tx = tx.clone();

        async move {
            loop {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }

                let to_send = tx
                    .lock()
                    .map(|mut v| {
                        let ret = v.clone();
                        v.clear();
                        ret
                    })
                    .unwrap_or_default();

                for command in to_send {
                    socket_tx
                        .write_all(format!("{}\n", serde_json::to_string(&command)?).as_bytes())
                        .await?;
                }
            }

            anyhow::Ok(())
        }
    });

    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .insert_resource(shared::Graph::new_client())
        .insert_resource(WindowDescriptor {
            width: 1600.,
            height: 900.,
            title: format!("Exoform {}", env!("CARGO_PKG_VERSION")),
            ..Default::default()
        })
        .insert_resource(NetworkState {
            shutdown: shutdown.clone(),
            tx,
            rx,
        })
        .init_resource::<OccupiedScreenSpace>()
        .init_resource::<RenderParameters>()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy::pbr::wireframe::WireframePlugin)
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

fn synchronise_network_to_local(mut graph: ResMut<Graph>, network_state: Res<NetworkState>) {
    let changes = &mut network_state.rx.lock().unwrap();
    graph.apply_changes(changes);
    changes.clear();
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
