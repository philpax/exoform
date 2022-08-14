use super::{
    peer::{PeerHandle, PeerMessage},
    room::{RoomHandle, RoomMessage},
    util,
};
use std::{collections::HashMap, net::SocketAddr};
use tokio::{sync::mpsc, task::JoinHandle};

pub struct Coordinator {
    peers: HashMap<SocketAddr, PeerHandle>,
    receiver: mpsc::Receiver<CoordinatorMessage>,
    _listener_task: JoinHandle<anyhow::Result<()>>,
    rooms: HashMap<String, RoomHandle>,
}

#[derive(Debug, Clone)]
pub enum CoordinatorMessage {
    PeerJoin(SocketAddr, PeerHandle),
    PeerLeave(SocketAddr),
    PeerJoinRoom(SocketAddr, String),
}

impl Coordinator {
    async fn new(host: &str, port: u16) -> anyhow::Result<Self> {
        let (sender, receiver) = mpsc::channel(8);

        let listener_task = tokio::spawn({
            let sender = sender.clone();
            let host = host.to_owned();
            async move {
                let listener = tokio::net::TcpListener::bind((host, port)).await?;
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
            rooms: HashMap::new(),
        })
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                CoordinatorMessage::PeerJoin(addr, peer) => {
                    self.peers.insert(addr, peer);
                    println!("peer {addr:?}: joined");
                }
                CoordinatorMessage::PeerLeave(addr) => {
                    let peer = self
                        .peers
                        .get(&addr)
                        .cloned()
                        .expect("received peer leave request from untracked peer");
                    peer.send(PeerMessage::SetRoom(None)).await?;
                    self.peers.remove(&addr);
                    println!("peer {addr:?}: left");
                }
                CoordinatorMessage::PeerJoinRoom(addr, room_name) => {
                    let peer = self
                        .peers
                        .get(&addr)
                        .cloned()
                        .expect("received peer join request from untracked peer");

                    let room = self
                        .rooms
                        .entry(room_name.clone())
                        .or_insert_with(|| RoomHandle::new(room_name));

                    peer.send(PeerMessage::SetRoom(Some(room.clone()))).await?;
                    room.send(RoomMessage::PeerJoin(addr, peer.clone())).await?;
                }
            }
        }

        anyhow::Ok(())
    }

    pub async fn coordinate(host: &str, port: u16) -> anyhow::Result<()> {
        let mut coordinator = Coordinator::new(host, port).await?;
        coordinator.run().await
    }
}

util::make_handle_type!(CoordinatorHandle, CoordinatorMessage);
