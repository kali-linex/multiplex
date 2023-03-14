use std::{net::SocketAddr, io, sync::Arc};

use clap::Parser;
use log::{info, error};
use tokio::{net::{TcpListener, TcpStream}, spawn, sync::{self, RwLock, broadcast::{Sender, Receiver}}, select};

mod cli;
#[tokio::main]
async fn main() -> io::Result<()> {
    let args = cli::Cli::parse();
    cli::init_logger();

    let listen_addr = SocketAddr::new(args.listen_address, args.port);
    let listener = TcpListener::bind(listen_addr).await?;
    let client = TcpStream::connect(args.target).await?;
    let (soci_tx, mut soci_rx) = sync::broadcast::channel(64);
    let (sico_tx, mut sico_rx) = sync::broadcast::channel::<Arc<RwLock<Vec<u8>>>>(64);
    let soci_tx2 = soci_tx.clone();

    info!("Listening on {listen_addr:?}");

    spawn(async move {
        loop {
            select! {
                _ = client.readable() => {
                    let mut buf = vec![0; 1024];
                    match client.try_read(&mut buf) {
                        Ok(n) => {
                            buf.truncate(n);
                            let mutex = Arc::new(RwLock::new(buf));
                            soci_tx.send(mutex);
                        }
                        Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => { continue; }
                        Err(e) => {
                            error!("in connection thread: {}", e);
                        }
                    }
                }
                data = sico_rx.recv() => {
                    client.writable().await;

                    match client.try_write(&data.unwrap().read().await) {
                        Ok(_) => {}
                        Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {continue;}
                        Err(e) => {
                            error!("in connection thread: {}", e);
                        }
                    }
                }
            }
        }
    });
    
    loop {
        let (socket, a) = listener.accept().await?;
        info!("Client connected: {a:?}");
        spawn(client_handler(sico_tx.clone(), soci_tx2.subscribe(), socket));
    }
}

async fn client_handler(sico_tx: Sender<Arc<RwLock<Vec<u8>>>>, mut soci_rx: Receiver<Arc<RwLock<Vec<u8>>>>, stream: TcpStream) {
    loop {
            select! {
                _ = stream.readable() => {
                    let mut buf = vec![0; 1024];
                    match stream.try_read(&mut buf) {
                        Ok(n) => {
                            buf.truncate(n);
                            let mutex = Arc::new(RwLock::new(buf));
                            sico_tx.send(mutex);
                        }
                        Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => { continue; }
                        Err(e) => {
                            error!("in client thread: {}", e);
                        }
                    }
                }
                data = soci_rx.recv() => {
                    stream.writable().await;

                    match stream.try_write(&data.unwrap().read().await) {
                        Ok(_) => {}
                        Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {continue;}
                        Err(e) => {
                            error!("in client thread: {}", e);
                        }
                    }
                }
            }
    }
}