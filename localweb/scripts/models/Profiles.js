class Profiles {
    constructor() {
        this._profiles = null;
    }

    // Set the profiles data
    setProfiles(profiles) {
        this._profiles = profiles;
        console.log('Profiles updated:', this._profiles);
    }

    // Get profiles
    getProfiles() {
        return this._profiles || [];
    }

    // Add a new profile (local update)
    addProfile(profile) {
        if (this._profiles) {
            if (!this._profiles) {
                this._profiles = [];
            }
            this._profiles.push(profile);
            console.log('Profile added locally:', profile);
        }
    }

    // Update an existing profile (local update)
    updateProfile(updatedProfile) {
        if (this._profiles) {
            const index = this._profiles.findIndex(p => p.id === updatedProfile.id);
            if (index !== -1) {
                this._profiles[index] = updatedProfile;
                console.log('Profile updated locally:', updatedProfile);
            }
        }
    }

    // Remove a profile (local update)
    removeProfile(profileId) {
        if (this._profiles) {
            const index = this._profiles.findIndex(p => p.id === profileId);
            if (index !== -1) {
                const removedProfile = this._profiles.splice(index, 1)[0];
                console.log('Profile removed locally:', removedProfile);
            }
        }
    }

    // Check if profiles are loaded
    isLoaded() {
        return this._profiles !== null;
    }

    // Clear profiles (useful for logout/reset)
    clear() {
        this._profiles = null;
        console.log('Profiles cleared');
    }
}

// Create a singleton instance
let profiles = new Profiles();
export default profiles;
