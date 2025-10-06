import { API_URL } from '../constants.js';
import Backend from '../clients/Backend.js';
import Config from '../models/Config.js';
import { escapeHtml } from '../utils/text.js';

export default class AddLocalIndexComponent {
    constructor() {
        this.selectedMediaType = 'videos';
        this.selectedIcon = 'home';
        this.selectedFolders = [];
        this.currentStep = 1;
        this.customIconFile = null;
        this.modalId = 'add-index-modal';
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Modal controls
        const cancelBtn = document.getElementById('add-index-cancel-btn');
        cancelBtn?.addEventListener('click', () => this.hideModal());

        // Step navigation buttons
        const continueToIconBtn = document.getElementById('add-index-continue-to-icon-btn');
        continueToIconBtn?.addEventListener('click', () => this.goToStep(2));

        const backToNameBtn = document.getElementById('add-index-back-to-name-btn');
        backToNameBtn?.addEventListener('click', () => this.goToStep(1));

        const continueToFoldersBtn = document.getElementById('add-index-continue-to-folders-btn');
        continueToFoldersBtn?.addEventListener('click', () => this.goToStep(3));

        const backToIconBtn = document.getElementById('add-index-back-to-icon-btn');
        backToIconBtn?.addEventListener('click', () => this.goToStep(2));

        const saveBtn = document.getElementById('add-index-save-btn');
        saveBtn?.addEventListener('click', (e) => {
            e.preventDefault();
            this.onSaveClick();
        });

        // Folder selection
        const addFoldersBtn = document.getElementById('add-index-add-folders-btn');
        addFoldersBtn?.addEventListener('click', () => this.selectFolders());

        // Custom icon file input
        const customIconInput = document.getElementById('add-custom-icon');
        customIconInput?.addEventListener('change', (e) => this.handleCustomIconUpload(e));

        // Media type selection (scoped to add modal)
        document.addEventListener('click', (e) => {
            if (e.target.closest('#add-index-modal .media-type-option')) {
                const option = e.target.closest('.media-type-option');
                this.selectMediaType(option);
            }
        });

        // Icon selection (scoped to add modal)
        document.addEventListener('click', (e) => {
            if (e.target.closest('#add-index-modal .icon-option')) {
                const option = e.target.closest('.icon-option');
                this.selectIcon(option);
            }
        });
    }

    showModal() {
        const modal = document.getElementById(this.modalId);
        modal.classList.remove('hidden');
        
        // Reset to step 1
        this.currentStep = 1;
        this.goToStep(1);
        
        // Clear form
        document.getElementById('add-index-name').value = '';
        this.selectedMediaType = 'videos';
        this.selectedIcon = 'home';
        this.selectedFolders = [];
        this.customIconFile = null;
        this.updateMediaTypeSelection();
        this.updateIconSelection();
        this.updateFoldersList();
        this.resetCustomIconPreview();
        
        document.getElementById('add-index-name').focus();
    }

    hideModal() {
        const modal = document.getElementById(this.modalId);
        modal.classList.add('hidden');
        this.currentStep = 1;
        this.selectedFolders = [];
        this.customIconFile = null;
        this.clearCustomUpload();
    }

    goToStep(step) {
        document.querySelectorAll(`#${this.modalId} [id^="add-index-step-"]`).forEach(stepEl => {
            stepEl.classList.add('hidden');
        });
        const currentStepEl = document.getElementById(`add-index-step-${step}`);
        if (currentStepEl) {
            currentStepEl.classList.remove('hidden');
        }
        this.currentStep = step;
    }

    selectMediaType(optionElement) {
        const type = optionElement.getAttribute('data-type');
        this.selectedMediaType = type;
        this.updateMediaTypeSelection();
    }

    updateMediaTypeSelection() {
        document.querySelectorAll(`#${this.modalId} .media-type-option`).forEach(option => {
            const type = option.getAttribute('data-type');
            const div = option.querySelector('div');
            if (type === this.selectedMediaType) {
                div.classList.add('border-white');
                div.classList.remove('border-transparent');
            } else {
                div.classList.remove('border-white');
                div.classList.add('border-transparent');
            }
        });
    }

