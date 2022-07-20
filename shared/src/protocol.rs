use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn write<T: Serialize, W: AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &T,
) -> anyhow::Result<()> {
    let buf = bincode::serialize(&payload)?;
    let len: u32 = buf.len().try_into()?;
    writer.write_u32(len).await?;
    Ok(writer.write_all(&buf).await?)
}

pub async fn read<'a, T: DeserializeOwned, R: AsyncRead + Unpin>(
    reader: &mut R,
) -> Option<anyhow::Result<T>> {
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
            Ok(bincode::deserialize::<T>(&buf)?)
        }
        .await,
    )
}
