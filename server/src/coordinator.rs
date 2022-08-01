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

    pub async fn coordinate(host: &str, port: u16, filename: String) -> anyhow::Result<()> {
        let mut coordinator = Coordinator::new(host, port, filename).await?;
        coordinator.run().await
    }
}

util::make_handle_type!(CoordinatorHandle, CoordinatorMessage);
