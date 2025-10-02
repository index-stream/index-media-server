import Page from './Page.js';
import { PAGES } from '../constants.js';
import PageController from '../controllers/PageController.js';
import Backend from '../clients/Backend.js';

export default class SetupPage extends Page {
    constructor() {
        super(PAGES.SETUP);
        this.serverName = 'My Index Media'; // Store server name
        this.profiles = []; // Store multiple profiles
        this.currentProfile = { name: '', color: '#3B82F6' }; // Current profile being edited
        this.serverPassword = ''; // Store server password
        this.selectedIcon = 'home'; // Default selection
        this.indexName = 'Home'; // Store the entered name
        this.mediaType = 'videos'; // Store selected media type (default to videos)
        this.selectedFolders = []; // Store selected folder paths
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Setup button
        const setupBtn = document.getElementById('setup-btn');
        setupBtn?.addEventListener('click', () => this.onSetupBtnClick());

        // Step 1: Server name form
        const serverNameForm = document.getElementById('server-name-form');
        serverNameForm?.addEventListener('submit', (e) => this.handleServerNameSubmit(e));

        // Step 2: Profile management
        const addProfileBtn = document.getElementById('add-profile-btn');
        addProfileBtn?.addEventListener('click', () => this.addNewProfile());

        const continueBtn = document.getElementById('continue-to-password-btn');
        continueBtn?.addEventListener('click', () => this.handleProfileContinue());

        const backToServerNameBtn = document.getElementById('back-to-server-name-btn');
        backToServerNameBtn?.addEventListener('click', () => this.backToServerNameStep());

        // Step 3: Password form
        const passwordForm = document.getElementById('password-form');
        passwordForm?.addEventListener('submit', (e) => this.handlePasswordSubmit(e));

        const backToProfilesBtn = document.getElementById('back-to-profiles-btn');
        backToProfilesBtn?.addEventListener('click', () => this.backToProfileStep());

        // Step 4: Name form
        const nameForm = document.getElementById('name-form');
        nameForm?.addEventListener('submit', (e) => this.handleNameSubmit(e));

        // Step 4: Skip (same as skip from step 3)
        const skipBtn = document.getElementById('skip-btn');
        skipBtn?.addEventListener('click', () => this.onSkipClick());

        // Step 4: Back to password
        const backToPasswordBtn = document.getElementById('back-to-password-btn');
        backToPasswordBtn?.addEventListener('click', () => this.backToPasswordStep());

        // Step 4: Media type selection
        const mediaTypeOptions = document.querySelectorAll('.media-type-option');
        mediaTypeOptions.forEach(option => {
            option.addEventListener('click', () => this.selectMediaType(option));
        });

        // Step 5: Icon selection
        const iconOptions = document.querySelectorAll('.icon-option');
        iconOptions.forEach(option => {
            option.addEventListener('click', () => this.selectIcon(option));
        });

        // Step 5: Custom file upload (hidden input)
        const customIconInput = document.getElementById('custom-icon');
        customIconInput?.addEventListener('change', (e) => this.handleCustomIcon(e));

        // Step 5: Icon form
        const iconForm = document.getElementById('icon-form');
        iconForm?.addEventListener('submit', (e) => this.handleIconSubmit(e));

        // Step 5: Back to name
        const backToNameBtn = document.getElementById('back-to-name-btn');
        backToNameBtn?.addEventListener('click', () => this.backToNameStep());

        // Step 6: Add folders button
        const addFoldersBtn = document.getElementById('add-folders-btn');
        addFoldersBtn?.addEventListener('click', () => this.onAddFoldersClick());

        // Step 6: Folders form
        const foldersForm = document.getElementById('folders-form');
        foldersForm?.addEventListener('submit', (e) => this.handleFoldersSubmit(e));

        // Step 6: Back to icon
        const backToIconBtn = document.getElementById('back-to-icon-btn');
        backToIconBtn?.addEventListener('click', () => this.backToIconStep());
    }

    onShow() {
        console.log('Setup page shown');
        // Reset form state
        this.resetForm();
    }

