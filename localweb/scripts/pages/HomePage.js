import Page from './Page.js';
import { PAGES } from '../constants.js';
import Backend from '../clients/Backend.js';

export default class HomePage extends Page {
    constructor() {
        super(PAGES.HOME);
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Start streaming link
        const startStreamingLink = document.getElementById('start-streaming-link');
        startStreamingLink?.addEventListener('click', () => this.onStartStreamingLinkClick());

        // Manage indexes button
        const manageIndexesBtn = document.getElementById('manage-indexes-btn');
        manageIndexesBtn?.addEventListener('click', () => this.onManageIndexesClick());

        // Manage password button
        const managePasswordBtn = document.getElementById('manage-password-btn');
        managePasswordBtn?.addEventListener('click', () => this.onManagePasswordClick());

        // Manage profiles button
        const manageProfilesBtn = document.getElementById('manage-profiles-btn');
        manageProfilesBtn?.addEventListener('click', () => this.onManageProfilesClick());
    }

    onStartStreamingLinkClick() {
        console.log('Start streaming link clicked - TODO: Implement streaming functionality');
        // TODO: Implement actual streaming start logic
    }

    onManageIndexesClick() {
        console.log('Manage indexes clicked - TODO: Implement index management');
        // TODO: Navigate to index management page or show modal
    }

    onManagePasswordClick() {
        console.log('Manage password clicked - TODO: Implement password management');
        // TODO: Show password management modal or navigate to password setup
    }

    onManageProfilesClick() {
        console.log('Manage profiles clicked - TODO: Implement profile management');
        // TODO: Navigate to profile management page or show modal
    }

    // Method to update the connect code display
    updateConnectCode(code) {
        const connectCodeElement = document.getElementById('connect-code');
        if (connectCodeElement) {
            connectCodeElement.textContent = code;
        }
    }

    // Method to show/hide the home page
    async onShow() {
        const connectCode = await Backend.getConnectCode();
        this.updateConnectCode(connectCode.connectCode);
    }
}
