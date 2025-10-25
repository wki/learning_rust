use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    println!("Listening on port 8000");
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    println!("Connection from {}", socket.peer_addr().unwrap());
    let mut stream = BufStream::new(socket);
    let line = &mut String::new();

    loop {
        let nr_bytes = stream.read_line(line).await.unwrap();
        if nr_bytes == 0 {
            println!("0 bytes read, closing connection");
            stream.write(b"bye\r\n").await.unwrap();
            stream.flush().await.unwrap();
            break;
        } else {
            println!("read {} bytes -> {}", nr_bytes, line);
            stream.write(format!("consumed {} bytes\r\n", nr_bytes).as_bytes()).await.unwrap();
            stream.flush().await.unwrap();
        }
    }
}