    selectIcon(option) {
        const iconType = option.getAttribute('data-icon');
        
        if (iconType === 'custom') {
            // Trigger file input for custom upload
            const customIconInput = document.getElementById('add-custom-icon');
            customIconInput?.click();
            return; // Return early, selection will be handled after successful upload
        }

        // If switching from custom to default icon, clear the file input and reset custom option
        if (this.selectedIcon === 'custom') {
            this.clearCustomUpload();
        }
        
        this.selectedIcon = iconType;
        this.updateIconSelection();
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
                const preview = document.getElementById('add-custom-icon-preview');
                const innerDiv = preview.querySelector('div');
                
                innerDiv.innerHTML = `<img src="${e.target.result}" alt="Custom icon" class="w-full h-full object-contain">`;
                
                preview.classList.remove('border-dashed', 'border-gray-400', 'bg-gray-700', 'border-transparent');
                preview.classList.add('border-solid', 'border-white');
                
                // Now that we have a successful file upload, set custom as selected
                this.selectedIcon = 'custom';
                this.updateIconSelection();
            };
            reader.readAsDataURL(file);
        }
    }

    resetCustomIconPreview() {
        const preview = document.getElementById('add-custom-icon-preview');
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
        const customIconInput = document.getElementById('add-custom-icon');
        if (customIconInput) {
            customIconInput.value = '';
        }
        const customOption = document.querySelector(`#${this.modalId} [data-icon="custom"]`);
        if (customOption) {
            const customDiv = customOption.querySelector('#add-custom-icon-preview');
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
                this.updateFoldersList();
                console.log('Added folders:', newFolders);
            } else {
                console.log('No folders selected');
            }
        } catch (error) {
            console.error('Error selecting folders:', error);
            const errorDiv = document.getElementById('add-index-error');
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
        const foldersList = document.getElementById('add-index-selected-folders-list');
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
                this.updateFoldersList();
            });
        });
    }

    async onSaveClick() {
        const name = document.getElementById('add-index-name').value.trim();
        
        if (!name) {
            const errorDiv = document.getElementById('add-index-error');
            errorDiv.textContent = 'Please enter an index name';
            errorDiv.classList.remove('hidden');
            setTimeout(() => {
                errorDiv.classList.add('hidden');
            }, 3000);
            return;
        }

        if (this.selectedFolders.length === 0) {
            const errorDiv = document.getElementById('add-index-error');
            errorDiv.textContent = 'Please select at least one folder';
            errorDiv.classList.remove('hidden');
            setTimeout(() => {
                errorDiv.classList.add('hidden');
            }, 3000);
            return;
        }

        try {
            const errorDiv = document.getElementById('add-index-error');
            errorDiv.classList.add('hidden');

            // Create new index
            const newIndex = {
                name: name,
                mediaType: this.selectedMediaType,
                icon: this.selectedIcon,
                folders: this.selectedFolders
            };
            
            // Add custom icon file if present
            if (this.customIconFile && this.selectedIcon === 'custom') {
                // Convert file to base64
                const base64Data = await this.fileToBase64(this.customIconFile);
                newIndex.customIconFile = base64Data;
            }
            
            const result = await Backend.createLocalIndex(newIndex);
            
            // Update local config immediately with the returned index (includes ID)
            Config.addIndex(result.index);
            
            // Hide modal and show success
            this.hideModal();
            
            // Dispatch event to notify parent component
            document.dispatchEvent(new CustomEvent('indexCreated', {
                detail: { index: result.index }
            }));
            
        } catch (error) {
            console.error('Error creating index:', error);
            const errorDiv = document.getElementById('add-index-error');
            errorDiv.textContent = error.message || 'Failed to create index';
            errorDiv.classList.remove('hidden');
            setTimeout(() => {
                errorDiv.classList.add('hidden');
            }, 3000);
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
}
