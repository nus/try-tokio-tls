use std::net::{
    SocketAddr
};
use tokio::net::{TcpStream, TcpListener};
use tokio_tls;
use tokio_tls::{TlsAcceptor, TlsStream};
use tokio::io::{AsyncReadExt};
use native_tls;
use native_tls::Identity;

// Reference.
// https://github.com/tokio-rs/tokio/blob/master/tokio-tls/examples/tls-echo.rs
// https://github.com/tokio-rs/tokio

async fn handle_tls(mut tls_stream: TlsStream<TcpStream>) {
    let mut buf = [0; 1024];
    let n = match tls_stream.read(&mut buf).await {
        Ok(n) => n,
        Err(err) => {
            eprintln!("tls_stream.read() failed. {}", err.to_string());
            return;
        }
    };

    if n == 0 {
        return;
    }
    println!("read: {}", unsafe {
        String::from_utf8_unchecked(buf[0..n].into())
    });
}

async fn handle_tcp(stream: TcpStream, addr: SocketAddr, tls_acceptor: TlsAcceptor) {
    println!("Connected from {}", addr.to_string());

    match tls_acceptor.accept(stream).await {
        Ok(tls_stream) => {
            println!("Accepted on TLS.");
            handle_tls(tls_stream).await;
        }
        Err(err) => {
            eprintln!("tls_acceptor.accept() failed. {}", err.to_string());
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:9000";
    let try_socket = TcpListener::bind(&addr).await;
    let mut tcp_listener = try_socket.expect("TcpListener::bind() failed.");

    let der = include_bytes!("../cert/server.pfx");
    let cert = Identity::from_pkcs12(der, "aaa")?;
    let tls_acceptor = tokio_tls::TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build()?);

    loop {
        match tcp_listener.accept().await {
            Ok((stream, addr)) => {
                println!("Accepted from {} on TCP.", addr.to_string());
                let tls_acceptor = tls_acceptor.clone();
                tokio::spawn(handle_tcp(stream, addr, tls_acceptor));
            },
            Err(err) => {
                eprintln!("tcp_listener.accept() failed. {}", err.to_string());
            },
        }
    }
}
