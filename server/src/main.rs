use std::{collections::HashMap, net::SocketAddr};

use clap::Parser;
use shared::{protocol::Message, Graph, GraphChange, GraphCommand};
use tokio::{net, sync::mpsc, task::JoinHandle};

macro_rules! make_handle_type {
    ($handle_name:ident, $message_type:ident) => {
        #[derive(Debug, Clone)]
        pub struct $handle_name(::tokio::sync::mpsc::Sender<$message_type>);
        impl $handle_name {
            pub async fn send(&self, msg: $message_type) -> anyhow::Result<()> {
                Ok(self.0.send(msg).await?)
            }
        }
    };
}

struct Peer {
    address: SocketAddr,
    receiver: mpsc::Receiver<PeerMessage>,
    _read_task: JoinHandle<anyhow::Result<()>>,
    _write_task: JoinHandle<anyhow::Result<()>>,
    write_sender: mpsc::Sender<Message>,
    coordinator: CoordinatorHandle,
    room: Option<RoomHandle>,
}
#[derive(Debug, Clone)]
pub enum PeerMessage {
    Disconnect,
    GraphCommand(GraphCommand),
    GraphChange(GraphChange),
    SetRoom(Option<RoomHandle>),
}
impl Peer {
    async fn handle_message(&mut self, msg: PeerMessage) -> anyhow::Result<()> {
        match msg {
            PeerMessage::Disconnect => {
                self.coordinator
                    .send(CoordinatorMessage::PeerLeave(self.address))
                    .await?
            }
            PeerMessage::GraphCommand(gc) => {
                if let Some(room) = &self.room {
                    room.send(RoomMessage::GraphCommand(gc)).await?;
                }
            }
            PeerMessage::GraphChange(gc) => {
                self.write_sender.send(Message::GraphChange(gc)).await?;
            }
            PeerMessage::SetRoom(room) => {
                self.room = room;
            }
        }
        Ok(())
    }
    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await.unwrap();
        }
    }
}
make_handle_type!(PeerHandle, PeerMessage);
impl PeerHandle {
    pub fn new(
        coordinator: CoordinatorHandle,
        stream: net::TcpStream,
        address: SocketAddr,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(8);

        let (mut read, mut write) = stream.into_split();
        let read_task = tokio::spawn({
            let sender = sender.clone();
            async move {
                loop {
                    let message = match shared::protocol::read(&mut read).await {
                        Some(Ok(Message::GraphCommand(cmd))) => cmd,
                        Some(Ok(msg)) => anyhow::bail!("unexpected message: {msg:?}"),
                        Some(Err(err)) => return Err(err),
                        None => {
                            sender.send(PeerMessage::Disconnect).await?;
                            break;
                        }
                    };
                    sender.send(PeerMessage::GraphCommand(message)).await?;
                }

                anyhow::Ok(())
            }
        });

        let (write_sender, mut write_receiver) = mpsc::channel(8);
        let write_task = tokio::spawn(async move {
            while let Some(message) = write_receiver.recv().await {
                shared::protocol::write(&mut write, message).await?;
            }

            anyhow::Ok(())
        });

        let mut peer = Peer {
            address,
            receiver,
            _read_task: read_task,
            _write_task: write_task,
            write_sender,
            coordinator,
            room: None,
        };
        tokio::spawn(async move { peer.run().await });

        Self(sender)
    }
}

