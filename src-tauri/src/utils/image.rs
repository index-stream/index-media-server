use image::ImageFormat;

// Helper function to detect image format and return appropriate extension
pub fn detect_image_extension(image_data: &[u8]) -> Result<&'static str, String> {
    match image::guess_format(image_data) {
        Ok(format) => {
            match format {
                ImageFormat::Png => Ok("png"),
                ImageFormat::Jpeg => Ok("jpg"),
                ImageFormat::Gif => Ok("gif"),
                ImageFormat::Bmp => Ok("bmp"),
                ImageFormat::Ico => Ok("ico"),
                _ => Ok("png"), // Default fallback to PNG for unsupported formats
            }
        }
        Err(_) => {
            // If format detection fails, try to detect by magic bytes
            if image_data.len() >= 4 {
                match &image_data[0..4] {
                    [0x89, 0x50, 0x4E, 0x47] => Ok("png"),   // PNG signature
                    [0xFF, 0xD8, 0xFF, _] => Ok("jpg"),      // JPEG signature
                    [0x47, 0x49, 0x46, 0x38] => Ok("gif"),   // GIF signature
                    [0x42, 0x4D, _, _] => Ok("bmp"),         // BMP signature
                    _ => Ok("png"), // Default fallback
                }
            } else {
                Ok("png") // Default fallback for very small files
            }
        }
    }
}
