mod coordinator;
mod peer;
mod room;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use clap::Parser;

    #[derive(Parser)]
    #[clap(author, version, about, long_about = None)]
    struct Args {
        #[clap(short, long, default_value_t = String::from("localhost"))]
        host: String,
        #[clap(short, long)]
        port: Option<u16>,
    }

    let args = Args::parse();
    let port = args.port.unwrap_or(shared::DEFAULT_PORT);

    coordinator::Coordinator::coordinate(&args.host, port).await
}
