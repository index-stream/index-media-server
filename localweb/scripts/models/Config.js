class Config {
    constructor() {
        this._config = null;
    }

    // Set the configuration data
    setConfig(configData) {
        this._config = configData;
        console.log('Config updated:', this._config);
    }

    // Get the entire configuration
    getConfig() {
        return this._config;
    }

    // Get server name
    getServerName() {
        return this._config?.name || 'Index Media Server';
    }

    // Update server name (local update)
    updateServerName(newName) {
        if (this._config) {
            this._config.name = newName;
            console.log('Server name updated locally:', newName);
        }
    }

    // Update server password (local update)
    updatePassword(newPassword) {
        if (this._config) {
            this._config.password = newPassword;
            console.log('Password updated locally');
        }
    }

    // Get config ID
    getId() {
        return this._config?.id;
    }

    // Check if config is loaded
    isLoaded() {
        return this._config !== null;
    }

    // Clear config (useful for logout/reset)
    clear() {
        this._config = null;
        console.log('Config cleared');
    }
}

// Create a singleton instance
let config = new Config();
export default config;
