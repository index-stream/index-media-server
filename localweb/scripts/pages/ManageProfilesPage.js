import Page from './Page.js';
import { PAGES } from '../constants.js';
import Backend from '../clients/Backend.js';
import Profiles from '../models/Profiles.js';
import { escapeHtml, getInitials } from '../utils/text.js';

export default class ManageProfilesPage extends Page {
    constructor() {
        super(PAGES.MANAGE_PROFILES);
        this.editingProfile = null;
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Back button
        const backBtn = document.getElementById('profiles-back-btn');
        backBtn?.addEventListener('click', () => this.onBackClick());

        // Add profile button
        const addProfileBtn = document.getElementById('manage-add-profile-btn');
        addProfileBtn?.addEventListener('click', () => this.showAddProfileModal());

        // Modal buttons
        const cancelBtn = document.getElementById('profile-cancel-btn');
        cancelBtn?.addEventListener('click', () => this.hideProfileModal());

        const saveBtn = document.getElementById('profile-save-btn');
        saveBtn?.addEventListener('click', (e) => {
            e.preventDefault();
            this.onProfileSaveClick();
        });

        // Color picker synchronization
        const colorPicker = document.getElementById('profile-color');
        const colorText = document.getElementById('profile-color-text');
        
        colorPicker?.addEventListener('input', () => {
            colorText.value = colorPicker.value;
        });
        
        colorText?.addEventListener('input', () => {
            if (this.isValidColor(colorText.value)) {
                colorPicker.value = colorText.value;
            }
        });

        // Modal close on backdrop click
        const modal = document.getElementById('profile-modal');
        modal?.addEventListener('click', (e) => {
            if (e.target === modal) {
                this.hideProfileModal();
            }
        });

        // Escape key handling
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && !modal?.classList.contains('hidden')) {
                this.hideProfileModal();
            }
        });
    }

    onBackClick() {
        // Navigate back to home page
        document.dispatchEvent(new CustomEvent('navigate', {
            detail: { page: PAGES.HOME }
        }));
    }

    showAddProfileModal() {
        this.editingProfile = null;
        this.showProfileModal('Add Profile');
    }

    showEditProfileModal(profile) {
        this.editingProfile = profile;
        this.showProfileModal('Edit Profile');
        
        // Populate form with existing data
        document.getElementById('profile-name').value = profile.name;
        document.getElementById('profile-color').value = profile.color;
        document.getElementById('profile-color-text').value = profile.color;
    }

    showProfileModal(title) {
        const modal = document.getElementById('profile-modal');
        const modalTitle = document.getElementById('profile-modal-title');
        const form = document.getElementById('profile-form');
        
        modalTitle.textContent = title;
        modal.classList.remove('hidden');
        
        // Clear form if adding new profile
        if (!this.editingProfile) {
            form.reset();
            document.getElementById('profile-color').value = '#00C700';
            document.getElementById('profile-color-text').value = '#00C700';
        }
        
        // Focus on name input
        document.getElementById('profile-name').focus();
    }

    hideProfileModal() {
        const modal = document.getElementById('profile-modal');
        modal.classList.add('hidden');
        this.editingProfile = null;
    }

    async onProfileSaveClick() {
        const saveBtn = document.getElementById('profile-save-btn');
        const errorDiv = document.getElementById('manage-profiles-error');
        const successDiv = document.getElementById('manage-profiles-success');
        
        try {
            saveBtn.disabled = true;
            saveBtn.textContent = 'Saving...';
            errorDiv.classList.add('hidden');
            successDiv.classList.add('hidden');

            const name = document.getElementById('profile-name').value.trim();
            const color = document.getElementById('profile-color').value;

            // Validate input
            if (!name) {
                throw new Error('Profile name cannot be empty');
            }

            if (!this.isValidColor(color)) {
                throw new Error('Please select a valid color');
            }

            let result;
            if (this.editingProfile) {
                // Update existing profile
                const updatedProfile = {
                    ...this.editingProfile,
                    name: name,
                    color: color
                };
                result = await Backend.updateProfile(updatedProfile);
                
                // Update local config immediately with the result from server
                Profiles.updateProfile(result.profile);
            } else {
                // Create new profile
                const newProfile = {
                    name: name,
                    color: color
                };
                result = await Backend.createProfile(newProfile);
                
                // Update local config immediately with the returned profile (includes ID)
                Profiles.addProfile(result.profile);
            }

            // Refresh profiles list
            await this.loadProfiles();

            // Show success message
            successDiv.textContent = this.editingProfile ? 'Profile updated successfully!' : 'Profile created successfully!';
            successDiv.classList.remove('hidden');

            // Hide modal
            this.hideProfileModal();
            
            // Hide success message after a delay
            setTimeout(() => {
                successDiv.classList.add('hidden');
            }, 2000);

        } catch (error) {
            console.error('Error saving profile:', error);
            errorDiv.textContent = error.message || 'Failed to save profile';
            errorDiv.classList.remove('hidden');
        } finally {
            saveBtn.disabled = false;
            saveBtn.textContent = 'Save Profile';
        }
    }

    async deleteProfile(profileId) {
        const errorDiv = document.getElementById('manage-profiles-error');
        const successDiv = document.getElementById('manage-profiles-success');
        
        try {
            errorDiv.classList.add('hidden');
            successDiv.classList.add('hidden');

            // Check if this is the last profile
            const profiles = Profiles.getProfiles();
            if (profiles.length <= 1) {
                errorDiv.textContent = 'Cannot delete the last remaining profile. You must have at least one profile.';
                errorDiv.classList.remove('hidden');
                return;
            }

            // Confirm deletion
            if (!confirm('Are you sure you want to delete this profile? This action cannot be undone.')) {
                return;
            }

            await Backend.deleteProfile(profileId);

            // Update local config immediately
            Profiles.removeProfile(profileId);

            // Refresh profiles list
            await this.loadProfiles();

            // Show success message
            successDiv.textContent = 'Profile deleted successfully!';
            successDiv.classList.remove('hidden');
            
            // Hide success message after a delay
            setTimeout(() => {
                successDiv.classList.add('hidden');
            }, 2000);

        } catch (error) {
            console.error('Error deleting profile:', error);
            errorDiv.textContent = error.message || 'Failed to delete profile';
            errorDiv.classList.remove('hidden');
        }
    }

    isValidColor(color) {
        return /^#[0-9A-F]{6}$/i.test(color);
    }

    renderProfiles(profiles) {
        const profilesGrid = document.getElementById('profiles-grid');
        const emptyState = document.getElementById('profiles-empty-state');
        
        if (!profiles || profiles.length === 0) {
            profilesGrid.innerHTML = '';
            emptyState.classList.remove('hidden');
            return;
        }

        emptyState.classList.add('hidden');
        
        profilesGrid.innerHTML = profiles.map(profile => `
            <div class="bg-gray-800 border border-gray-600 rounded-lg p-4 hover:border-gray-500 transition-colors">
                <div class="flex items-center justify-between mb-3">
                    <div class="flex items-center space-x-3 overflow-hidden">
                        <div class="w-8 h-8 rounded-full flex items-center justify-center text-white text-sm font-semibold flex-shrink-0" style="background-color: ${profile.color}">
                            ${getInitials(profile.name)}
                        </div>
                        <h3 class="text-lg font-semibold text-white truncate" title="${escapeHtml(profile.name)}">${escapeHtml(profile.name)}</h3>
                    </div>
                    <div class="flex space-x-2">
                        <button class="edit-profile-btn text-blue-400 hover:text-blue-300 p-1" data-profile-id="${profile.id}">
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
                            </svg>
                        </button>
                        <button class="delete-profile-btn text-red-400 hover:text-red-300 p-1" data-profile-id="${profile.id}">
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                            </svg>
                        </button>
                    </div>
                </div>
                <div class="text-sm text-gray-400">
                    <span class="inline-block w-3 h-3 rounded-full mr-2" style="background-color: ${profile.color}"></span>
                    ${escapeHtml(profile.color)}
                </div>
            </div>
        `).join('');

        // Add event listeners to the dynamically created buttons
        profilesGrid.querySelectorAll('.edit-profile-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const profileId = btn.getAttribute('data-profile-id');
                const profile = profiles.find(p => p.id === profileId);
                if (profile) {
                    this.showEditProfileModal(profile);
                }
            });
        });

        profilesGrid.querySelectorAll('.delete-profile-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const profileId = btn.getAttribute('data-profile-id');
                this.deleteProfile(profileId);
            });
        });
    }

    async loadProfiles() {
        try {
            // Use config from model instead of API call
            if (!Profiles.isLoaded()) {
                throw new Error('Profiles not loaded');
            }
            
            const profiles = Profiles.getProfiles();
            this.renderProfiles(profiles);
            
        } catch (error) {
            console.error('Error loading profiles:', error);
            const errorDiv = document.getElementById('manage-profiles-error');
            errorDiv.textContent = 'Failed to load profiles';
            errorDiv.classList.remove('hidden');
            
            // Show empty state
            const emptyState = document.getElementById('profiles-empty-state');
            emptyState.classList.remove('hidden');
        }
    }

    async onShow() {
        try {
            // Hide any previous error/success messages
            const errorDiv = document.getElementById('manage-profiles-error');
            const successDiv = document.getElementById('manage-profiles-success');
            errorDiv.classList.add('hidden');
            successDiv.classList.add('hidden');
            
            // Load and display profiles
            await this.loadProfiles();
            
        } catch (error) {
            console.error('Error showing profiles page:', error);
        }
    }
}
