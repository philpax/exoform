use std::net::SocketAddr;

use super::{
    coordinator::{CoordinatorHandle, CoordinatorMessage},
    room::{RoomHandle, RoomMessage},
    util,
};
use tokio::{net, sync::mpsc, task::JoinHandle};

use shared::{
    protocol::{Message, RequestJoin},
    GraphChange, GraphCommand,
};

pub struct Peer {
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
    RequestJoin(RequestJoin),
    Disconnect,
    GraphCommand(GraphCommand),
    GraphChange(GraphChange),
    SetRoom(Option<RoomHandle>),
}

impl Peer {
    async fn handle_message(&mut self, msg: PeerMessage) -> anyhow::Result<()> {
        match msg {
            PeerMessage::RequestJoin(req) => {
                self.coordinator
                    .send(CoordinatorMessage::PeerJoinRoom(self.address, req.room))
                    .await?
            }
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
                if let Some(room) = &self.room {
                    room.send(RoomMessage::PeerLeave(self.address)).await?;
                }
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

util::make_handle_type!(PeerHandle, PeerMessage);
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
                        Some(Ok(Message::RequestJoin(req))) => PeerMessage::RequestJoin(req),
                        Some(Ok(Message::GraphCommand(cmd))) => PeerMessage::GraphCommand(cmd),
                        Some(Ok(msg)) => anyhow::bail!("unexpected message: {msg:?}"),
                        Some(Err(err)) => return Err(err),
                        None => {
                            sender.send(PeerMessage::Disconnect).await?;
                            break;
                        }
                    };
                    sender.send(message).await?;
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
