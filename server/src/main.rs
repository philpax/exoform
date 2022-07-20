use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use clap::Parser;
use shared::{Graph, GraphChange, NodeData};
use tokio::{net, sync};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[derive(Parser)]
    #[clap(author, version, about, long_about = None)]
    struct Args {
        #[clap(short, long, default_value_t = String::from("localhost"))]
        host: String,
        #[clap(short, long)]
        port: Option<u16>,
        #[clap(default_value_t = String::from("graph.json"))]
        filename: String,
    }

    let args = Args::parse();
    let port = args.port.unwrap_or(shared::DEFAULT_PORT);

    let (command_tx, mut command_rx) = sync::mpsc::channel(128);

    let graph = match tokio::fs::read_to_string(&args.filename).await {
        Ok(contents) => serde_json::from_str(&contents)?,
        _ => Graph::new_authoritative(NodeData::Union(Default::default())),
    };
    let graph = Arc::new(Mutex::new(graph));
    let (graph_tx, graph_rx) = sync::watch::channel(vec![]);
    let _graph_broadcast_task = tokio::spawn({
        let graph = graph.clone();
        async move {
            while let Some(command) = command_rx.recv().await {
                if let Ok(mut graph) = graph.lock() {
                    graph_tx.send(graph.apply_commands(&[command]))?;
                }
            }

            anyhow::Ok(())
        }
    });
    let _graph_save_task = tokio::spawn({
        let graph = graph.clone();
        let filename = args.filename.clone();
        async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                interval.tick().await;

                let contents = match graph.lock() {
                    Ok(graph) => serde_json::to_string_pretty(&*graph)?,
                    _ => continue,
                };
                tokio::fs::write(&filename, contents).await?;
            }

            #[allow(unreachable_code)]
            anyhow::Ok(())
        }
    });

    let listener = net::TcpListener::bind((args.host.as_ref(), port)).await?;
    println!("Listening on {}:{}", args.host, port);

    loop {
        let (socket, peer_addr) = listener.accept().await?;

        handle_peer(
            socket,
            peer_addr,
            command_tx.clone(),
            graph_rx.clone(),
            graph.clone(),
        )
        .await?;
    }
}

async fn handle_peer(
    socket: net::TcpStream,
    peer_addr: std::net::SocketAddr,
    command_tx: sync::mpsc::Sender<shared::GraphCommand>,
    mut graph_rx: sync::watch::Receiver<Vec<GraphChange>>,
    graph: Arc<Mutex<Graph>>,
) -> anyhow::Result<()> {
    let (mut read, mut write) = socket.into_split();

    let connected = Arc::new(AtomicBool::new(true));
    println!("{}: new connection", peer_addr);

    if graph_rx.has_changed()? {
        // clear the change flag if necessary - we're about to initialise this peer
        graph_rx.changed().await?;
    }
    shared::protocol::write(
        &mut write,
        &vec![GraphChange::Initialize(
            graph.lock().unwrap().to_components(),
        )],
    )
    .await?;

    let _peer_read_task = tokio::spawn({
        let connected = connected.clone();
        async move {
            loop {
                let message = match shared::protocol::read(&mut read).await {
                    Some(cmd) => cmd?,
                    None => break,
                };
                command_tx.send(message).await?;
            }

            println!("{}: disconnected", peer_addr);
            connected.store(false, Ordering::SeqCst);

            anyhow::Ok(())
        }
    });

    let _peer_write_task = tokio::spawn({
        async move {
            while connected.load(Ordering::SeqCst) {
                graph_rx.changed().await?;
                let payload = graph_rx.borrow().to_owned();
                shared::protocol::write(&mut write, &payload).await?;
            }
            anyhow::Ok(())
        }
    });

    Ok(())
}
