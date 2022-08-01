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

pub(super) use make_handle_type;
