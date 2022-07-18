use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use clap::Parser;
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
    }

    let args = Args::parse();
    let port = args.port.unwrap_or(shared::DEFAULT_PORT);

    let mut graph = shared::Graph::new_authoritative(shared::NodeData::Union(shared::Union::new()));
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(128);
    let (graph_tx, graph_rx) = tokio::sync::watch::channel(graph.to_components());
    tokio::spawn({
        async move {
            while let Some(event) = event_rx.recv().await {
                graph.apply_events(&[event]);
                graph_tx.send(graph.to_components())?;
            }
            anyhow::Ok(())
        }
    });

    let listener = net::TcpListener::bind((args.host.as_ref(), port)).await?;
    println!("Listening on {}:{}", args.host, port);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        let (read, mut write) = socket.into_split();
        let event_tx = event_tx.clone();
        let mut graph_rx = graph_rx.clone();
        let connected = Arc::new(AtomicBool::new(true));

        println!("{}: new connection", peer_addr);
        let value = graph_rx.borrow().clone();
        write
            .write_all(format!("{}\n", serde_json::to_string(&value)?).as_bytes())
            .await?;

        tokio::spawn({
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
                    let event = serde_json::from_str(buf)?;
                    event_tx.send(event).await?;
                }

                println!("{}: disconnected", peer_addr);
                connected.store(false, Ordering::SeqCst);
                anyhow::Ok(())
            }
        });

        tokio::spawn(async move {
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
