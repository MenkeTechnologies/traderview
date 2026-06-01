// Fast path to the backend — runs before the heavy app.js view graph loads.

import { initApi, api, ApiError } from './api.js';
import { bootI18n, t } from './i18n.js';
import { showAuthScreen } from './auth.js';

function showBootError(err) {
    const appEl = document.getElementById('app');
    const msg = err && err.message ? err.message : String(err);
    if (appEl) {
        appEl.innerHTML = `<p class="boot">${t('boot.failed_connect', { err: msg })}</p>`;
    }
}

async function connectBackend() {
    try {
        await bootI18n('en');
        await initApi();
        const spinnerText = document.querySelector('.tv-spinner-text');
        if (spinnerText) spinnerText.textContent = 'loading application…';

        const cfg = await api.config().catch(() => ({ mode: 'desktop' }));
        let me;
        let accounts;
        try {
            me = await api.me();
            accounts = await api.accounts();
        } catch (e) {
            if (e instanceof ApiError && e.status === 401 && cfg.mode === 'web') {
                showAuthScreen();
                return;
            }
            throw e;
        }

        const { mountApp } = await import('./app.js');
        await mountApp({ cfg, me, accounts });
    } catch (e) {
        showBootError(e);
    }
}

document.addEventListener('DOMContentLoaded', () => { void connectBackend(); });
