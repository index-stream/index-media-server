const hostname = window.location.hostname;
const port = window.location.port;
export const API_URL = `https://${hostname}:${port}/api`;

const urlParams = new URLSearchParams(window.location.search);
const dev = urlParams.get('dev');
export const REDIRECT_URL = (!dev)
    ? 'https://indexstream.org'
    : ((dev == 'local') ? 'https://localhost:8000' : 'https://dev.indexstream.org');