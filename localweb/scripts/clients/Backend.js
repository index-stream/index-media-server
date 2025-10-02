import { API_URL } from '../constants.js';

class Backend {
    constructor() {
        this._authExpiration = Number(localStorage.getItem('authExpiration'));
    }

    async getFolders() {
        return this._post('/select-folders');
    }

    async getConfiguration() {
        return this._get('/config');
    }

    async saveConfiguration(config) {
        return this._post('/config', config);
    }

    async getConnectCode() {
        return this._get('/connect-code');
    }

    async _handleError(response) {
        let errorData = {
            status: response.status,
        }
        try {
            const errorBody = await response.json();
            if (errorBody) {
                errorData.body = errorBody;
            }
        } catch (e) {
            // Response wasn't JSON or didn't have a message field
        }
        throw new Error(JSON.stringify(errorData));
    }

    async _get(path, params, apiUrl = API_URL) {
        const queryString = params ? `?${new URLSearchParams(params)}` : '';
        const response = await fetch(apiUrl + path + queryString, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json'
            },
        });

        if (!response.ok) {
            await this._handleError(response);
        }
        return await response.json();
    }

    isAuthenticated() {
        return !isNaN(this._authExpiration) && this._authExpiration > Date.now();
    }

    async _getAuthenticated(path, params, apiUrl = API_URL) {
        if(!this.isAuthenticated())
            location.href = '/login';
            
        const queryString = params ? `?${new URLSearchParams(params)}` : '';
        const response = await fetch(apiUrl + path + queryString, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json'
            },
            credentials: 'include',
        });
        
        if (!response.ok) {
            await this._handleError(response);
        }
        return await response.json();
    }

    async _post(path, request, apiUrl = API_URL) {
        const response = await fetch(apiUrl + path, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(request),
            credentials: 'include'
        });

        if (!response.ok) {
            await this._handleError(response);
        }
        return await response.json();
    }

    async _postAuthenticated(path, request, apiUrl = API_URL) {
        if(!this.isAuthenticated())
            location.href = '/login';

        const response = await fetch(apiUrl + path, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(request),
            credentials: 'include'
        });

        if (!response.ok) {
            await this._handleError(response);
        }
        return await response.json();
    }

    async _putAuthenticated(path, request, apiUrl = API_URL) {
        if(!this.isAuthenticated())
            location.href = '/login';

        const response = await fetch(apiUrl + path, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(request),
            credentials: 'include'
        });

        if (!response.ok) {
            await this._handleError(response);
        }
        return await response.json();
    }

    async _delete(path, params, apiUrl = API_URL) {
        const queryString = params ? `?${new URLSearchParams(params)}` : '';
        const response = await fetch(apiUrl + path + queryString, {
            method: 'DELETE',
            headers: {
                'Content-Type': 'application/json'
            },
        });

        if (!response.ok) {
            await this._handleError(response);
        }
        return await response.json();
    }

    async _deleteAuthenticated(path, params, apiUrl = API_URL) {
        const queryString = params ? `?${new URLSearchParams(params)}` : '';
        if(!this.isAuthenticated())
            location.href = '/login';

        const response = await fetch(apiUrl + path + queryString, {
            method: 'DELETE',
            headers: {
                'Content-Type': 'application/json'
            },
            credentials: 'include'
        });

        if (!response.ok) {
            await this._handleError(response);
        }
        return await response.json();
    }
}

let backend = new Backend();
export default backend;