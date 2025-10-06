import { API_URL } from '../constants.js';
import Backend from '../clients/Backend.js';
import Config from '../models/Config.js';
import { escapeHtml } from '../utils/text.js';

export default class UpdateLocalIndexComponent {
    constructor() {
        this.editingIndex = null;
        this.selectedIcon = null;
        this.selectedFolders = [];
        this.customIconFile = null;
        this.hasCustomIconChanged = false; // Track if user uploaded a new custom icon
        this.currentView = 'main'; // 'main', 'name', 'icon', 'folders'
        this.modalId = 'update-index-modal';
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Modal controls
        const cancelBtn = document.getElementById('update-index-cancel-btn');
        cancelBtn?.addEventListener('click', () => this.hideModal());

        // Main view buttons
        const updateNameBtn = document.getElementById('update-index-name-btn');
        updateNameBtn?.addEventListener('click', () => this.showNameEdit());

        const updateIconBtn = document.getElementById('update-index-icon-btn');
        updateIconBtn?.addEventListener('click', () => this.showIconEdit());

        const updateFoldersBtn = document.getElementById('update-index-folders-btn');
        updateFoldersBtn?.addEventListener('click', () => this.showFoldersEdit());

        const saveBtn = document.getElementById('update-index-save-btn');
        saveBtn?.addEventListener('click', (e) => {
            e.preventDefault();
            this.onSaveClick();
        });

        // Name edit view
        const backFromNameBtn = document.getElementById('update-index-back-from-name-btn');
        backFromNameBtn?.addEventListener('click', () => this.showMainView());

        // Icon edit view
        const backFromIconBtn = document.getElementById('update-index-back-from-icon-btn');
        backFromIconBtn?.addEventListener('click', () => this.showMainView());

        // Folders edit view
        const backFromFoldersBtn = document.getElementById('update-index-back-from-folders-btn');
        backFromFoldersBtn?.addEventListener('click', () => this.showMainView());

        const addFoldersBtn = document.getElementById('update-index-add-folders-btn');
        addFoldersBtn?.addEventListener('click', () => this.selectFolders());

        // Custom icon file input
        const customIconInput = document.getElementById('update-custom-icon');
        customIconInput?.addEventListener('change', (e) => this.handleCustomIconUpload(e));

        // Icon selection (scoped to update modal)
        document.addEventListener('click', (e) => {
            if (e.target.closest('#update-index-modal .icon-option')) {
                const option = e.target.closest('.icon-option');
                this.selectIcon(option);
            }
        });
    }

    showModal(index) {
        // Create a deep clone of the index for editing
        this.editingIndex = JSON.parse(JSON.stringify(index));
        const modal = document.getElementById(this.modalId);
        modal.classList.remove('hidden');
        
        // Reset to main view
        this.currentView = 'main';
        this.showMainView();
        
        // Populate form with existing data
        this.selectedIcon = this.editingIndex.icon || 'home';
        this.selectedFolders = [...(this.editingIndex.folders || [])];
        this.customIconFile = null;
        this.hasCustomIconChanged = false; // Reset custom icon change tracking
        this.updateIconSelection();
        this.updateFoldersList();
        
        // Update main view display
        this.updateMainViewDisplay();
    }

    hideModal() {
        const modal = document.getElementById(this.modalId);
        modal.classList.add('hidden');
        this.editingIndex = null;
        this.currentView = 'main';
        this.selectedFolders = [];
        this.customIconFile = null;
        this.clearCustomUpload();
    }

    showMainView() {
        this.currentView = 'main';
        document.querySelectorAll(`#${this.modalId} [id^="update-index-view-"]`).forEach(viewEl => {
            viewEl.classList.add('hidden');
        });
        const mainView = document.getElementById('update-index-view-main');
        if (mainView) {
            mainView.classList.remove('hidden');
        }
        this.updateMainViewDisplay();
    }

    showNameEdit() {
        this.currentView = 'name';
        document.querySelectorAll(`#${this.modalId} [id^="update-index-view-"]`).forEach(viewEl => {
            viewEl.classList.add('hidden');
        });
        const nameView = document.getElementById('update-index-view-name');
        if (nameView) {
            nameView.classList.remove('hidden');
        }
        
        // Populate name field
        const nameInput = document.getElementById('update-index-name-input');
        nameInput.value = this.editingIndex.name;
        nameInput.focus();
        
        // Remove any existing listener and add new one to update the cloned index when name changes
        nameInput.removeEventListener('input', this._nameInputHandler);
        this._nameInputHandler = () => {
            this.editingIndex.name = nameInput.value;
            this.updateMainViewDisplay();
        };
        nameInput.addEventListener('input', this._nameInputHandler);
    }

    showIconEdit() {
        this.currentView = 'icon';
        document.querySelectorAll(`#${this.modalId} [id^="update-index-view-"]`).forEach(viewEl => {
            viewEl.classList.add('hidden');
        });
        const iconView = document.getElementById('update-index-view-icon');
        if (iconView) {
            iconView.classList.remove('hidden');
        }
        this.updateIconSelection();
    }

