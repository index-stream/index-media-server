use warp::path::FullPath;
use crate::constants::DEFAULT_HTTPS_PORT;

// Handler for serving static files with SPA fallback
pub async fn handle_static_file(path: FullPath) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let path_str = path.as_str();
    
    // Get the current working directory and construct absolute paths
    // Tauri runs from src-tauri directory, so we need to go up one level to find localweb/
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let web_dir = current_dir.parent().unwrap_or(&current_dir).join("localweb");
    
    let file_path = if path_str == "/" {
        web_dir.join("index.html")
    } else {
        web_dir.join(path_str.trim_start_matches('/'))
    };
    
    // Try to serve the requested file
    match tokio::fs::metadata(&file_path).await {
        Ok(metadata) if metadata.is_file() => {
            let content_type = get_content_type(file_path.to_str().unwrap_or(""));
            match tokio::fs::read(&file_path).await {
                Ok(content) => {
                    let mut response = warp::reply::Response::new(content.into());
                    response.headers_mut().insert(
                        "content-type",
                        warp::http::HeaderValue::from_static(content_type),
                    );
                    Ok(Box::new(response))
                }
                Err(_) => Ok(Box::new(warp::reply::with_status(
                    "Internal Server Error",
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                )))
            }
        }
        _ => {
            // SPA fallback - serve index.html for any non-file requests
            let index_path = web_dir.join("index.html");
            match tokio::fs::read(&index_path).await {
                Ok(content) => {
                    let mut response = warp::reply::Response::new(content.into());
                    response.headers_mut().insert(
                        "content-type",
                        warp::http::HeaderValue::from_static("text/html"),
                    );
                    Ok(Box::new(response))
                }
                Err(_) => Ok(Box::new(warp::reply::with_status(
                    "Not Found",
                    warp::http::StatusCode::NOT_FOUND,
                )))
            }
        }
    }
}

// Get content type based on file extension
fn get_content_type(path: &str) -> &'static str {
    match path.split('.').last().unwrap_or("") {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}




