use std::{borrow::Borrow, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};

pub struct Proxy<T: Fn(&mut TcpStream)> {
    socket: SocketAddr,
    handler: Option<Box<T>>,
}

impl<T: Fn(&mut TcpStream)> Proxy<T> {
    pub fn new(socket: SocketAddr) -> Proxy<T> {
        Proxy {
            socket,
            handler: None,
        }
    }

    pub fn use_handler(&mut self, handler: Box<T>) {
        self.handler = Some(handler);
    }

    pub async fn listen(&self) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(&self.socket).await;

        if let Err(e) = listener {
            panic!("Failed to bind socket: {}", e);
        }

        let listener = listener.unwrap();

        loop {
            match listener.accept().await {
                Ok((mut socket, _)) => {
                    if let Some(handler) = self.handler.borrow() {
                        handler(&mut socket);
                    }
                }
                Err(e) => {
                    eprintln!("Couldn't get client: {}", e);
                }
            }
        }
    }
}
