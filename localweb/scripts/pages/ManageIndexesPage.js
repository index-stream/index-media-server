import Page from './Page.js';
import { PAGES, API_URL } from '../constants.js';
import Backend from '../clients/Backend.js';
import Indexes from '../models/Indexes.js';
import AddLocalIndexComponent from '../components/AddLocalIndexComponent.js';
import UpdateLocalIndexComponent from '../components/UpdateLocalIndexComponent.js';

export default class ManageIndexesPage extends Page {
    constructor() {
        super(PAGES.MANAGE_INDEXES);
        this.addIndexComponent = new AddLocalIndexComponent();
        this.updateIndexComponent = new UpdateLocalIndexComponent();
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Back button
        const backBtn = document.getElementById('indexes-back-btn');
        backBtn?.addEventListener('click', () => this.onBackClick());

        // Add Local Index button
        const addLocalIndexBtn = document.getElementById('manage-add-local-index-btn');
        addLocalIndexBtn?.addEventListener('click', () => this.addIndexComponent.showModal());

        // Add Plugin Index button
        const addPluginIndexBtn = document.getElementById('manage-add-plugin-index-btn');
        addPluginIndexBtn?.addEventListener('click', () => this.showPluginComingSoonModal());

        // Plugin coming soon modal close button
        const pluginComingSoonCloseBtn = document.getElementById('plugin-coming-soon-close-btn');
        pluginComingSoonCloseBtn?.addEventListener('click', () => this.hidePluginComingSoonModal());

        // Listen for component events
        document.addEventListener('indexCreated', (e) => {
            this.onIndexCreated(e.detail.index);
        });

        document.addEventListener('indexUpdated', (e) => {
            this.onIndexUpdated(e.detail.index);
        });
    }

    onBackClick() {
        // Navigate back to home page
        document.dispatchEvent(new CustomEvent('navigate', {
            detail: { page: PAGES.HOME }
        }));
    }

    onIndexCreated(index) {
        // Refresh the indexes list
        this.renderIndexes(Indexes.getIndexes());
        
        // Show success message
        const successDiv = document.getElementById('manage-indexes-success');
        successDiv.textContent = `Index "${index.name}" created successfully!`;
        successDiv.classList.remove('hidden');
        
        // Hide success message after 3 seconds
        setTimeout(() => {
            successDiv.classList.add('hidden');
        }, 3000);
    }

    onIndexUpdated(index) {
        // Refresh the indexes list
        this.renderIndexes(Indexes.getIndexes());
        
        // Show success message
        const successDiv = document.getElementById('manage-indexes-success');
        successDiv.textContent = `Index "${index.name}" updated successfully!`;
        successDiv.classList.remove('hidden');
        
        // Hide success message after 3 seconds
        setTimeout(() => {
            successDiv.classList.add('hidden');
        }, 3000);
    }




    showPluginComingSoonModal() {
        const modal = document.getElementById('plugin-coming-soon-modal');
        modal.classList.remove('hidden');
    }

    hidePluginComingSoonModal() {
        const modal = document.getElementById('plugin-coming-soon-modal');
        modal.classList.add('hidden');
    }





