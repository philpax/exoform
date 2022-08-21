use crate::coordinator::{CoordinatorHandle, CoordinatorMessage};

use super::{
    peer::{PeerHandle, PeerMessage},
    util,
};
use shared::{Graph, GraphChange, GraphCommand};
use std::{collections::HashMap, net::SocketAddr, path::PathBuf};
use tokio::{sync::mpsc, task::JoinHandle};

pub struct Room {
    name: String,
    peers: HashMap<SocketAddr, PeerHandle>,
    _save_kicker_task: JoinHandle<anyhow::Result<()>>,
    graph: Graph,
    receiver: mpsc::Receiver<RoomMessage>,
    coordinator: CoordinatorHandle,
}

#[derive(Debug, Clone)]
pub enum RoomMessage {
    PeerJoin(SocketAddr, PeerHandle),
    PeerLeave(SocketAddr),
    GraphCommand(GraphCommand),
    Save,
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
                println!("room {:?}: {:?} joined", self.name, address);
            }
            RoomMessage::PeerLeave(address) => {
                self.peers.remove(&address);
                println!("room {:?}: {:?} left", self.name, address);

                if self.peers.is_empty() {
                    self.coordinator
                        .send(CoordinatorMessage::RoomShutdown(self.name.clone()))
                        .await?;
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
            RoomMessage::Save => {
                self.save().await?;
            }
        }
        Ok(())
    }
    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await.unwrap();
        }
    }

    fn path(&self) -> PathBuf {
        PathBuf::from("models")
            .join(&self.name)
            .with_extension("json")
    }

    async fn load(&mut self) -> anyhow::Result<()> {
        if let Ok(contents) = tokio::fs::read_to_string(self.path()).await {
            self.graph = serde_json::from_str(&contents)?;
        }
        Ok(())
    }
    async fn save(&mut self) -> anyhow::Result<()> {
        if let Some(path) = self.path().parent() {
            tokio::fs::create_dir_all(path).await?;
        }
        Ok(tokio::fs::write(self.path(), serde_json::to_string_pretty(&self.graph)?).await?)
    }
}

util::make_handle_type!(RoomHandle, RoomMessage);

impl RoomHandle {
    pub fn new(name: String, coordinator: CoordinatorHandle) -> RoomHandle {
        let (sender, receiver) = mpsc::channel(8);

        let graph = Graph::new_authoritative();

        let save_kicker_task = tokio::spawn({
            let sender = sender.clone();
            async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    sender.send(RoomMessage::Save).await?;
                }

                #[allow(unreachable_code)]
                anyhow::Ok(())
            }
        });

        let mut room = Room {
            name,
            peers: HashMap::new(),
            _save_kicker_task: save_kicker_task,
            graph,
            receiver,
            coordinator,
        };
        tokio::spawn(async move {
            room.load().await.unwrap();
            room.run().await;
        });

        RoomHandle(sender)
    }
}
