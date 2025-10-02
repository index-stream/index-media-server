use serde::{Deserialize, Serialize};

// Configuration data structures
#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaIndex {
    pub id: String,
    pub name: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub icon: String,
    #[serde(rename = "customIconId")]
    pub custom_icon_id: Option<String>,
    pub folders: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub profiles: Vec<Profile>,
    pub password: String,
    pub indexes: Vec<MediaIndex>,
}

// Temporary struct for handling incoming configuration with base64 custom icon data
#[derive(Debug, Deserialize)]
pub struct IncomingProfile {
    pub id: Option<String>,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct IncomingMediaIndex {
    pub id: Option<String>,
    pub name: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub icon: String,
    #[serde(rename = "customIconFile")]
    pub custom_icon_file: Option<String>, // Base64 encoded image data
    pub folders: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct IncomingConfiguration {
    pub profiles: Vec<IncomingProfile>,
    pub password: String,
    pub indexes: Vec<IncomingMediaIndex>,
}
