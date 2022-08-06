use crate::message::{ToConnThread, ToMainThread};
use tokio::net::TcpStream;
use tokio::runtime;
use tokio::sync::mpsc::{Receiver, Sender};

pub fn conn_thread(tx: Sender<ToMainThread>, mut rx: Receiver<ToConnThread>) {
    let runtime = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async move {
        let mut connection = None;

        'main: loop {
            tokio::select! {
                msg = rx.recv() => {
                    use ToConnThread::*;

                    match msg.unwrap() {
                        Connect(addr) => {
                            match TcpStream::connect(addr).await {
                                Ok(c) => {
                                    connection = Some(c);
                                    tx.send(ToMainThread::Connected).await.unwrap();
                                }
                                Err(e) => {
                                    tx.send(ToMainThread::ConnectionError(e.to_string())).await.unwrap();
                                }
                            }
                        }

                        SendMessage(msg) => {

                        }

                        Shutdown => {
                            break 'main;
                        }
                    }
                }
            }
        }
    });
}
