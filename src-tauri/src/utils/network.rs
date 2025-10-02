/// Find an available port starting from the given port
/// 
/// This function tries to bind to ports starting from `start_port` and returns
/// the first available port. It searches up to 1000 ports ahead.
/// 
/// # Arguments
/// * `start_port` - The port number to start searching from
/// 
/// # Returns
/// * `Ok(u16)` - The first available port found
/// * `Err(std::io::Error)` - If no available ports are found
pub fn find_available_port(start_port: u16) -> Result<u16, std::io::Error> {
    let end_port = start_port + 1000;
    for port in start_port..end_port {
        match std::net::TcpListener::bind(("0.0.0.0", port)) {
            Ok(_) => return Ok(port),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::AddrInUse {
                    continue; // Port is in use, try next one
                } else {
                    return Err(e); // Some other error
                }
            }
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, "No available ports found"))
}
