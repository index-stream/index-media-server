import Backend from './clients/Backend.js';
import { REDIRECT_URL } from './constants/constants.js';

class Main {
    constructor() {
        this.setupEventListeners();
        this.init();
    }

    async init() {
        const urlParams = new URLSearchParams(window.location.search);
        const hasOpener = urlParams.get('hasOpener');

        // Check if we're opened by another window/tab
        if (window.opener) {
            // We're opened by another page, handle the connection flow
            await this.handleConnectionFlow();
        } else if(hasOpener) {
            window.close();
        } else {
            // No opener, redirect to localhost
            this.redirectToStreamingPage();
        }
    }

    async handleConnectionFlow() {
        try {
            this.updateStatusMessage('Attempting to connect to server...');

            await new Promise(resolve => setTimeout(resolve, 1000));
            // Check if we have a token in localStorage
            const token = localStorage.getItem('token');
            if (token && await Backend.checkToken(token)) {
                this.handleSuccessfulConnection({ token });
                return;
            }
        } catch (error) {
            console.log('Token not found');
        }

        try {
            // Try to login without password first
            const response = await Backend.login();
            if (response.success) {
                localStorage.setItem('token', response.token);
                this.handleSuccessfulConnection(response);
            } else {
                throw new Error('Login failed');
            }
        } catch (error) {
            console.log('Password required for login');
            this.showPage('password');
            this.setupPasswordForm();
        }
    }

    isUnauthorizedError(error) {
        try {
            const errorData = JSON.parse(error.message);
            return errorData.status === 401;
        } catch {
            return false;
        }
    }

    async handleSuccessfulConnection(response) {
        // Show success page
        this.showPage('success');
        
        // Communicate back to opener if available
        if (window.opener && !window.opener.closed) {
            try {
                await new Promise(resolve => setTimeout(resolve, 2000));
                window.opener.postMessage({
                    type: 'AUTH_SUCCESS',
                    token: response.token,
                    timestamp: Date.now()
                }, '*');
            } catch (e) {
                console.warn('Could not communicate with opener:', e);
            }
        } else {
            // Redirect after a short delay
            setTimeout(() => {
                this.redirectToStreamingPage(response.token);
            }, 2000);
        }
    }

    showPage(pageId) {
        // Hide all pages
        document.getElementById('loading-page').classList.add('hidden');
        document.getElementById('password-page').classList.add('hidden');
        document.getElementById('success-page').classList.add('hidden');

        // Show the requested page
        document.getElementById(`${pageId}-page`).classList.remove('hidden');
    }

    updateStatusMessage(message) {
        const statusElement = document.getElementById('status-message');
        if (statusElement) {
            statusElement.textContent = message;
        }
    }

    setupPasswordForm() {
        this.hideError();
        
        // Focus password input
        const passwordInput = document.getElementById('password');
        passwordInput.focus();
    }

    setupEventListeners() {
        // Password form submission
        const passwordForm = document.getElementById('password-form');
        passwordForm?.addEventListener('submit', (e) => this.handlePasswordSubmit(e));
    }

    handlePasswordSubmit(event) {
        event.preventDefault();
        
        // Prevent multiple submissions
        const submitBtn = document.getElementById('submit-btn');
        if (submitBtn.disabled) {
            return;
        }
        
        const formData = new FormData(event.target);
        const password = formData.get('password');
        
        if (!password || password.trim() === '') {
            this.showError('Please enter a password');
            return;
        }

        this.handlePasswordSubmission(password.trim());
    }

    async handlePasswordSubmission(password) {
        const passwordInput = document.getElementById('password');
        const submitBtn = document.getElementById('submit-btn');

        // Disable button during submission
        submitBtn.disabled = true;
        submitBtn.textContent = 'Connecting...';
        this.hideError();

        try {
            this.updateStatusMessage('Authenticating with server...');
            
            const response = await Backend.login(password);
            
            if (response.success) {
                localStorage.setItem('token', response.token);
                // Success! Store auth info and show success page
                this.handleSuccessfulConnection(response);
            } else {
                // Login failed, show error
                this.showError('Login failed. Please check your password and try again.');
            }
        } catch (error) {
            console.error('Password login failed:', error);
            
            // Check if it's a 401 (unauthorized) error
            if (this.isUnauthorizedError(error)) {
                this.showError('Invalid password. Please try again.');
            } else {
                // Other error, show generic error
                this.showError('Connection failed. Please check your network connection and try again.');
            }
        } finally {
            // Re-enable button
            submitBtn.disabled = false;
            submitBtn.textContent = 'Connect to Server';
        }
    }

    showError(message) {
        const errorElement = document.getElementById('error-message');
        if (errorElement) {
            errorElement.textContent = message;
            errorElement.classList.remove('hidden');
        }
    }

    hideError() {
        const errorElement = document.getElementById('error-message');
        if (errorElement) {
            errorElement.classList.add('hidden');
        }
    }

    redirectToStreamingPage(token) {
        window.location.href = (token) ? `${REDIRECT_URL}?token=${token}` : REDIRECT_URL;
    }
}

const main = new Main();
export default main;