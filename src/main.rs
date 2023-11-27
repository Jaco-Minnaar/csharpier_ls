use std::str::FromStr;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    net::TcpListener,
};
use tower_lsp::{LspService, Server};

use crate::{logger::create_logger, server::CSharpierLanguageServer};

mod buffer;
mod logger;
mod processes;
mod server;

#[derive(Debug, ValueEnum, Clone, Copy)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Into<log::LevelFilter> for LogLevel {
    fn into(self) -> log::LevelFilter {
        match self {
            Self::Trace => log::LevelFilter::Trace,
            Self::Debug => log::LevelFilter::Debug,
            Self::Info => log::LevelFilter::Info,
            Self::Warn => log::LevelFilter::Warn,
            Self::Error => log::LevelFilter::Error,
        }
    }
}

#[derive(Debug, ValueEnum, Clone, Copy)]
enum LspCommsMode {
    Stdio,
    Tcp,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The log level to use
    #[arg(value_enum, long, default_value = "info")]
    log_level: LogLevel,

    /// The mode of communication to use
    #[arg(value_enum, short, long, default_value = "stdio")]
    mode: LspCommsMode,
}

async fn start_server<I, O>(input: I, output: O) -> Result<()>
where
    I: AsyncRead + Unpin,
    O: AsyncWrite,
{
    log::info!("Starting LSP Service");

    let (service, socket) = LspService::new(|client| CSharpierLanguageServer::new(client));

    log::info!("Starting Server");

    Server::new(input, output, socket).serve(service).await;

    log::info!("Shutting down CSharpier Language Server");

    Ok(())
}

async fn start_stdio_server() -> Result<()> {
    log::info!("Starting CSharpier LS in stdio mode.");

    start_server(io::stdin(), io::stdout()).await
}

async fn start_tcp_server() -> Result<()> {
    log::info!("Starting CSharpier LS in tcp mode.");

    let listener = TcpListener::bind("127.0.0.1:50051").await?;

    log::info!("TCP Listener binded to port 50051");

    loop {
        let (socket, _) = listener.accept().await?;

        let addr = socket
            .peer_addr()
            .map_or(String::from_str("{}")?, |addr| addr.to_string());
        log::info!("Accepted connection from {}", addr);

        let (reader, writer) = io::split(socket);

        tokio::spawn(async move {
            if let Err(err) = start_server(reader, writer).await {
                log::error!("Error in TCP Server connection: {}", err)
            }

            log::info!("Connection to {} closed", addr);
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    create_logger("csharpier_ls.log", args.log_level.into()).expect("Failed to create logger");

    match args.mode {
        LspCommsMode::Tcp => start_tcp_server().await,
        LspCommsMode::Stdio => start_stdio_server().await,
    }
}
