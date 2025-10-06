import Page from './Page.js';
import { PAGES, STREAMING_URL } from '../constants.js';
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

        // Server settings button
        const serverSettingsBtn = document.getElementById('server-settings-btn');
        serverSettingsBtn?.addEventListener('click', () => this.onServerSettingsClick());

        // Manage profiles button
        const manageProfilesBtn = document.getElementById('manage-profiles-btn');
        manageProfilesBtn?.addEventListener('click', () => this.onManageProfilesClick());
    }

    onStartStreamingLinkClick() {
        console.log('Start streaming link clicked - TODO: Implement streaming functionality');
        window.location.href = STREAMING_URL + '/app?connectCode=' + this._connectCode;
    }

    onManageIndexesClick() {
        console.log('Manage indexes clicked - TODO: Implement index management');
        // TODO: Navigate to index management page or show modal
    }

    onServerSettingsClick() {
        document.dispatchEvent(new CustomEvent('navigate', {
            detail: { page: PAGES.SERVER_SETTINGS }
        }));
    }

    onManageProfilesClick() {
        document.dispatchEvent(new CustomEvent('navigate', {
            detail: { page: PAGES.MANAGE_PROFILES }
        }));
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
        const response = await Backend.getConnectCode();
        this._connectCode = response.connectCode;
        this.updateConnectCode(this._connectCode);
    }
}
