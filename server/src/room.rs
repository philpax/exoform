use super::{
    peer::{PeerHandle, PeerMessage},
    util,
};
use shared::{Graph, GraphChange, GraphCommand};
use std::{collections::HashMap, net::SocketAddr};
use tokio::{sync::mpsc, task::JoinHandle};

pub struct Room {
    peers: HashMap<SocketAddr, PeerHandle>,
    _save_kicker_task: JoinHandle<anyhow::Result<()>>,
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

util::make_handle_type!(RoomHandle, RoomMessage);

impl RoomHandle {
    pub async fn new(filename: String) -> anyhow::Result<RoomHandle> {
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
