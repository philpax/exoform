use std::io::{self as io, Write};

use anyhow::Context;
use lunatic::{
    net,
    process::{AbstractProcess, Message, ProcessMessage, ProcessRef, Request},
    Mailbox, Process,
};
use serde::{Deserialize, Serialize};

use crate::file_store::FileStore;

#[derive(Serialize, Deserialize, Clone)]
pub struct HttpRequest(pub String, pub String);

pub struct ClientProcess {
    stream: net::TcpStream,
    files: ProcessRef<FileStore>,
}

impl ClientProcess {
    pub fn respond(
        &mut self,
        (status, reason): (u16, &str),
        mime_type: String,
        body: &[u8],
    ) -> anyhow::Result<()> {
        http_tiny::Header::new(
            http_tiny::HeaderStartLine::new_response(status, reason),
            http_tiny::HeaderFields::from_iter([
                ("Content-Type", mime_type),
                ("Content-Length", body.len().to_string()),
            ]),
        )
        .write_all(&mut self.stream)
        .ok()
        .context("failed to write header")?;
        self.stream.write_all(&body)?;

        Ok(())
    }
}

impl AbstractProcess for ClientProcess {
    type Arg = (net::TcpStream, ProcessRef<FileStore>);
    type State = Self;

    fn init(this: ProcessRef<Self>, (stream, files): Self::Arg) -> Self::State {
        Process::spawn_link(
            (this.clone(), stream.clone()),
            |(client, stream), _: Mailbox<()>| {
                let reader = io::BufReader::new(stream);
                let mut reader = http_tiny::Limiter::new(reader, 4096, 4096);

                loop {
                    let (method, target) = {
                        let header = http_tiny::Header::read(&mut reader).unwrap();
                        let start_line = header.start_line();

                        (
                            String::from_utf8_lossy(start_line.request_method()).into_owned(),
                            String::from_utf8_lossy(start_line.request_target())
                                .strip_prefix("/")
                                .unwrap_or_default()
                                .to_string(),
                        )
                    };

                    client.send(HttpRequest(method, target));
                }
            },
        );

        ClientProcess { stream, files }
    }
}

impl ProcessMessage<HttpRequest> for ClientProcess {
    fn handle(state: &mut Self::State, HttpRequest(method, target): HttpRequest) {
        if method == "GET" {
            let target = if target.is_empty() {
                "index.html".to_owned()
            } else {
                target
            };

            if let Some((mime_type, data)) = state.files.request(target) {
                return state.respond((200, "OK"), mime_type, &data).unwrap();
            }
        }

        state
            .respond((404, "Not Found"), "text/html".to_string(), {
                use malvolio::prelude::*;

                html()
                    .head(head().child(title("There be dragons here")))
                    .body(body().h1("404 Not Found"))
                    .to_string()
                    .as_bytes()
            })
            .unwrap();

        std::process::exit(1);
    }
}