    onSetupBtnClick() {
        console.log('Setup button clicked');
        this.showStep1();
    }

    handleServerNameSubmit(event) {
        event.preventDefault();
        
        const formData = new FormData(event.target);
        this.serverName = formData.get('server-name') || 'My Index Media';
        
        console.log('Server name entered:', this.serverName);
        this.showStep2();
    }

    showStep1() {
        const initialContent = document.getElementById('setup-initial-content');
        const step1 = document.getElementById('setup-step-1');
        
        if (initialContent && step1) {
            initialContent.classList.add('hidden');
            step1.classList.remove('hidden');
        }
    }

    showStep2() {
        const step1 = document.getElementById('setup-step-1');
        const step2 = document.getElementById('setup-step-2');
        
        if (step1 && step2) {
            step1.classList.add('hidden');
            step2.classList.remove('hidden');
        }
    }

    backToServerNameStep() {
        const step1 = document.getElementById('setup-step-1');
        const step2 = document.getElementById('setup-step-2');
        
        if (step1 && step2) {
            step2.classList.add('hidden');
            step1.classList.remove('hidden');
        }
    }

    addNewProfile() {
        // Generate a default name
        const defaultName = `Profile ${this.profiles.length + 1}`;
        const defaultColor = '#3B82F6';
        
        // Add profile to the list
        this.profiles.push({ name: defaultName, color: defaultColor });
        this.updateProfilesList();
        this.updateContinueButton();
    }

    getTextColorForBackground(hexColor) {
        // Remove # if present
        const hex = hexColor.replace('#', '');
        
        // Convert to RGB
        const r = parseInt(hex.substr(0, 2), 16);
        const g = parseInt(hex.substr(2, 2), 16);
        const b = parseInt(hex.substr(4, 2), 16);
        
        // Calculate brightness using luminance formula
        const brightness = (r * 299 + g * 587 + b * 114) / 1000;
        
        // Return black for light backgrounds, white for dark backgrounds
        return brightness > 160 ? '#000000' : '#FFFFFF';
    }

    updateProfilesList() {
        const profilesList = document.getElementById('profiles-list');
        if (!profilesList) return;

        if (this.profiles.length === 0) {
            profilesList.innerHTML = '<p class="text-gray-500 text-sm text-center py-8">No profiles added yet</p>';
            return;
        }

        // Clear the list first
        profilesList.innerHTML = '';

        // Create profile items
        this.profiles.forEach((profile, index) => {
            const profileItem = document.createElement('div');
            profileItem.className = 'flex items-center justify-between bg-gray-600 rounded-lg p-4';
            
            profileItem.innerHTML = `
                <div class="flex items-center justify-between">
                    <div class="flex items-center">
                        <div class="relative mr-4">
                            <input type="color" value="${profile.color}" 
                                   class="w-10 h-10 rounded-full opacity-0 absolute z-10 pointer-events-none" 
                                   data-index="${index}" data-edit="color">
                            <div class="w-10 h-10 rounded-full flex items-center justify-center font-semibold border-2 border-gray-500 cursor-pointer hover:scale-105 transition-transform" 
                                 style="background-color: ${profile.color}; color: ${this.getTextColorForBackground(profile.color)}" data-index="${index}" data-edit="color">
                                ${profile.name.charAt(0).toUpperCase()}
                            </div>
                        </div>
                        <div class="flex-1">
                            <input type="text" value="${profile.name}" 
                                   class="text-white font-medium bg-gray-500 border border-gray-400 rounded-sm px-2 py-1 w-full focus:outline-hidden focus:ring-2 focus:ring-blue-500 focus:border-transparent" 
                                   data-index="${index}" data-edit="name">
                            <div class="text-gray-400 text-sm font-mono cursor-pointer hover:text-blue-400 transition-colors text-left" 
                                 data-index="${index}" data-edit="color">${profile.color}</div>
                        </div>
                    </div>
                </div>
                <button type="button" class="text-red-400 hover:text-red-300 transition-colors p-2" data-index="${index}" data-action="remove">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                    </svg>
                </button>
            `;

            // Add event listeners
            const nameInput = profileItem.querySelector('[data-edit="name"]');
            const colorInput = profileItem.querySelector('input[type="color"]');
            const hexDisplay = profileItem.querySelector('.text-gray-400');
            const circleDiv = profileItem.querySelector('.w-10.h-10.rounded-full.cursor-pointer');
            const removeBtn = profileItem.querySelector('[data-action="remove"]');
            
            nameInput.addEventListener('blur', () => this.saveProfileName(index));
            nameInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    e.target.blur();
                }
            });
            
