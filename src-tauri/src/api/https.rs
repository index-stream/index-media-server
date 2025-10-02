use rcgen::generate_simple_self_signed;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::interval;
use tokio_rustls::TlsAcceptor;
use rustls::{pki_types::{CertificateDer, PrivateKeyDer}, ServerConfig};
use chrono::{DateTime, Utc};

use crate::constants::DEFAULT_HTTPS_PORT;
use crate::utils::network::find_available_port;
use super::router::{Router, handle_connection_with_router};
use super::controllers::{handle_login, handle_token_check, handle_ping, handle_static_files};

/// Certificate storage paths
const CERT_FILE: &str = "https_cert.pem";
const KEY_FILE: &str = "https_key.pem";
const CERT_EXPIRY_FILE: &str = "https_cert_expiry.txt";

/// Certificate validity period (1 year)
const CERT_VALIDITY_DAYS: u64 = 365;

/// Renewal threshold (72 hours before expiry)
const RENEWAL_THRESHOLD_HOURS: i64 = 72;

/// Periodic check interval (24 hours)
const PERIODIC_CHECK_INTERVAL_HOURS: u64 = 24;

/// Get the local IP address for network access
fn get_local_ip_address() -> Result<String, Box<dyn std::error::Error>> {
    use std::net::UdpSocket;
    
    // Connect to a remote address to determine local IP
    // This doesn't actually send data, just determines the local interface
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}

/// Get all available network interfaces
fn get_network_interfaces() -> Vec<String> {
    let mut interfaces = Vec::new();
    
    // Try to get the primary interface
    if let Ok(ip) = get_local_ip_address() {
        interfaces.push(ip);
    }
    
    // Add common local addresses
    interfaces.push("127.0.0.1".to_string());
    interfaces.push("localhost".to_string());
    
    interfaces.sort();
    interfaces.dedup();
    interfaces
}

/// Get the data directory path for certificate storage
fn get_cert_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Use the Tauri app data directory
    // For Tauri apps, we can use the app's data directory
    let mut data_dir = std::env::current_dir()?;
    data_dir.push("data");
    data_dir.push("certs");
    
    // Create the directory if it doesn't exist
    fs::create_dir_all(&data_dir)?;
    
    Ok(data_dir)
}

/// Get the full path for a certificate file
fn get_cert_file_path(filename: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = get_cert_data_dir()?;
    path.push(filename);
    Ok(path)
}

/// Generate a self-signed certificate for HTTPS with proper expiration
pub fn generate_self_signed_cert() -> Result<(Vec<u8>, Vec<u8>, DateTime<Utc>), Box<dyn std::error::Error>> {
    // Get all available network interfaces to include in the certificate
    let interfaces = get_network_interfaces();
    
    // Define the subject alternative names for the certificate
    let mut subject_alt_names = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "0.0.0.0".to_string(),
    ];
    
    // Add all available network interfaces
    for interface in interfaces {
        if !subject_alt_names.contains(&interface) {
            subject_alt_names.push(interface);
        }
    }

    // Generate the certificate
    let cert = generate_simple_self_signed(subject_alt_names)?;

    // Get the certificate and private key in PEM format
    let cert_pem = cert.cert.pem();
    let key_pem = cert.key_pair.serialize_pem();
    
    // Calculate expiration date (1 year from now)
    let now = Utc::now();
    let expiry = now + chrono::Duration::days(CERT_VALIDITY_DAYS as i64);

    Ok((cert_pem.into_bytes(), key_pem.into_bytes(), expiry))
}

/// Save certificate files and expiration date
fn save_certificate_files(cert_pem: Vec<u8>, key_pem: Vec<u8>, expiry: DateTime<Utc>) -> Result<(), Box<dyn std::error::Error>> {
    let cert_path = get_cert_file_path(CERT_FILE)?;
    let key_path = get_cert_file_path(KEY_FILE)?;
    let expiry_path = get_cert_file_path(CERT_EXPIRY_FILE)?;
    
    fs::write(&cert_path, cert_pem)?;
    fs::write(&key_path, key_pem)?;
    fs::write(&expiry_path, expiry.to_rfc3339())?;
    
    println!("📜 Certificate saved to: {}", cert_path.display());
    println!("🔑 Private key saved to: {}", key_path.display());
    println!("⏰ Certificate expires: {}", expiry);
    
    Ok(())
}

/// Load certificate expiration date
fn load_certificate_expiry() -> Result<Option<DateTime<Utc>>, Box<dyn std::error::Error>> {
    let expiry_path = get_cert_file_path(CERT_EXPIRY_FILE)?;
    
    if !expiry_path.exists() {
        return Ok(None);
    }
    
    let expiry_str = fs::read_to_string(&expiry_path)?;
    let expiry = DateTime::parse_from_rfc3339(&expiry_str.trim())?.with_timezone(&Utc);
    Ok(Some(expiry))
}

/// Check if certificate files exist
fn certificate_files_exist() -> Result<bool, Box<dyn std::error::Error>> {
    let cert_path = get_cert_file_path(CERT_FILE)?;
    let key_path = get_cert_file_path(KEY_FILE)?;
    Ok(cert_path.exists() && key_path.exists())
}

