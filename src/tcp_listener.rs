use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::net::tcp::OwnedWriteHalf;
// use tokio_util::codec::{Framed, LinesCodec};
use uuid::Uuid;
use tokio_codec::{FramedRead, LinesCodec};
use tokio_util::codec::Framed;

type TcpConnections = Arc<DashMap<Uuid, OwnedWriteHalf>>;

pub struct TcpServer {
    port: u16,
    address: &'static str,
    connections: TcpConnections,
}

impl TcpServer {
    pub fn new(port: u16, address: &'static str) -> Self {
        TcpServer { port, address, connections: Arc::new(DashMap::new()) }
    }

    pub async fn connect(&self) -> Result<(), tokio::io::Error> {
        let addr = format!("{}:{}", self.address, self.port).parse().unwrap();
        let socket = TcpSocket::new_v4()?;
        socket.set_reuseaddr(true);
        socket.set_reuseport(true);
        socket.bind(addr)?;

        let listener = socket.listen(128)?;
        let clients = Arc::clone(&self.connections);

        tokio::spawn(async move {
            connection_handler(clients, listener).await;
        });
        println!("Here");
        self.on_start().await;
        Ok(())
    }

    async fn on_start(&self) {
        let mut now = Instant::now();
        loop {

            let instant_now = Instant::now();
            if instant_now.duration_since(now) > Duration::from_millis(2000) {
                for mut connection in self.connections.iter_mut() {
                    connection.writable().await;
                    let mut client = connection.try_write(b"").unwrap();
                }
                now = instant_now;
            }
        }
    }
}


async fn match_listener(map: TcpConnections, stream: TcpStream, socket_address: SocketAddr) -> Result<(), Box<dyn Error>> {
    println!("Someone connected");
    let id = Uuid::new_v4();
    let (mut stream, mut sink) = stream.into_split();
    // let mut lines: Framed<TcpStream, LinesCodec> = Framed::new(stream, LinesCodec::new());
    // lines.get_ref().try_write(b"Message");
    // lines.send("Hi there!").await.unwrap();
    map.entry(id).or_insert(sink);


    tokio::spawn(async move {
        let map = Arc::clone(&map);
        loop {
            let mut buf = [0; 4096];
            // let tcp = lines.get_ref();
            stream.readable().await;
            match stream.try_read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let message = std::str::from_utf8(&buf[..n]).unwrap();
                    for connection in map.iter_mut() {
                        if *connection.key() != id.clone() {
                            connection.writable().await;
                            let mut client = connection.try_write(&buf[..n]).unwrap();
                        }

                    }
                    println!("read {}", message);
                }
                Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    eprintln!("{:?}", e);
                }
            };
        }

    });

    Ok(())
}

async fn connection_handler(map: TcpConnections, listener: TcpListener) {
    loop {
        tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            println!("Adding tcpstream 1");
                            let clients = Arc::clone(&map);
                            match_listener(clients, stream, addr).await;
                        },
                        Err(e) => {
                            eprintln!("{:?}", e);
                        }
                    }
                }
            }
    }
}