            colorInput.addEventListener('change', () => this.saveProfileColor(index));
            hexDisplay.addEventListener('click', () => colorInput.click());
            circleDiv.addEventListener('click', () => colorInput.click());
            removeBtn.addEventListener('click', () => this.removeProfile(index));

            profilesList.appendChild(profileItem);
        });
    }

    saveProfileName(index) {
        const nameInput = document.querySelector(`[data-index="${index}"][data-edit="name"]`);
        if (!nameInput) return;
        
        const trimmedName = nameInput.value.trim();
        if (!trimmedName) {
            // Reset to original name if empty
            nameInput.value = this.profiles[index].name;
            return;
        }
        
        // Check for duplicate names
        if (this.profiles.some((p, i) => i !== index && p.name.toLowerCase() === trimmedName.toLowerCase())) {
            alert('A profile with this name already exists');
            nameInput.value = this.profiles[index].name;
            return;
        }
        
        this.profiles[index].name = trimmedName;
        // Update the circle with new initial
        this.updateProfileCircle(index);
    }

    saveProfileColor(index) {
        const colorInput = document.querySelector(`[data-index="${index}"][data-edit="color"]`);
        if (!colorInput) return;
        
        const newColor = colorInput.value.toUpperCase();
        this.profiles[index].color = newColor;
        
        // Update the visual circle and hex display
        this.updateProfileCircle(index);
        this.updateProfileColorDisplay(index);
    }

    updateProfileCircle(index) {
        const profile = this.profiles[index];
        const profileItem = document.querySelector(`[data-index="${index}"]`).closest('.flex.items-center.justify-between');
        if (profileItem) {
            const circle = profileItem.querySelector('.w-10.h-10.rounded-full.cursor-pointer');
            if (circle) {
                circle.style.backgroundColor = profile.color;
                circle.style.color = this.getTextColorForBackground(profile.color);
                circle.textContent = profile.name.charAt(0).toUpperCase();
            }
        }
    }

    updateProfileColorDisplay(index) {
        const profile = this.profiles[index];
        const profileItem = document.querySelector(`[data-index="${index}"]`).closest('.flex.items-center.justify-between');
        if (profileItem) {
            const colorDisplay = profileItem.querySelector('.text-gray-400');
            if (colorDisplay) {
                colorDisplay.textContent = profile.color;
            }
        }
    }

    removeProfile(index) {
        if (this.profiles.length <= 1) {
            alert('You must have at least one profile');
            return;
        }
        
        this.profiles.splice(index, 1);
        this.updateProfilesList();
        this.updateContinueButton();
    }

    updateContinueButton() {
        const continueBtn = document.getElementById('continue-to-password-btn');
        const errorMsg = document.getElementById('profiles-error');
        
        if (continueBtn && errorMsg) {
            const hasProfiles = this.profiles.length > 0;
            continueBtn.disabled = !hasProfiles;
            
            if (hasProfiles) {
                errorMsg.classList.add('hidden');
            } else {
                errorMsg.classList.remove('hidden');
            }
        }
    }

    handleProfileContinue() {
        if (this.profiles.length === 0) {
            const errorMsg = document.getElementById('profiles-error');
            if (errorMsg) {
                errorMsg.classList.remove('hidden');
            }
            return;
        }
        
        console.log('Profiles data:', this.profiles);
        this.showStep3();
    }

    handlePasswordSubmit(event) {
        event.preventDefault();
        
        const formData = new FormData(event.target);
        const password = formData.get('server-password');
        const confirmPassword = formData.get('confirm-password');
        
        // Hide any existing error message
        const errorElement = document.getElementById('password-error');
        if (errorElement) {
            errorElement.classList.add('hidden');
        }
        
        // Validate passwords match if password is provided
        if (password && password !== confirmPassword) {
            // Show error message
            if (errorElement) {
                errorElement.classList.remove('hidden');
            }
            return;
        }
        
        // Store the password
        this.serverPassword = password || '';
        
        console.log('Password set:', this.serverPassword ? 'Yes' : 'No');
        this.showStep4();
    }

    backToPasswordStep() {
        const step3 = document.getElementById('setup-step-3');
        const step4 = document.getElementById('setup-step-4');
        
        if (step3 && step4) {
            step4.classList.add('hidden');
            step3.classList.remove('hidden');
        }
    }

    backToProfileStep() {
        const step2 = document.getElementById('setup-step-2');
        const step3 = document.getElementById('setup-step-3');
        
        if (step2 && step3) {
            step3.classList.add('hidden');
            step2.classList.remove('hidden');
        }
    }

    showStep3() {
        const step2 = document.getElementById('setup-step-2');
        const step3 = document.getElementById('setup-step-3');
        
        if (step2 && step3) {
            step2.classList.add('hidden');
            step3.classList.remove('hidden');
            
            // Initialize media type selection (default to videos)
            this.initializeMediaTypeSelection();
        }
    }

    showStep4() {
        const step3 = document.getElementById('setup-step-3');
        const step4 = document.getElementById('setup-step-4');
        
        if (step3 && step4) {
            step3.classList.add('hidden');
            step4.classList.remove('hidden');
            
            // Update the display name in step 4
            const nameDisplay = document.getElementById('index-name-display');
            if (nameDisplay) {
                nameDisplay.textContent = this.indexName;
            }
        }
    }

    showStep5() {
        const step4 = document.getElementById('setup-step-4');
        const step5 = document.getElementById('setup-step-5');
        
        if (step4 && step5) {
            step4.classList.add('hidden');
            step5.classList.remove('hidden');
            
            // Update the display name in step 5
            const nameDisplay = document.getElementById('index-name-display');
            if (nameDisplay) {
                nameDisplay.textContent = this.indexName;
            }
        }
    }

    showStep6() {
        const step5 = document.getElementById('setup-step-5');
        const step6 = document.getElementById('setup-step-6');
        
        if (step5 && step6) {
            step5.classList.add('hidden');
            step6.classList.remove('hidden');
            
            // Update the display name in step 6
            const nameDisplay = document.getElementById('index-name-display-step6');
            if (nameDisplay) {
                nameDisplay.textContent = this.indexName;
            }
            
            // Update folders list
            this.updateFoldersList();
        }
    }


    backToNameStep() {
        const step4 = document.getElementById('setup-step-4');
        const step5 = document.getElementById('setup-step-5');
        
        if (step4 && step5) {
            step5.classList.add('hidden');
            step4.classList.remove('hidden');
        }
    }

    selectIcon(optionElement) {
        const iconType = optionElement.dataset.icon;
        
        if (iconType === 'custom') {
            // Trigger file input for custom upload
            const customIconInput = document.getElementById('custom-icon');
            customIconInput?.click();
            return;
        }

        // If switching from custom to default icon, clear the file input and reset custom option
        if (this.selectedIcon === 'custom') {
            this.clearCustomUpload();
        }

        // Remove selection from all options
        document.querySelectorAll('.icon-option').forEach(option => {
            const img = option.querySelector('img') || option.querySelector('div');
            if (img) {
                img.classList.remove('border-blue-500');
                img.classList.add('border-transparent');
            }
        });

        // Add selection to clicked option
        const img = optionElement.querySelector('img') || optionElement.querySelector('div');
        if (img) {
            img.classList.remove('border-transparent');
            img.classList.add('border-blue-500');
        }

        // Update selected icon
        this.selectedIcon = iconType;
        console.log('Selected icon:', this.selectedIcon);
    }

    selectMediaType(optionElement) {
        const mediaType = optionElement.dataset.type;
        
        // Remove selection from all options
        document.querySelectorAll('.media-type-option').forEach(option => {
            const img = option.querySelector('img');
            if (img) {
                img.classList.remove('border-blue-500');
                img.classList.add('border-transparent');
            }
        });

        // Add selection to clicked option
        const img = optionElement.querySelector('img');
        if (img) {
            img.classList.remove('border-transparent');
            img.classList.add('border-blue-500');
        }

        // Update selected media type
        this.mediaType = mediaType;
        console.log('Selected media type:', this.mediaType);
    }

    initializeMediaTypeSelection() {
        // Clear all selections first
        document.querySelectorAll('.media-type-option').forEach(option => {
            const img = option.querySelector('img');
            if (img) {
                img.classList.remove('border-blue-500');
                img.classList.add('border-transparent');
            }
        });

        // Select the default media type (videos)
        const videosOption = document.querySelector('.media-type-option[data-type="videos"]');
        if (videosOption) {
            const img = videosOption.querySelector('img');
            if (img) {
                img.classList.remove('border-transparent');
                img.classList.add('border-blue-500');
            }
        }
    }

    handleCustomIcon(event) {
        const file = event.target.files[0];
        if (file) {
            // Clear default icon selection
            document.querySelectorAll('.icon-option').forEach(option => {
                const img = option.querySelector('img') || option.querySelector('div');
                if (img) {
                    img.classList.remove('border-blue-500');
                    img.classList.add('border-transparent');
                }
            });

            // Update the custom upload option to show the uploaded image
            const customOption = document.querySelector('[data-icon="custom"]');
            if (customOption) {
                const customDiv = customOption.querySelector('div');
                if (customDiv) {
                    // Create image preview
                    const reader = new FileReader();
                    reader.onload = (e) => {
                        // Replace the pencil icon with the uploaded image
                        customDiv.innerHTML = `<img src="${e.target.result}" alt="Custom icon" class="w-16 h-16 rounded-lg object-contain">`;
                        customDiv.classList.remove('border-dashed', 'border-gray-400', 'bg-gray-700', 'border-transparent');
                        customDiv.classList.add('border-solid', 'border-blue-500');
                    };
                    reader.readAsDataURL(file);
                }
            }

            this.selectedIcon = 'custom';
            console.log('Custom icon selected:', file.name);
        }
    }

    clearCustomUpload() {
        // Clear the file input
        const customIconInput = document.getElementById('custom-icon');
        if (customIconInput) {
            customIconInput.value = '';
        }

        // Reset the custom upload option to pencil icon
        const customOption = document.querySelector('[data-icon="custom"]');
        if (customOption) {
            const customDiv = customOption.querySelector('div');
            if (customDiv) {
                customDiv.innerHTML = `
                    <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"></path>
                    </svg>
                `;
                customDiv.classList.remove('border-solid', 'border-blue-500');
                customDiv.classList.add('border-dashed', 'border-gray-400', 'bg-gray-700', 'border-transparent');
            }
        }
    }

    handleNameSubmit(event) {
        event.preventDefault();
        
        const formData = new FormData(event.target);
        this.indexName = formData.get('index-name') || 'Home';
        
        console.log('Index name entered:', this.indexName);
        this.showStep5();
    }

    handleIconSubmit(event) {
        event.preventDefault();
        
        const formData = new FormData(event.target);
        const customFile = formData.get('custom-icon');

        const setupData = {
            name: this.indexName,
            icon: this.selectedIcon,
            ...(this.selectedIcon === 'custom' ? { customFile: customFile } : {})
        };

        console.log('Complete index setup data:', setupData);
        this.onContinueToFolders();
    }

    onContinueToFolders() {
        console.log('Continue to folders clicked - processing setup...');
        this.showStep6();
    }

    showStep3() {
        const step2 = document.getElementById('setup-step-2');
        const step3 = document.getElementById('setup-step-3');
        
        if (step2 && step3) {
            step2.classList.add('hidden');
            step3.classList.remove('hidden');
            
            // Update the display name in step 3
            const nameDisplay = document.getElementById('index-name-display-step3');
            if (nameDisplay) {
                nameDisplay.textContent = this.indexName;
            }
            
            // Update folders list
            this.updateFoldersList();
        }
    }

    backToIconStep() {
        const step5 = document.getElementById('setup-step-5');
        const step6 = document.getElementById('setup-step-6');
        
        if (step5 && step6) {
            step6.classList.add('hidden');
            step5.classList.remove('hidden');
        }
    }

    async onAddFoldersClick() {
        try {
            console.log('Add folders clicked - requesting folder selection...');
            
            // Call Tauri HTTP server for folder selection
            const data = await Backend.getFolders();

            if (data.error) {
                throw new Error(data.error);
            }
            
            if (data.folders && data.folders.length > 0) {
                // Add new folders to selection (avoid duplicates)
                const newFolders = data.folders.filter(folder => !this.selectedFolders.includes(folder));
                this.selectedFolders.push(...newFolders);
                
                // Filter out redundant child folders
                this.selectedFolders = this.filterRedundantFolders(this.selectedFolders);
                
                this.updateFoldersList();
                console.log('Added folders:', newFolders);
            } else {
                console.log('No folders selected');
            }
        } catch (error) {
            console.error('Error selecting folders:', error);
            alert('Failed to select folders. Please make sure the Index Media Server app is running.');
        }
    }

    removeFolder(folderPath) {
        this.selectedFolders = this.selectedFolders.filter(folder => folder !== folderPath);
        this.updateFoldersList();
        console.log('Removed folder:', folderPath);
    }

    filterRedundantFolders(folders) {
        if (folders.length === 0) {
            return folders;
        }
        
        // Sort folders by path length (shorter paths first)
        const sortedFolders = [...folders].sort((a, b) => a.length - b.length);
        
        const filtered = [];
        
        for (const folder of sortedFolders) {
            // Check if this folder is a child of any already filtered folder
            const isChild = filtered.some(parent => {
                return folder.startsWith(parent) && 
                       folder.length > parent.length && 
                       (folder.charAt(parent.length) === '/' || folder.charAt(parent.length) === '\\');
            });
            
            if (!isChild) {
                filtered.push(folder);
            }
        }
        
        return filtered;
    }

    updateFoldersList() {
        const foldersList = document.getElementById('selected-folders-list');
        if (!foldersList) return;

        if (this.selectedFolders.length === 0) {
            foldersList.innerHTML = '<p class="text-gray-500 text-sm text-center py-8">No folders selected yet</p>';
            return;
        }

        // Clear the list first
        foldersList.innerHTML = '';

        // Create folder items with event listeners
        this.selectedFolders.forEach(folder => {
            const folderItem = document.createElement('div');
            folderItem.className = 'flex items-center justify-between bg-gray-600 rounded-lg p-3';
            
            folderItem.innerHTML = `
                <div class="flex items-center">
                    <svg class="w-5 h-5 text-gray-400 mr-3 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2z"></path>
                    </svg>
                    <span class="text-white text-sm font-mono break-all">${folder}</span>
                </div>
                <button type="button" class="text-red-400 hover:text-red-300 transition-colors flex-shrink-0">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                    </svg>
                </button>
            `;

            // Add click event listener to the remove button
            const removeBtn = folderItem.querySelector('button');
            removeBtn.addEventListener('click', () => this.removeFolder(folder));

            foldersList.appendChild(folderItem);
        });
    }

    handleFoldersSubmit(event) {
        event.preventDefault();
        this.onFinishSetup();
    }

    getCustomFile() {
        const customIconInput = document.getElementById('custom-icon');
        return customIconInput?.files[0] || null;
    }

    async fileToBase64(file) {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.readAsDataURL(file);
            reader.onload = () => {
                // Remove the data URL prefix (e.g., "data:image/png;base64,")
                const base64 = reader.result.split(',')[1];
                resolve(base64);
            };
            reader.onerror = error => reject(error);
        });
    }

    async onFinishSetup() {
        // Handle custom icon file conversion to base64
        let customIconFile = null;
        if (this.selectedIcon === 'custom') {
            const file = this.getCustomFile();
            if (file) {
                customIconFile = await this.fileToBase64(file);
            }
        }

        const config = {
            name: this.serverName,
            profiles: this.profiles,
            password: this.serverPassword,
            indexes: [{
                name: this.indexName,
                mediaType: this.mediaType,
                icon: this.selectedIcon,
                folders: this.selectedFolders,
                ...(customIconFile ? { customIconFile } : {})
            }]
        };

        try {
            console.log('Saving configuration...', config);
            
            // Store the config via Backend call
            const result = await Backend.saveConfiguration(config);
            
            if (result.success) {
                console.log('Configuration saved successfully!');
                console.log('Setup completed! Navigating to home page...');
                
                // Navigate to home page
                PageController.showPage(PAGES.HOME);
            } else {
                throw new Error(result.error || 'Failed to save configuration');
            }
        } catch (error) {
            console.error('Error saving configuration:', error);
            alert('Failed to save configuration. Please try again.');
        }
    }

    async onSkipClick() {
        const config = {
            name: this.serverName,
            profiles: this.profiles,
            password: this.serverPassword,
            indexes: []
        };
        await Backend.saveConfiguration(config);
        PageController.showPage(PAGES.HOME);
    }

    resetForm() {
        // Reset to initial state
        const initialContent = document.getElementById('setup-initial-content');
        const step1 = document.getElementById('setup-step-1');
        const step2 = document.getElementById('setup-step-2');
        const step3 = document.getElementById('setup-step-3');
        const step4 = document.getElementById('setup-step-4');
        const step5 = document.getElementById('setup-step-5');
        const step6 = document.getElementById('setup-step-6');
        
        if (initialContent && step1 && step2 && step3 && step4 && step5 && step6) {
            initialContent.classList.remove('hidden');
            step1.classList.add('hidden');
            step2.classList.add('hidden');
            step3.classList.add('hidden');
            step4.classList.add('hidden');
            step5.classList.add('hidden');
            step6.classList.add('hidden');
        }

        // Reset form values
        const serverNameInput = document.getElementById('server-name');
        if (serverNameInput) serverNameInput.value = 'My Index Media';

        const nameInput = document.getElementById('index-name');
        if (nameInput) nameInput.value = 'Home';

        const customIconInput = document.getElementById('custom-icon');
        if (customIconInput) customIconInput.value = '';

        // Reset stored values
        this.serverName = 'My Index Media';
        this.profiles = [];
        this.indexName = 'Home';
        this.mediaType = 'videos';
        this.selectedIcon = 'home';
        this.selectedFolders = [];

        // Reset profile UI
        this.updateProfilesList();
        this.updateContinueButton();

        // Reset icon selection
        document.querySelectorAll('.icon-option').forEach(option => {
            const img = option.querySelector('img') || option.querySelector('div');
            if (img) {
                img.classList.remove('border-blue-500');
                img.classList.add('border-transparent');
            }
        });

        // Reset custom upload option to pencil icon
        this.clearCustomUpload();

        // Select home icon by default
        const homeOption = document.querySelector('[data-icon="home"]');
        if (homeOption) {
            this.selectIcon(homeOption);
        }

        // Reset media type selection
        document.querySelectorAll('.media-type-option').forEach(option => {
            const img = option.querySelector('img');
            if (img) {
                img.classList.remove('border-blue-500');
                img.classList.add('border-transparent');
            }
        });

        // Select videos by default
        const videosOption = document.querySelector('.media-type-option[data-type="videos"]');
        if (videosOption) {
            const img = videosOption.querySelector('img');
            if (img) {
                img.classList.remove('border-transparent');
                img.classList.add('border-blue-500');
            }
        }
    }
}
