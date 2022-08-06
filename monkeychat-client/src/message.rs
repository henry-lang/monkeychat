use std::net::SocketAddr;

#[derive(Debug)]
pub enum ToConnThread {
    Connect(SocketAddr),
    SendMessage(String),
    Shutdown,
}

#[derive(Debug)]
pub enum ToMainThread {
    ConnectionError(String),
    Connected,
    RecieveMessage(String),
}
