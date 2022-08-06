use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:9090").await.unwrap();
    // We don't recieve values
    let (tx, _) = broadcast::channel(32);

    println!("server started!");

    loop {
        // Connect to client
        let (mut socket, addr) = listener.accept().await.expect("accept listener");
        println!("connected to {}", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        tx.send((addr, line.clone())).unwrap();
                        line.clear();
                    }

                    result = rx.recv() => {
                        let (other_addr, msg) = result.unwrap();
                        if addr != other_addr {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
