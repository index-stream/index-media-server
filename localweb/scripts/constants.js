export const PAGES = {
    INITIAL: 'initial',
    SETUP: 'setup',
    HOME: 'home',
    SERVER_SETTINGS: 'server-settings',
    // Add other page constants as needed
};

export const API_URL = 'http://localhost:1420/api';

const urlParams = new URLSearchParams(window.location.search);
const dev = urlParams.get('dev');
export const STREAMING_URL = (!dev)
    ? 'https://indexstream.org'
    : ((dev == 'local') ? 'http://localhost:8000' : 'https://dev.indexstream.org');