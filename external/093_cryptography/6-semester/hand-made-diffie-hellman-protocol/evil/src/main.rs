use std::io::Read;
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9090").unwrap();
    let mut buffer = [0u8;1024];

    while let Ok((mut stream, _)) = listener.accept() {
        let size= stream.read(&mut buffer).unwrap();
        println!("{:?}", &buffer[..size]);
    }
}
