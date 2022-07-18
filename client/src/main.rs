use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use clap::Parser;

use shared::{Graph, Node, NodeId};

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
    tx: Arc<Mutex<Vec<shared::GraphEvent>>>,
    rx: Arc<Mutex<Option<(HashMap<NodeId, Node>, Option<NodeId>)>>>,
    _write_thread: std::thread::JoinHandle<anyhow::Result<()>>,
    _read_thread: std::thread::JoinHandle<anyhow::Result<()>>,
}
impl NetworkState {
    pub fn send(&mut self, events: &[shared::GraphEvent]) {
        self.tx.lock().unwrap().extend_from_slice(events);
    }
}
impl Drop for NetworkState {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

pub fn main() -> anyhow::Result<()> {
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
    let socket = std::net::TcpStream::connect((args.host.as_ref(), port)).unwrap();

    let (tx, rx) = (Arc::new(Mutex::new(vec![])), Arc::new(Mutex::new(None)));
    let write_thread = std::thread::spawn({
        let tx = tx.clone();
        let shutdown = shutdown.clone();

        let mut socket = socket.try_clone()?;
        move || {
            use std::io::Write;

            loop {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }

                let mut to_send = vec![];
                to_send.append(&mut tx.lock().unwrap());

                for event in to_send {
                    println!("{:?}", event);
                    writeln!(socket, "{}", serde_json::to_string(&event)?)?;
                }
            }

            anyhow::Ok(())
        }
    });
    let read_thread = std::thread::spawn({
        let rx = rx.clone();
        let shutdown = shutdown.clone();
        let socket = socket.try_clone()?;
        move || {
            let mut reader = BufReader::new(socket);

            loop {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }

                let mut buf = String::new();
                let n = reader.read_line(&mut buf)?;
                if n == 0 {
                    break;
                }
                let buf = buf.trim();

                println!("network: {buf}");
                *rx.lock().unwrap() = Some(serde_json::from_str(buf)?);
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
            _write_thread: write_thread,
            _read_thread: read_thread,
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
    if let Some((nodes, root_node_id)) = network_state.rx.lock().unwrap().clone() {
        *graph = Graph::from_components(nodes, root_node_id)
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