    showFoldersEdit() {
        this.currentView = 'folders';
        document.querySelectorAll(`#${this.modalId} [id^="update-index-view-"]`).forEach(viewEl => {
            viewEl.classList.add('hidden');
        });
        const foldersView = document.getElementById('update-index-view-folders');
        if (foldersView) {
            foldersView.classList.remove('hidden');
        }
        this.updateFoldersList();
    }

    updateMainViewDisplay() {
        if (!this.editingIndex) return;
        
        // Update index info display
        document.getElementById('update-index-name-display').textContent = this.editingIndex.name;
        document.getElementById('update-index-media-type-display').textContent = this.editingIndex.mediaType || 'Unknown';
        
        // Update icon display
        const iconDisplay = document.getElementById('update-index-icon-display');
        if (this.editingIndex.icon === 'custom') {
            // If user uploaded a new custom icon, show preview; otherwise use server URL
            if (this.hasCustomIconChanged && this.customIconFile) {
                // Show the uploaded custom icon preview
                const reader = new FileReader();
                reader.onload = (e) => {
                    iconDisplay.innerHTML = `<img src="${e.target.result}" alt="Custom icon" class="w-8 h-8 object-contain">`;
                };
                reader.readAsDataURL(this.customIconFile);
            } else {
                // Use server URL for existing custom icon
                iconDisplay.innerHTML = `
                    <img src="${API_URL}/index/${this.editingIndex.id}/icon" alt="Custom icon" class="w-8 h-8 object-contain" onerror="this.style.display='none'; this.nextElementSibling.style.display='flex';">
                    <div class="w-8 h-8 rounded-lg bg-gradient-to-r from-slate-600 to-slate-700 flex items-center justify-center" style="display:none;">
                        <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"></path>
                        </svg>
                    </div>
                `;
            }
        } else {
            iconDisplay.innerHTML = `
                <div class="w-8 h-8 rounded-lg ${this.getIconBackgroundClass(this.editingIndex.icon)} flex items-center justify-center">
                    <img src="${this.getIconPath(this.editingIndex.icon)}" alt="${this.editingIndex.icon}" class="w-5 h-5">
                </div>
            `;
        }
        
        // Update folders count
        document.getElementById('update-index-folders-count').textContent = `${this.editingIndex.folders?.length || 0} folders`;
    }

    selectIcon(option) {
        const iconType = option.getAttribute('data-icon');
        
        if (iconType === 'custom') {
            // Trigger file input for custom upload
            const customIconInput = document.getElementById('update-custom-icon');
            customIconInput?.click();
            return; // Return early, selection will be handled after successful upload
        }

        // If switching from custom to default icon, clear the file input and reset custom option
        if (this.selectedIcon === 'custom') {
            this.clearCustomUpload();
        }
        
        this.selectedIcon = iconType;
        this.editingIndex.icon = iconType;
        this.hasCustomIconChanged = false; // Reset custom icon change flag
        this.updateIconSelection();
        this.updateMainViewDisplay();
    }

    updateIconSelection() {
        document.querySelectorAll(`#${this.modalId} .icon-option`).forEach(option => {
            const icon = option.getAttribute('data-icon');
            const div = option.querySelector('div');
            if (icon === this.selectedIcon) {
                div.classList.add('border-white');
                div.classList.remove('border-transparent');
            } else {
                div.classList.remove('border-white');
                div.classList.add('border-transparent');
            }
        });
    }

    handleCustomIconUpload(event) {
        const file = event.target.files[0];
        if (file) {
            this.customIconFile = file;
            const reader = new FileReader();
            reader.onload = (e) => {
                const preview = document.getElementById('update-custom-icon-preview');
                const innerDiv = preview.querySelector('div');
                
                innerDiv.innerHTML = `<img src="${e.target.result}" alt="Custom icon" class="w-full h-full object-contain">`;
                
                preview.classList.remove('border-dashed', 'border-gray-400', 'bg-gray-700', 'border-transparent');
                preview.classList.add('border-solid', 'border-white');
                
                // Now that we have a successful file upload, set custom as selected
                this.selectedIcon = 'custom';
                this.editingIndex.icon = 'custom';
                this.hasCustomIconChanged = true; // Mark that user uploaded a new custom icon
                this.updateIconSelection();
                this.updateMainViewDisplay();
            };
            reader.readAsDataURL(file);
        }
    }

    resetCustomIconPreview() {
        const preview = document.getElementById('update-custom-icon-preview');
        const innerDiv = preview.querySelector('div');
        
        innerDiv.innerHTML = `
            <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"></path>
            </svg>
        `;
        
        preview.classList.remove('border-solid', 'border-white');
        preview.classList.add('border-dashed', 'border-gray-400', 'bg-gray-700', 'border-transparent');
    }

