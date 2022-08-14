use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{GraphChange, GraphCommand};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RequestJoin {
    pub room: String,
}

// TODO: consider splitting this up into PeerOutgoingMessage and PeerIncomingMessage
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Message {
    RequestJoin(RequestJoin),
    GraphCommand(GraphCommand),
    GraphChange(GraphChange),
}
impl From<RequestJoin> for Message {
    fn from(req: RequestJoin) -> Self {
        Self::RequestJoin(req)
    }
}
impl From<GraphCommand> for Message {
    fn from(cmd: GraphCommand) -> Self {
        Self::GraphCommand(cmd)
    }
}
impl From<GraphChange> for Message {
    fn from(change: GraphChange) -> Self {
        Self::GraphChange(change)
    }
}

pub async fn write<W: AsyncWrite + Unpin>(writer: &mut W, payload: Message) -> anyhow::Result<()> {
    let buf = bincode::serialize(&payload)?;
    let len: u32 = buf.len().try_into()?;
    writer.write_u32(len).await?;
    Ok(writer.write_all(&buf).await?)
}

pub async fn read<'a, R: AsyncRead + Unpin>(reader: &mut R) -> Option<anyhow::Result<Message>> {
    let size = match reader.read_u32().await {
        Ok(size) => size,
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            return None;
        }
        Err(e) => return Some(Err(e.into())),
    };
    Some(
        async {
            let mut buf = vec![0u8; size.try_into()?];
            reader.read_exact(&mut buf).await?;
            Ok(bincode::deserialize(&buf)?)
        }
        .await,
    )
}