    async deleteIndex(indexId) {
        const errorDiv = document.getElementById('manage-indexes-error');
        const successDiv = document.getElementById('manage-indexes-success');
        
        try {
            errorDiv.classList.add('hidden');
            successDiv.classList.add('hidden');

            // Confirm deletion
            if (!confirm('Are you sure you want to delete this index? This action cannot be undone.')) {
                return;
            }

            await Backend.deleteIndex(indexId);

            // Update local config immediately
            Indexes.removeIndex(indexId);

            // Refresh indexes list
            await this.loadIndexes();

            // Show success message
            successDiv.textContent = 'Index deleted successfully!';
            successDiv.classList.remove('hidden');
            
            // Hide success message after a delay
            setTimeout(() => {
                successDiv.classList.add('hidden');
            }, 2000);

        } catch (error) {
            console.error('Error deleting index:', error);
            errorDiv.textContent = error.message || 'Failed to delete index';
            errorDiv.classList.remove('hidden');
        }
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

    getMediaTypeIcon(mediaType) {
        const icons = {
            videos: './images/icon_movie.svg',
            audio: './images/icon_music.svg',
            photos: './images/icon_image.svg'
        };
        return icons[mediaType] || './images/icon_home.svg';
    }

    getIconPath(icon) {
        if (icon === 'custom') {
            return null; // Handle custom icons separately
        }
        return `./images/icon_${icon}.svg`;
    }

    renderIndexes(indexes) {
        const indexesGrid = document.getElementById('indexes-grid');
        const emptyState = document.getElementById('indexes-empty-state');
        
        if (!indexes || indexes.length === 0) {
            indexesGrid.innerHTML = '';
            emptyState.classList.remove('hidden');
            return;
        }

        emptyState.classList.add('hidden');
        
        indexesGrid.innerHTML = indexes.map(index => `
            <div class="bg-gray-800 border border-gray-600 rounded-lg p-4 hover:border-gray-500 transition-colors">
                <div class="flex items-center justify-between mb-3">
                    <div class="flex items-center space-x-3 overflow-hidden">
                        <div class="w-8 h-8 flex items-center justify-center flex-shrink-0">
                            ${index.icon === 'custom'
                                ? `<img src="${API_URL}/index/${index.id}/icon" alt="Custom icon" class="w-8 h-8 object-contain" onerror="this.style.display='none'; this.nextElementSibling.style.display='flex';">
                                <div class="w-8 h-8 rounded-lg bg-gradient-to-r from-slate-600 to-slate-700 flex items-center justify-center" style="display:none;">
                                    <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"></path>
                                    </svg>
                                </div>`
                                : `<div class="w-8 h-8 rounded-lg ${this.getIconBackgroundClass(index.icon)} flex items-center justify-center">
                                    <img src="${this.getIconPath(index.icon)}" alt="${index.icon}" class="w-5 h-5">
                                </div>`
                            }
                        </div>
                        <div class="min-w-0 flex-1 text-left">
                            <h3 class="text-lg font-semibold text-white truncate" title="${index.name}">${index.name}</h3>
                        </div>
                    </div>
                    <div class="flex space-x-2">
                        <button class="edit-index-btn text-blue-400 hover:text-blue-300 p-1" data-index-id="${index.id}">
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
                            </svg>
                        </button>
                        <button class="delete-index-btn text-red-400 hover:text-red-300 p-1" data-index-id="${index.id}">
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                            </svg>
                        </button>
                    </div>
                </div>
                <div class="flex items-center justify-between text-sm text-gray-400">
                    <div class="flex items-center space-x-2">
                        <img src="${this.getMediaTypeIcon(index.type)}" alt="${index.type}" class="w-4 h-4">
                        <span>${index.type || 'Unknown'}</span>
                    </div>
                    <span class="text-xs">Local Index</span>
                </div>
            </div>
        `).join('');

        // Add event listeners to the dynamically created buttons
        indexesGrid.querySelectorAll('.edit-index-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const indexId = btn.getAttribute('data-index-id');
                const index = indexes.find(i => i.id === indexId);
                if (index) {
                    this.updateIndexComponent.showModal(index);
                }
            });
        });

        indexesGrid.querySelectorAll('.delete-index-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const indexId = btn.getAttribute('data-index-id');
                this.deleteIndex(indexId);
            });
        });
    }

    async loadIndexes() {
        try {
            // Use config from model instead of API call
            if (!Indexes.isLoaded()) {
                throw new Error('Indexes not loaded');
            }
            
            const indexes = Indexes.getIndexes();
            this.renderIndexes(indexes);
            
        } catch (error) {
            console.error('Error loading indexes:', error);
            const errorDiv = document.getElementById('manage-indexes-error');
            errorDiv.textContent = 'Failed to load indexes';
            errorDiv.classList.remove('hidden');
            
            // Show empty state
            const emptyState = document.getElementById('indexes-empty-state');
            emptyState.classList.remove('hidden');
        }
    }

    async onShow() {
        try {
            // Hide any previous error/success messages
            const errorDiv = document.getElementById('manage-indexes-error');
            const successDiv = document.getElementById('manage-indexes-success');
            errorDiv.classList.add('hidden');
            successDiv.classList.add('hidden');
            
            // Load and display indexes
            await this.loadIndexes();
            
        } catch (error) {
            console.error('Error showing indexes page:', error);
        }
    }
}