// Handler for ping endpoint - returns health status
pub async fn handle_ping() -> Result<impl warp::Reply, warp::Rejection> {
    let response_body = serde_json::json!({
        "status": "healthy",
        "service": "Index Media Server",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(warp::reply::with_status(
        warp::reply::json(&response_body),
        warp::http::StatusCode::OK,
    ))
}

/// Get the local IP address for network access (copied from https.rs)
fn get_local_ip_address() -> Result<String, Box<dyn std::error::Error>> {
    use std::net::UdpSocket;
    
    // Connect to a remote address to determine local IP
    // This doesn't actually send data, just determines the local interface
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}

/// Convert a number to base 24 using characters A-Z (excluding I and O)
/// Character set: A B C D E F G H J K L M N P Q R S T U V W X Y Z
/// This gives us 24 characters (base 24)
fn number_to_base24(mut num: u32) -> String {
    const CHARS: &[char] = &['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
    
    if num == 0 {
        return "A".to_string();
    }
    
    let mut result = String::new();
    while num > 0 {
        result.push(CHARS[(num % 24) as usize]);
        num /= 24;
    }
    
    result.chars().rev().collect()
}

/// Convert base 24 string back to number
#[allow(dead_code)]
fn base24_to_number(s: &str) -> u32 {
    const CHARS: &[char] = &['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
    
    let mut result = 0;
    for ch in s.chars() {
        if let Some(pos) = CHARS.iter().position(|&c| c == ch) {
            result = result * 24 + pos as u32;
        }
    }
    result
}

/// Compress IP address into a code string
/// 
/// Connect Code Format:
/// The first character dictates the compression format:
/// A - 192.168.0.x, followed by 2 *s (where * represents base24 characters)
/// B - 192.168.1.x, followed by 2 *s  
/// C - 10.x.x.x, followed by 6 *s
/// D - 172.x.x.x, followed by 6 *s
/// E - external connection via server (not implemented yet)
/// F - no compression, followed by 8 *s
fn compress_ip_to_code(ip: &str) -> String {
    let mut code = String::new();
    
    // Parse IP address
    let ip_parts: Vec<&str> = ip.split('.').collect();
    if ip_parts.len() != 4 {
        // Invalid IP, use format F (no compression)
        code.push('F');
        // Encode the full IP as 8 base24 characters
        let ip_bytes: Vec<u8> = ip_parts.iter()
            .filter_map(|part| part.parse::<u8>().ok())
            .collect();
        if ip_bytes.len() == 4 {
            // Encode each octet as 2-character base24 groups
            for octet in &ip_bytes {
                let mut encoded = number_to_base24(*octet as u32);
                // Pad to 2 characters
                while encoded.len() < 2 {
                    encoded.insert(0, 'A');
                }
                code.push_str(&encoded);
            }
        } else {
            code.push_str("AAAAAAAA"); // fallback
        }
    } else {
        let octets: Vec<u8> = ip_parts.iter()
            .filter_map(|part| part.parse::<u8>().ok())
            .collect();
        
        if octets.len() == 4 {
            match (octets[0], octets[1], octets[2]) {
                (192, 168, 0) => {
                    code.push('A');
                    let encoded = number_to_base24(octets[3] as u32);
                    code.push_str(&encoded);
                    // Pad to 2 characters total for IP part
                    while code.len() < 3 {
                        code.push('A');
                    }
                }
                (192, 168, 1) => {
                    code.push('B');
                    let encoded = number_to_base24(octets[3] as u32);
                    code.push_str(&encoded);
                    // Pad to 2 characters total for IP part
                    while code.len() < 3 {
                        code.push('A');
                    }
                }
                (10, _, _) => {
                    code.push('C');
                    // Encode each octet as 2-character base24 groups
                    for i in 1..4 {
                        let mut encoded = number_to_base24(octets[i] as u32);
                        // Pad to 2 characters
                        while encoded.len() < 2 {
                            encoded.insert(0, 'A');
                        }
                        code.push_str(&encoded);
                    }
                }
                (172, _, _) => {
                    code.push('D');
                    // Encode each octet as 2-character base24 groups
                    for i in 1..4 {
                        let mut encoded = number_to_base24(octets[i] as u32);
                        // Pad to 2 characters
                        while encoded.len() < 2 {
                            encoded.insert(0, 'A');
                        }
                        code.push_str(&encoded);
                    }
                }
                _ => {
                    // External IP, use format F (no compression)
                    code.push('F');
                    // Encode each octet as 2-character base24 groups
                    for octet in &octets {
                        let mut encoded = number_to_base24(*octet as u32);
                        // Pad to 2 characters
                        while encoded.len() < 2 {
                            encoded.insert(0, 'A');
                        }
                        code.push_str(&encoded);
                    }
                }
            }
        } else {
            // Invalid IP format, use F
            code.push('F');
            code.push_str("AAAAAAAA");
        }
    }
    
    code
}

/// Compress port into a code string
/// 
/// Port compression:
/// If port == DEFAULT_HTTPS_PORT: no characters needed (0 offset)
/// If port < DEFAULT_HTTPS_PORT + 24: 1 character
/// If port < DEFAULT_HTTPS_PORT + 24^2: 2 characters  
/// If port < DEFAULT_HTTPS_PORT + 24^3: 3 characters
/// Otherwise: 4 characters (absolute port)
fn compress_port_to_code(port: u16) -> String {
    let port_offset = if port >= DEFAULT_HTTPS_PORT {
        port - DEFAULT_HTTPS_PORT
    } else {
        // For ports below default, encode as absolute port (4 characters)
        let mut port_encoded = number_to_base24(port as u32);
        // Pad to 4 characters
        while port_encoded.len() < 4 {
            port_encoded.insert(0, 'A');
        }
        return port_encoded;
    };
    
    // Determine port encoding based on offset from default port
    if port_offset == 0 {
        // Port equals default port - no characters needed
        String::new()
    } else if port_offset < 24 {
        // 1 character needed
        let mut port_encoded = number_to_base24(port_offset as u32);
        // Pad to 1 character
        while port_encoded.len() < 1 {
            port_encoded.insert(0, 'A');
        }
        port_encoded
    } else if port_offset < 24 * 24 {
        // 2 characters needed
        let mut port_encoded = number_to_base24(port_offset as u32);
        // Pad to 2 characters
        while port_encoded.len() < 2 {
            port_encoded.insert(0, 'A');
        }
        port_encoded
    } else if port_offset < 24 * 24 * 24 {
        // 3 characters needed
        let mut port_encoded = number_to_base24(port_offset as u32);
        // Pad to 3 characters
        while port_encoded.len() < 3 {
            port_encoded.insert(0, 'A');
        }
        port_encoded
    } else {
        // 4 characters needed (absolute port)
        let mut port_encoded = number_to_base24(port as u32);
        // Pad to 4 characters
        while port_encoded.len() < 4 {
            port_encoded.insert(0, 'A');
        }
        port_encoded
    }
}

/// Compress IP address and port into a connect code
fn compress_to_connect_code(ip: &str, port: u16) -> String {
    let ip_code = compress_ip_to_code(ip);
    let port_code = compress_port_to_code(port);
    
    format!("{}{}", ip_code, port_code)
}



// Handler for connect code endpoint - returns compressed IP and port for HTTPS server
pub async fn handle_connect_code(app_state: crate::api::state::ExtendedAppState) -> Result<impl warp::Reply, warp::Rejection> {
    // Get the local IP address
    let ip = match get_local_ip_address() {
        Ok(ip) => ip,
        Err(_) => "127.0.0.1".to_string(), // fallback to localhost
    };
    
    // Get the HTTPS server port from shared state
    let https_port = {
        let state = app_state.lock().await;
        state.https_port.unwrap_or(DEFAULT_HTTPS_PORT)
    };
    
    // Generate the connect code for the HTTPS server
    let connect_code = compress_to_connect_code(&ip, https_port);
    
    let response_body = serde_json::json!({
        "success": true,
        "connectCode": connect_code,
        "ip": ip,
        "port": https_port,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(warp::reply::with_status(
        warp::reply::json(&response_body),
        warp::http::StatusCode::OK,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_to_base24() {
        assert_eq!(number_to_base24(0), "A");
        assert_eq!(number_to_base24(1), "B");
        assert_eq!(number_to_base24(23), "Z");
        assert_eq!(number_to_base24(24), "BA");
        assert_eq!(number_to_base24(25), "BB");
        assert_eq!(number_to_base24(48), "CA");
        assert_eq!(number_to_base24(575), "ZZ");
        assert_eq!(number_to_base24(576), "BAA");
    }

    #[test]
    fn test_base24_to_number() {
        assert_eq!(base24_to_number("A"), 0);
        assert_eq!(base24_to_number("B"), 1);
        assert_eq!(base24_to_number("Z"), 23);
        assert_eq!(base24_to_number("BA"), 24);
        assert_eq!(base24_to_number("BB"), 25);
        assert_eq!(base24_to_number("CA"), 48);
        assert_eq!(base24_to_number("ZZ"), 575);
        assert_eq!(base24_to_number("BAA"), 576);
    }

    #[test]
    fn test_format_a_192_168_0_x() {
        // Format A - 192.168.0.x test cases
        assert_eq!(compress_to_connect_code("192.168.0.26", DEFAULT_HTTPS_PORT), "ABC");
        assert_eq!(compress_to_connect_code("192.168.0.26", DEFAULT_HTTPS_PORT + 1), "ABCB");
        assert_eq!(compress_to_connect_code("192.168.0.255", DEFAULT_HTTPS_PORT), "ALR");
        assert_eq!(compress_to_connect_code("192.168.0.26", DEFAULT_HTTPS_PORT + 23), "ABCZ");
        assert_eq!(compress_to_connect_code("192.168.0.0", DEFAULT_HTTPS_PORT), "AAA");
    }

    #[test]
    fn test_format_b_192_168_1_x() {
        // Format B - 192.168.1.x test cases
        assert_eq!(compress_to_connect_code("192.168.1.0", DEFAULT_HTTPS_PORT), "BAA");
        assert_eq!(compress_to_connect_code("192.168.1.26", DEFAULT_HTTPS_PORT), "BBC");
        assert_eq!(compress_to_connect_code("192.168.1.26", DEFAULT_HTTPS_PORT + 2), "BBCC");
        assert_eq!(compress_to_connect_code("192.168.1.255", DEFAULT_HTTPS_PORT), "BLR");
    }

    #[test]
    fn test_format_c_10_x_x_x() {
        // Format C - 10.x.x.x test cases
        assert_eq!(compress_to_connect_code("10.0.0.0", DEFAULT_HTTPS_PORT), "CAAAAAA");
        assert_eq!(compress_to_connect_code("10.1.1.1", DEFAULT_HTTPS_PORT), "CABABAB");
        assert_eq!(compress_to_connect_code("10.0.0.0", DEFAULT_HTTPS_PORT + 2), "CAAAAAAC");
        assert_eq!(compress_to_connect_code("10.255.255.255", DEFAULT_HTTPS_PORT), "CLRLRLR");
    }

    #[test]
    fn test_format_d_172_x_x_x() {
        // Format D - 172.x.x.x test cases
        assert_eq!(compress_to_connect_code("172.0.0.0", DEFAULT_HTTPS_PORT), "DAAAAAA");
        assert_eq!(compress_to_connect_code("172.2.2.2", DEFAULT_HTTPS_PORT + 50), "DACACACCC");
        assert_eq!(compress_to_connect_code("172.0.0.0", DEFAULT_HTTPS_PORT + 3), "DAAAAAAD");
        assert_eq!(compress_to_connect_code("172.255.255.255", DEFAULT_HTTPS_PORT), "DLRLRLR");
    }

    #[test]
    fn test_format_f_no_compression() {
        // Format F - No compression test cases
        assert_eq!(compress_to_connect_code("0.0.0.0", DEFAULT_HTTPS_PORT), "FAAAAAAAA");
        assert_eq!(compress_to_connect_code("25.25.25.25", DEFAULT_HTTPS_PORT), "FBBBBBBBB");
        assert_eq!(compress_to_connect_code("0.0.0.0", DEFAULT_HTTPS_PORT + 5), "FAAAAAAAAF");
        assert_eq!(compress_to_connect_code("0.0.0.0", 0), "FAAAAAAAAAAAA");
        assert_eq!(compress_to_connect_code("255.255.255.255", DEFAULT_HTTPS_PORT), "FLRLRLRLR");
    }

    #[test]
    fn test_special_port_cases() {
        // Special port cases
        assert_eq!(compress_to_connect_code("192.168.0.0", 80), "AAAAADJ");
        assert_eq!(compress_to_connect_code("192.168.0.0", 443), "AAAAAUM");
        assert_eq!(compress_to_connect_code("192.168.0.0", 3000), "AAAAFFA");
        assert_eq!(compress_to_connect_code("192.168.0.26", DEFAULT_HTTPS_PORT), "ABC");
        assert_eq!(compress_to_connect_code("192.168.0.26", DEFAULT_HTTPS_PORT + 1), "ABCB");
        assert_eq!(compress_to_connect_code("192.168.0.26", DEFAULT_HTTPS_PORT + 24), "ABCBA");
        assert_eq!(compress_to_connect_code("192.168.0.26", DEFAULT_HTTPS_PORT + 26), "ABCBC");
        assert_eq!(compress_to_connect_code("192.168.0.26", 627), "ABCABCD");
        assert_eq!(compress_to_connect_code("192.168.0.0", DEFAULT_HTTPS_PORT + 80), "AAADJ");
    }
}
