use serde::{Deserialize, Serialize};

// Configuration data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub folders: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub id: String,
    pub name: String,
    pub profiles: Vec<Profile>,
    pub password: String,
    pub indexes: Vec<MediaIndex>,
}

// Configuration response structure that excludes the password field
#[derive(Debug, Serialize)]
pub struct ConfigurationResponse {
    pub id: String,
    pub name: String,
    pub profiles: Vec<Profile>,
    pub indexes: Vec<MediaIndex>,
}

impl From<Configuration> for ConfigurationResponse {
    fn from(config: Configuration) -> Self {
        Self {
            id: config.id,
            name: config.name,
            profiles: config.profiles,
            indexes: config.indexes,
        }
    }
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
    pub name: String,
    pub profiles: Vec<IncomingProfile>,
    pub password: String,
    pub indexes: Vec<IncomingMediaIndex>,
}

// Request structures for individual server updates
#[derive(Debug, Deserialize)]
pub struct ServerPasswordUpdate {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerNameUpdate {
    pub name: String,
}
