import { API_URL } from '../constants.js';

class Backend {
    constructor() {
        this._token = new URLSearchParams(window.location.search).get('token');
    }

    async getFolders() {
        return this._postAuthenticated('/select-folders');
    }

    async getConfiguration() {
        return this._getAuthenticated('/config');
    }

    async saveConfiguration(config) {
        return this._postAuthenticated('/config', config);
    }

    async getConnectCode() {
        return this._getAuthenticated('/connect-code');
    }

    async updateServerName(name) {
        return this._putAuthenticated('/server/name', { name });
    }

    async updateServerPassword(password) {
        return this._putAuthenticated('/server/password', { password });
    }

    async getIndexes() {
        return this._getAuthenticated('/indexes');
    }

    async createLocalIndex(index) {
        return this._postAuthenticated('/index/local', index);
    }

    async updateIndex(index) {
        return this._putAuthenticated('/index/' + index.id, index);
    }

    async deleteIndex(indexId) {
        return this._deleteAuthenticated('/index/' + indexId);
    }

    async getProfiles() {
        return this._getAuthenticated('/profiles');
    }

    async createProfile(profile) {
        return this._postAuthenticated('/profile', profile);
    }

    async updateProfile(profile) {
        return this._putAuthenticated('/profile/' + profile.id, profile);
    }

    async deleteProfile(profileId) {
        return this._deleteAuthenticated('/profile/' + profileId);
    }

    async _handleError(response) {
        let errorData = {
            status: response.status,
        }
        if(response.status === 401) {
            // Redirect to unauthorized page
            window.location.href = './unauthorized.html';
            return;
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

    async _getAuthenticated(path, params, apiUrl = API_URL) {
        const queryString = params ? `?${new URLSearchParams(params)}` : '';
        const response = await fetch(apiUrl + path + queryString, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${this._token}`
            },
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
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(request),
        });

        if (!response.ok) {
            await this._handleError(response);
        }
        return await response.json();
    }

    async _postAuthenticated(path, request, apiUrl = API_URL) {
        const response = await fetch(apiUrl + path, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${this._token}`
            },
            body: JSON.stringify(request),
        });

        if (!response.ok) {
            await this._handleError(response);
        }
        return await response.json();
    }

    async _put(path, request, apiUrl = API_URL) {
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

    async _putAuthenticated(path, request, apiUrl = API_URL) {
        const response = await fetch(apiUrl + path, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${this._token}`
            },
            body: JSON.stringify(request),
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

        const response = await fetch(apiUrl + path + queryString, {
            method: 'DELETE',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${this._token}`
            },
        });

        if (!response.ok) {
            await this._handleError(response);
        }
        return await response.json();
    }
}

let backend = new Backend();
export default backend;