    clearCustomUpload() {
        const customIconInput = document.getElementById('update-custom-icon');
        if (customIconInput) {
            customIconInput.value = '';
        }
        const customOption = document.querySelector(`#${this.modalId} [data-icon="custom"]`);
        if (customOption) {
            const customDiv = customOption.querySelector('#update-custom-icon-preview');
            const innerDiv = customDiv.querySelector('div');
            if (innerDiv) {
                innerDiv.innerHTML = `
                    <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"></path>
                    </svg>
                `;
                customDiv.classList.remove('border-solid', 'border-white');
                customDiv.classList.add('border-dashed', 'border-gray-400', 'bg-gray-700', 'border-transparent');
            }
        }
        this.customIconFile = null;
        this.hasCustomIconChanged = false;
    }

    async selectFolders() {
        try {
            console.log('Add folders clicked - requesting folder selection...');
            const data = await Backend.getFolders();
            if (data.error) {
                throw new Error(data.error);
            }
            if (data.folders && data.folders.length > 0) {
                const newFolders = data.folders.filter(folder => !this.selectedFolders.includes(folder));
                this.selectedFolders.push(...newFolders);
                this.selectedFolders = this.filterRedundantFolders(this.selectedFolders);
                this.editingIndex.folders = [...this.selectedFolders];
                this.updateFoldersList();
                this.updateMainViewDisplay();
                console.log('Added folders:', newFolders);
            } else {
                console.log('No folders selected');
            }
        } catch (error) {
            console.error('Error selecting folders:', error);
            const errorDiv = document.getElementById('update-index-error');
            errorDiv.textContent = error.message || 'Failed to select folders';
            errorDiv.classList.remove('hidden');
        }
    }

    filterRedundantFolders(folders) {
        if (folders.length === 0) {
            return folders;
        }
        const sortedFolders = [...folders].sort((a, b) => a.length - b.length);
        const filtered = [];
        for (const folder of sortedFolders) {
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
        const foldersList = document.getElementById('update-index-selected-folders-list');
        if (!foldersList) return;
        if (this.selectedFolders.length === 0) {
            foldersList.innerHTML = '<p class="text-gray-500 text-sm text-center py-8">No folders selected yet</p>';
            return;
        }
        foldersList.innerHTML = this.selectedFolders.map((folder, index) => `
            <div class="flex items-center justify-between bg-gray-600 rounded-lg p-3">
                <div class="flex items-center space-x-3">
                    <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2z"></path>
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 5a2 2 0 012-2h4a2 2 0 012 2v2H8V5z"></path>
                    </svg>
                    <span class="text-gray-300 text-sm break-all">${escapeHtml(folder)}</span>
                </div>
                <button type="button" class="remove-folder-btn text-red-400 hover:text-red-300 p-1" data-index="${index}">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                    </svg>
                </button>
            </div>
        `).join('');
        foldersList.querySelectorAll('.remove-folder-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const index = parseInt(btn.getAttribute('data-index'));
                this.selectedFolders.splice(index, 1);
                this.editingIndex.folders = [...this.selectedFolders];
                this.updateFoldersList();
                this.updateMainViewDisplay();
            });
        });
    }

    async onSaveClick() {
        try {
            const errorDiv = document.getElementById('update-index-error');
            errorDiv.classList.add('hidden');

            // Collect all changes from the cloned index
            const updatedIndex = {
                id: this.editingIndex.id,
                name: this.editingIndex.name,
                mediaType: this.editingIndex.mediaType, // Don't allow changing media type
                icon: this.editingIndex.icon,
                folders: this.editingIndex.folders || []
            };
            
            // Add custom icon file if present and user uploaded a new one
            if (this.customIconFile && this.hasCustomIconChanged && this.editingIndex.icon === 'custom') {
                // Convert file to base64
                const base64Data = await this.fileToBase64(this.customIconFile);
                updatedIndex.customIconFile = base64Data;
            }
            
            const result = await Backend.updateIndex(updatedIndex);
            
            // Update local config immediately with the result from server
            Config.updateIndex(result.index);
            
            // Hide modal and show success
            this.hideModal();
            
            // Dispatch event to notify parent component
            document.dispatchEvent(new CustomEvent('indexUpdated', {
                detail: { index: result.index }
            }));
            
        } catch (error) {
            console.error('Error updating index:', error);
            const errorDiv = document.getElementById('update-index-error');
            errorDiv.textContent = error.message || 'Failed to update index';
            errorDiv.classList.remove('hidden');
        }
    }

    // Helper function to convert file to base64
    fileToBase64(file) {
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

    getIconBackgroundClass(iconType) {
        const backgrounds = {
            home: 'bg-gradient-to-r from-slate-600 to-slate-700',
            movie: 'bg-gradient-to-r from-blue-600 to-blue-700',
            music: 'bg-gradient-to-r from-green-600 to-green-700',
            image: 'bg-gradient-to-r from-purple-600 to-purple-700'
        };
        return backgrounds[iconType] || 'bg-gradient-to-r from-slate-600 to-slate-700';
    }

    getIconPath(iconType) {
        const icons = {
            home: './images/icon_home.svg',
            movie: './images/icon_movie.svg',
            music: './images/icon_music.svg',
            image: './images/icon_image.svg'
        };
        return icons[iconType] || './images/icon_home.svg';
    }
}
