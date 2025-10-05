import Page from './Page.js';
import { PAGES } from '../constants.js';
import Backend from '../clients/Backend.js';
import Config from '../models/Config.js';

export default class ServerSettingsPage extends Page {
    constructor() {
        super(PAGES.SERVER_SETTINGS);
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Back button
        const backBtn = document.getElementById('server-settings-back-btn');
        backBtn?.addEventListener('click', () => this.onBackClick());

        // Update buttons
        const updateNameBtn = document.getElementById('update-name-btn');
        updateNameBtn?.addEventListener('click', () => this.showNameEditMode());

        const updatePasswordBtn = document.getElementById('update-password-btn');
        updatePasswordBtn?.addEventListener('click', () => this.showPasswordEditMode());

        // Cancel buttons
        const cancelNameBtn = document.getElementById('cancel-name-update-btn');
        cancelNameBtn?.addEventListener('click', (e) => {
            e.preventDefault();
            this.hideNameEditMode();
        });

        const cancelPasswordBtn = document.getElementById('cancel-password-update-btn');
        cancelPasswordBtn?.addEventListener('click', (e) => {
            e.preventDefault();
            this.hidePasswordEditMode();
        });

        // Save buttons
        const saveNameBtn = document.getElementById('save-name-btn');
        saveNameBtn?.addEventListener('click', (e) => {
            e.preventDefault();
            this.onNameSaveClick();
        });

        // Save password button
        const savePasswordBtn = document.getElementById('save-password-btn');
        savePasswordBtn?.addEventListener('click', (e) => {
            e.preventDefault();
            this.onPasswordSaveClick();
        });

        // Password validation
        const passwordInput = document.getElementById('server-password-input');
        const confirmPasswordInput = document.getElementById('confirm-password-input');
        
        passwordInput?.addEventListener('input', () => this.validatePasswordForm());
        confirmPasswordInput?.addEventListener('input', () => this.validatePasswordForm());

        // Enter key handling for name input
        const nameInput = document.getElementById('server-name-input');
        nameInput?.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                this.onNameSaveClick();
            } else if (e.key === 'Escape') {
                e.preventDefault();
                this.hideNameEditMode();
            }
        });

        // Enter key handling for password inputs
        passwordInput?.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                this.onPasswordSaveClick();
            } else if (e.key === 'Escape') {
                e.preventDefault();
                this.hidePasswordEditMode();
            }
        });
        
        confirmPasswordInput?.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                this.onPasswordSaveClick();
            } else if (e.key === 'Escape') {
                e.preventDefault();
                this.hidePasswordEditMode();
            }
        });
    }

    onBackClick() {
        // Navigate back to home page
        document.dispatchEvent(new CustomEvent('navigate', {
            detail: { page: PAGES.HOME }
        }));
    }

    showNameEditMode() {
        const displayDiv = document.getElementById('server-name-display');
        const editDiv = document.getElementById('server-name-edit');
        
        // Hide display mode, show edit mode
        displayDiv?.classList.add('hidden');
        editDiv?.classList.remove('hidden');
        
        // Set current value in input and focus
        const nameInput = document.getElementById('server-name-input');
        const currentName = document.getElementById('current-server-name').textContent;
        nameInput.value = currentName;
        nameInput.focus();
        nameInput.select(); // Select all text for easy replacement
    }

    hideNameEditMode() {
        const displayDiv = document.getElementById('server-name-display');
        const editDiv = document.getElementById('server-name-edit');
        
        // Hide edit mode, show display mode
        editDiv?.classList.add('hidden');
        displayDiv?.classList.remove('hidden');
        
        // Clear the input
        const nameInput = document.getElementById('server-name-input');
        nameInput.value = '';
    }

    showPasswordEditMode() {
        const displayDiv = document.getElementById('password-display');
        const editDiv = document.getElementById('password-edit');
        
        // Hide display mode, show edit mode
        displayDiv?.classList.add('hidden');
        editDiv?.classList.remove('hidden');
        
        // Focus on the password input
        const passwordInput = document.getElementById('server-password-input');
        passwordInput?.focus();
    }

    hidePasswordEditMode() {
        const displayDiv = document.getElementById('password-display');
        const editDiv = document.getElementById('password-edit');
        
        // Hide edit mode, show display mode
        editDiv?.classList.add('hidden');
        displayDiv?.classList.remove('hidden');
        
        // Clear the inputs
        const passwordInput = document.getElementById('server-password-input');
        const confirmPasswordInput = document.getElementById('confirm-password-input');
        passwordInput.value = '';
        confirmPasswordInput.value = '';
        
        // Reset validation
        this.validatePasswordForm();
    }

    async onNameSaveClick() {
        const saveBtn = document.getElementById('save-name-btn');
        const errorDiv = document.getElementById('server-settings-error');
        
        try {
            saveBtn.disabled = true;
            saveBtn.textContent = 'Saving...';
            errorDiv.classList.add('hidden');

            const serverName = document.getElementById('server-name-input').value.trim();

            // Validate input
            if (!serverName) {
                throw new Error('Server name cannot be empty');
            }

            // Update server name
            await Backend.updateServerName(serverName);

            // Update the displayed name and local config
            const currentNameElement = document.getElementById('current-server-name');
            currentNameElement.textContent = serverName;
            Config.updateServerName(serverName);

            // Show success message
            errorDiv.textContent = 'Server name updated successfully!';
            errorDiv.classList.remove('hidden', 'text-red-400');
            errorDiv.classList.add('text-green-400');

            // Immediately hide edit mode and show updated name
            this.hideNameEditMode();
            
            // Hide success message after a delay
            setTimeout(() => {
                errorDiv.classList.add('hidden');
            }, 2000);

        } catch (error) {
            console.error('Error updating server name:', error);
            errorDiv.textContent = error.message || 'Failed to update server name';
            errorDiv.classList.remove('hidden', 'text-green-400');
            errorDiv.classList.add('text-red-400');
        } finally {
            saveBtn.disabled = false;
            saveBtn.textContent = 'Save Name';
        }
    }

    async onPasswordSaveClick() {
        const saveBtn = document.getElementById('save-password-btn');
        const errorDiv = document.getElementById('server-settings-error');
        
        try {
            saveBtn.disabled = true;
            saveBtn.textContent = 'Saving...';
            errorDiv.classList.add('hidden');

            const password = document.getElementById('server-password-input').value;
            const confirmPassword = document.getElementById('confirm-password-input').value;

            // Validate inputs
            if (!password) {
                throw new Error('Password cannot be empty');
            }

            if (password !== confirmPassword) {
                throw new Error('Passwords do not match');
            }

            // Update server password
            await Backend.updateServerPassword(password);

            // Update local config
            Config.updatePassword(password);

            // Show success message
            errorDiv.textContent = 'Password updated successfully!';
            errorDiv.classList.remove('hidden', 'text-red-400');
            errorDiv.classList.add('text-green-400');

            // Immediately hide edit mode
            this.hidePasswordEditMode();
            
            // Hide success message after a delay
            setTimeout(() => {
                errorDiv.classList.add('hidden');
            }, 2000);

        } catch (error) {
            console.error('Error updating password:', error);
            errorDiv.textContent = error.message || 'Failed to update password';
            errorDiv.classList.remove('hidden', 'text-green-400');
            errorDiv.classList.add('text-red-400');
        } finally {
            saveBtn.disabled = false;
            saveBtn.textContent = 'Save Password';
        }
    }

    validatePasswordForm() {
        const saveBtn = document.getElementById('save-password-btn');
        const password = document.getElementById('server-password-input').value;
        const confirmPassword = document.getElementById('confirm-password-input').value;

        const isValid = password && password === confirmPassword;
        saveBtn.disabled = !isValid;
    }

    async onShow() {
        try {
            // Use config from model instead of API call
            if (!Config.isLoaded()) {
                throw new Error('Configuration not loaded');
            }
            
            // Display current server name from config
            const currentNameElement = document.getElementById('current-server-name');
            currentNameElement.textContent = Config.getServerName();
            
            // Hide any forms that might be open
            this.hideNameEditMode();
            this.hidePasswordEditMode();
            
            // Hide any previous error messages
            const errorDiv = document.getElementById('server-settings-error');
            errorDiv.classList.add('hidden');
            
        } catch (error) {
            console.error('Error loading server configuration:', error);
            const errorDiv = document.getElementById('server-settings-error');
            errorDiv.textContent = 'Configuration not available';
            errorDiv.classList.remove('hidden');
            
            // Show placeholder name if config fails
            const currentNameElement = document.getElementById('current-server-name');
            currentNameElement.textContent = 'Index Media Server';
        }
    }
}
