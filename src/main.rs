use dns::DnsPacket;
use tokio::{net::{TcpListener, TcpStream, UdpSocket}, io::{AsyncReadExt, AsyncWriteExt}};
use tokio_native_tls::native_tls::{Identity, self};
mod dns;

const ADDRESS: &'static str = "127.0.0.1";
const PORT: u16 = 5300;
const PORT_TLS: u16 = 8530;
const DNS_SERVER: &'static str = "1.1.1.1";
const DNS_SERVER_PORT: u16 = 853;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let udp_socket = UdpSocket::bind(&(ADDRESS, PORT)).await?;
    let tcp_listener = TcpListener::bind(&(ADDRESS, PORT)).await?;
    let tls_listener = TcpListener::bind(&(ADDRESS, PORT_TLS)).await?;

    // TLS
    let pem = include_bytes!("./localhost.crt");
    let key = include_bytes!("./localhost.key");
    let id = Identity::from_pkcs8(pem, key).expect("Failed to create Identity.");
    let tls_acceptor = tokio_native_tls::TlsAcceptor::from(native_tls::TlsAcceptor::builder(id).build()?);

    // Threads
    let udp_handle = tokio::spawn(handle_udp(udp_socket));
    let tcp_handle = tokio::spawn(handle_tcp(tcp_listener));
    let tls_handle = tokio::spawn(handle_tls(tls_listener, tls_acceptor));

    udp_handle.await?;
    tcp_handle.await?;
    tls_handle.await?;

    Ok(())
}

async fn handle_udp(socket: UdpSocket) {
    let mut buf = [0; 1024];
    loop {
        let (_, addr) = socket.recv_from(&mut buf).await.expect("Failed to receive bytes.");
        let response_bytes = handle_dns_request(&buf, 0).await.expect("Failed to resolve dns request.");
        socket.send_to(&response_bytes[2..response_bytes.len()], addr).await.expect("Failed to send bytes.");
    }
}

async fn handle_tcp(listener: TcpListener) {
    let mut buf = [0; 1024];
    loop {
        let (mut socket, _) = listener.accept().await.expect("Failed to accept.");
        tokio::spawn(async move {
            let len = match socket.read(&mut buf).await {
                Ok(n) if n == 0 => return,
                Ok(n) => n,
                Err(e) => return eprintln!("{e}"),
            };
            let response_bytes = handle_dns_request(&buf, len).await.expect("Failed to resolve dns request.");
            socket.write_all(&response_bytes).await.expect("Failed to send packet.");
        });
    }
}

async fn handle_tls(listener: TcpListener, acceptor: tokio_native_tls::TlsAcceptor) {
    let mut buf = [0; 1024];
    loop {
        let (socket, _) = listener.accept().await.expect("Failed to accept.");
        let acceptor = acceptor.clone();
        tokio::spawn(async move {
            let mut stream = acceptor.accept(socket).await.expect("Failed to accept TLS connection");
            let len = match stream.read(&mut buf).await {
                Ok(n) if n == 0 => return,
                Ok(n) => n,
                Err(e) => return eprintln!("{e}"),
            };
            let response_bytes = handle_dns_request(&buf, len).await.expect("Failed to resolve dns request.");
            stream.write_all(&response_bytes).await.expect("Failed to send packet.");
        });
    }
}

async fn handle_dns_request(bytes: &[u8], len: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let packet = if len == 0 {DnsPacket::from(bytes)} else {DnsPacket::from_tcp(bytes, len)};
    println!("Incoming Packet:\n{:#?}", packet);
    let socket = TcpStream::connect((DNS_SERVER, DNS_SERVER_PORT));
    let cx = tokio_native_tls::TlsConnector::from(native_tls::TlsConnector::builder().build()?);
    let mut socket = cx.connect(DNS_SERVER, socket.await?).await?;

    let mut bytes = bytes.to_vec();
    if len == 0 {
        bytes = [packet.size(), bytes.to_vec()].concat()
    }

    socket.write_all(&bytes).await?;
    let mut buf = [0; 1024];
    let len = match socket.read(&mut buf).await {
        Ok(n) if n == 0 => return Ok(Vec::new()),
        Ok(n) => n,
        Err(e) => return Err(Box::new(e)),
    };
    std::fs::write("expected.dns", &buf[0..len]);

    let packet = DnsPacket::from_tcp(&buf, len);
    println!("Outgoing Packet:\n{:#?}\n", packet);
    std::fs::write("./out.dns", packet.clone().bytes());
    Ok(packet.bytes())
}