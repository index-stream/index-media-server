import { API_URL } from '../constants/constants.js';

class Backend {
    constructor() {}

    async login(password = null) {
        return this._post('/login', { password });
    }

    async checkToken(token) {
        return this._get('/token', { token });
    }

    async ping() {
        return this._get('/ping');
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

    async _post(path, request, apiUrl = API_URL) {
        const response = await fetch(apiUrl + path, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(request),
        });

        if (!response.ok) {
            await this._handleError(response);
        }
        return await response.json();
    }
}

let backend = new Backend();
export default backend;
