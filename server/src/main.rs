#[cfg(target_family = "wasm")]
mod file_store;

#[cfg(target_family = "wasm")]
mod http;

#[cfg(target_family = "wasm")]
#[lunatic::main]
fn main(_: lunatic::Mailbox<()>) {
    use clap::Parser;
    use lunatic::{net, process::StartProcess};

    #[derive(Parser)]
    #[clap(author, version, about, long_about = None)]
    struct Args {
        #[clap(short, long, default_value_t = 8080)]
        port: u16,
    }

    let Args { port } = Args::parse();
    println!("Starting server on port {}", port);

    let file_store = file_store::FileStore::start((), None);

    let listener = net::TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    while let Ok((stream, _)) = listener.accept() {
        http::ClientProcess::start((stream, file_store.clone()), None);
    }
}

#[cfg(not(target_family = "wasm"))]
fn main() {}
