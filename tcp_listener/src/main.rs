use std::io::{BufRead, BufReader, LineWriter, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

fn handle_client(stream: TcpStream) {
    println!("Starting handling a client");
    let line = &mut String::new();
    let mut reader = BufReader::new(&stream);
    let mut writer = LineWriter::new(&stream);
    loop {
        // stream.read_to_string(line).unwrap();
        let nr_bytes = reader.read_line(line).unwrap();
        if nr_bytes == 0 {
            println!("0 bytes read");
            writer.write_fmt(format_args!("bye!\r\n")).unwrap();
            break;
        } else {
            println!("read {} bytes -> {}", nr_bytes, line);
            reader.consume(nr_bytes);
            // stream.write_fmt(format_args!("consumed {} bytes\r\n", nr_bytes)).unwrap();
            writer.write_fmt(format_args!("consumed {} bytes\r\n", nr_bytes)).unwrap();
        }
    }
    println!("end of stream");
}

fn main() {
    let port:u16 = 8000;
    println!("Listening for connections on port {}", port);
    let address = SocketAddr::from(([127,0,0,1], port));
    let listener = TcpListener::bind(address)
        .expect("Unable to bind TCP socket");

    for stream in listener.incoming() {
        handle_client(stream.unwrap());
    }
}