pub struct Room {
    peers: HashMap<SocketAddr, PeerHandle>,
    _save_kicker_task: JoinHandle<Result<(), anyhow::Error>>,
    graph: Graph,
    receiver: mpsc::Receiver<RoomMessage>,
}
#[derive(Debug, Clone)]
pub enum RoomMessage {
    PeerJoin(SocketAddr, PeerHandle),
    PeerLeave(SocketAddr),
    GraphCommand(GraphCommand),
    Save(String),
}
impl Room {
    async fn handle_message(&mut self, msg: RoomMessage) -> anyhow::Result<()> {
        match msg {
            RoomMessage::PeerJoin(address, peer) => {
                peer.send(PeerMessage::GraphChange(GraphChange::Initialize(
                    self.graph.to_components(),
                )))
                .await?;
                self.peers.insert(address, peer);
            }
            RoomMessage::PeerLeave(address) => {
                if let Some(peer) = self.peers.remove(&address) {
                    peer.send(PeerMessage::SetRoom(None)).await?;
                }
            }
            RoomMessage::GraphCommand(gc) => {
                let changes = self.graph.apply_command(&gc);
                for change in changes {
                    for peer in self.peers.values() {
                        peer.send(PeerMessage::GraphChange(change.clone())).await?;
                    }
                }
            }
            RoomMessage::Save(filename) => {
                tokio::fs::write(filename, serde_json::to_string_pretty(&self.graph)?).await?;
            }
        }
        Ok(())
    }
    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await.unwrap();
        }
    }
}
make_handle_type!(RoomHandle, RoomMessage);
impl RoomHandle {
    async fn new(filename: String) -> anyhow::Result<RoomHandle> {
        let (sender, receiver) = mpsc::channel(8);

        let graph = match tokio::fs::read_to_string(&filename).await {
            Ok(contents) => serde_json::from_str(&contents)?,
            _ => Graph::new_authoritative(),
        };

        let save_kicker_task = tokio::spawn({
            let sender = sender.clone();
            async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    sender.send(RoomMessage::Save(filename.clone())).await?;
                }

                #[allow(unreachable_code)]
                anyhow::Ok(())
            }
        });

        let mut room = Room {
            peers: HashMap::new(),
            _save_kicker_task: save_kicker_task,
            graph,
            receiver,
        };
        tokio::spawn(async move { room.run().await });

        Ok(RoomHandle(sender))
    }
}

pub struct Coordinator {
    peers: HashMap<SocketAddr, PeerHandle>,
    receiver: mpsc::Receiver<CoordinatorMessage>,
    _listener_task: JoinHandle<anyhow::Result<()>>,
    room: RoomHandle,
}
#[derive(Debug, Clone)]
pub enum CoordinatorMessage {
    PeerJoin(SocketAddr, PeerHandle),
    PeerLeave(SocketAddr),
}
impl Coordinator {
    async fn new(host: &str, port: u16, filename: String) -> anyhow::Result<Self> {
        let (sender, receiver) = mpsc::channel(8);

        let listener_task = tokio::spawn({
            let sender = sender.clone();
            let host = host.to_owned();
            async move {
                let listener = net::TcpListener::bind((host, port)).await?;
                loop {
                    let (stream, address) = listener.accept().await?;
                    let peer = PeerHandle::new(CoordinatorHandle(sender.clone()), stream, address);
                    sender
                        .send(CoordinatorMessage::PeerJoin(address, peer))
                        .await?;
                }

                #[allow(unreachable_code)]
                anyhow::Ok(())
            }
        });

        Ok(Self {
            peers: HashMap::new(),
            receiver,
            _listener_task: listener_task,
            room: RoomHandle::new(filename).await?,
        })
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                CoordinatorMessage::PeerJoin(addr, peer) => {
                    peer.send(PeerMessage::SetRoom(Some(self.room.clone())))
                        .await?;
                    self.room
                        .send(RoomMessage::PeerJoin(addr, peer.clone()))
                        .await?;
                    self.peers.insert(addr, peer);
                    println!("{addr:?}: joined");
                }
                CoordinatorMessage::PeerLeave(addr) => {
                    self.room.send(RoomMessage::PeerLeave(addr)).await?;
                    self.peers.remove(&addr);
                    println!("{addr:?}: left");
                }
            }
        }

        anyhow::Ok(())
    }
}
make_handle_type!(CoordinatorHandle, CoordinatorMessage);

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

    let mut coordinator = Coordinator::new(&args.host, port, args.filename).await?;
    coordinator.run().await?;

    Ok(())
}
