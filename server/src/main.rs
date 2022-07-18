use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use clap::Parser;
use shared::{Graph, NodeData};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net,
};

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

    let (command_tx, mut command_rx) = tokio::sync::mpsc::channel(128);

    let graph = match tokio::fs::read_to_string(&args.filename).await {
        Ok(contents) => serde_json::from_str(&contents)?,
        _ => Graph::new_authoritative(NodeData::Union(Default::default())),
    };
    let (graph_tx, graph_rx) = tokio::sync::watch::channel(graph.to_components());
    let _graph_task = tokio::spawn(async move {
        let graph = Arc::new(Mutex::new(graph));

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

        while let Some(command) = command_rx.recv().await {
            if let Ok(mut graph) = graph.lock() {
                graph.apply_commands(&[command]);
                graph_tx.send(graph.to_components())?;
            }
        }

        anyhow::Ok(())
    });

    let listener = net::TcpListener::bind((args.host.as_ref(), port)).await?;
    println!("Listening on {}:{}", args.host, port);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        let (read, mut write) = socket.into_split();
        let command_tx = command_tx.clone();
        let mut graph_rx = graph_rx.clone();
        let connected = Arc::new(AtomicBool::new(true));

        println!("{}: new connection", peer_addr);
        let value = graph_rx.borrow().clone();
        write
            .write_all(format!("{}\n", serde_json::to_string(&value)?).as_bytes())
            .await?;

        let _peer_read_task = tokio::spawn({
            let connected = connected.clone();
            async move {
                let mut reader = BufReader::new(read);

                loop {
                    let mut buf = String::new();
                    let n = reader.read_line(&mut buf).await?;
                    if n == 0 {
                        break;
                    }
                    let buf = buf.trim();

                    println!("{}: {buf}", peer_addr);
                    command_tx.send(serde_json::from_str(buf)?).await?;
                }

                println!("{}: disconnected", peer_addr);
                connected.store(false, Ordering::SeqCst);
                anyhow::Ok(())
            }
        });

        let _peer_write_task = tokio::spawn(async move {
            while graph_rx.changed().await.is_ok() && connected.load(Ordering::SeqCst) {
                let value = graph_rx.borrow().clone();
                write
                    .write_all(format!("{}\n", serde_json::to_string(&value)?).as_bytes())
                    .await?;
            }
            anyhow::Ok(())
        });
    }
}