/// Check if certificate needs renewal (expires within 72 hours)
fn certificate_needs_renewal() -> Result<bool, Box<dyn std::error::Error>> {
    match load_certificate_expiry()? {
        Some(expiry) => {
            let now = Utc::now();
            let time_until_expiry = expiry - now;
            let needs_renewal = time_until_expiry.num_hours() <= RENEWAL_THRESHOLD_HOURS;
            
            if needs_renewal {
                println!("⚠️  Certificate expires in {} hours, renewal needed", time_until_expiry.num_hours());
            }
            
            Ok(needs_renewal)
        }
        None => Ok(true), // No expiry info means we need to generate
    }
}

/// Load certificates from PEM files
fn load_certs(filename: &str) -> Result<Vec<CertificateDer<'static>>, Box<dyn std::error::Error>> {
    let cert_path = get_cert_file_path(filename)?;
    let certfile = File::open(cert_path)?;
    let mut reader = BufReader::new(certfile);
    let certs = rustls_pemfile::certs(&mut reader)?;
    Ok(certs.into_iter().map(|cert| cert.into()).collect())
}

/// Load private key from PEM file
fn load_private_key(filename: &str) -> Result<PrivateKeyDer<'static>, Box<dyn std::error::Error>> {
    let key_path = get_cert_file_path(filename)?;
    let keyfile = File::open(key_path)?;
    let mut reader = BufReader::new(keyfile);
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut reader)?;
    Ok(PrivateKeyDer::Pkcs8(keys.remove(0).into()))
}

/// Ensure certificate exists and is valid
async fn ensure_valid_certificate() -> Result<(), Box<dyn std::error::Error>> {
    // Check if certificate files exist
    if !certificate_files_exist()? {
        println!("📜 No existing certificate found, generating new one...");
        let (cert_pem, key_pem, expiry) = generate_self_signed_cert()?;
        save_certificate_files(cert_pem, key_pem, expiry)?;
        return Ok(());
    }
    
    // Check if certificate needs renewal
    if certificate_needs_renewal()? {
        println!("🔄 Certificate needs renewal, generating new one...");
        let (cert_pem, key_pem, expiry) = generate_self_signed_cert()?;
        save_certificate_files(cert_pem, key_pem, expiry)?;
    } else {
        println!("✅ Existing certificate is valid");
    }
    
    Ok(())
}

/// Periodic certificate renewal check
async fn periodic_certificate_check() {
    let mut interval = interval(Duration::from_secs(PERIODIC_CHECK_INTERVAL_HOURS * 3600));
    
    loop {
        interval.tick().await;
        
        println!("🔍 Performing periodic certificate check...");
        
        if let Err(e) = ensure_valid_certificate().await {
            eprintln!("❌ Error during periodic certificate check: {}", e);
        } else {
            println!("✅ Periodic certificate check completed successfully");
        }
    }
}

/// Start the HTTPS server for network access
pub async fn start_https_server(app_state: crate::api::state::ExtendedAppState) -> Result<u16, Box<dyn std::error::Error>> {
    // Ensure we have a valid certificate
    ensure_valid_certificate().await?;
    
    // Start periodic certificate check
    tokio::spawn(periodic_certificate_check());
    
    // Load certificates and private key
    let certs = load_certs(CERT_FILE)?;
    let key = load_private_key(KEY_FILE)?;
    
    // Create TLS configuration
    let tls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    
    let tls_config = Arc::new(tls_config);
    let tls_acceptor = TlsAcceptor::from(tls_config);
    
    // Find an available port starting from the default HTTPS port
    let port = find_available_port(DEFAULT_HTTPS_PORT)?;
    
    // Store the port in the shared state
    {
        let mut state = app_state.lock().await;
        state.https_port = Some(port);
    }
    
    // Bind to all interfaces on the determined port
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    // Get the local IP address for display
    let _local_ip = get_local_ip_address().unwrap_or_else(|_| "unknown".to_string());
    let interfaces = get_network_interfaces();
    
    println!("🔒 HTTPS API server running on https://0.0.0.0:{}", port);
    println!("   Network accessible endpoints:");
    for interface in &interfaces {
        println!("     https://{}:{}/api/ping", interface, port);
    }
    println!("   Web interface available at:");
    for interface in &interfaces {
        println!("     https://{}:{}/", interface, port);
    }
    println!("   Certificate stored in: {}", get_cert_data_dir()?.display());
    println!("   Periodic renewal check every {} hours", PERIODIC_CHECK_INTERVAL_HOURS);
    
    // Create router and add routes
    let mut router = Router::new();
    router.add_route("POST", "/api/login", handle_login);
    router.add_route("GET", "/api/token*", handle_token_check);
    router.add_route("GET", "/api/ping", handle_ping);
    router.add_route("GET", "*", handle_static_files);
    
    // Accept connections and handle them
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let tls_acceptor = tls_acceptor.clone();
                let router = router.clone();
                
                tokio::spawn(async move {
                    match tls_acceptor.accept(stream).await {
                        Ok(tls_stream) => {
                            if let Err(e) = handle_connection_with_router(tls_stream, &router).await {
                                eprintln!("Error handling connection from {}: {}", addr, e);
                            }
                        }
                        Err(_e) => {
                            // Leaving commented out as every this always gets logged with self signed certs
                            // eprintln!("TLS handshake failed for {}: {}", addr, e);
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}