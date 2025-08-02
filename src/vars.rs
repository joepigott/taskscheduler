use std::net::SocketAddr;

/// Validates the server address by attempting to bind to it.
pub fn is_available(address: SocketAddr) -> bool {
    std::net::TcpListener::bind(address).is_ok()
}

