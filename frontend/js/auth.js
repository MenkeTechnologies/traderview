// Auth screen — shown only when /api/auth/me returns 401 in web mode.
// Desktop mode auto-logs in via the token injected by Tauri.

import { api, setToken } from './api.js';
import { t } from './i18n.js';

let currentMode = 'login';

export function showAuthScreen() {
    const auth = document.getElementById('auth-screen');
    const app  = document.getElementById('app');
    if (auth) auth.classList.remove('hidden');
    if (app)  app.classList.add('hidden');
    bindOnce();
}

export function hideAuthScreen() {
    const auth = document.getElementById('auth-screen');
    const app  = document.getElementById('app');
    if (auth) auth.classList.add('hidden');
    if (app)  app.classList.remove('hidden');
}

function bindOnce() {
    const root = document.getElementById('auth-screen');
    if (!root || root.dataset.bound === '1') return;
    root.dataset.bound = '1';

    root.querySelectorAll('.auth-tab').forEach(btn => {
        btn.addEventListener('click', () => {
            currentMode = btn.dataset.auth;
            root.querySelectorAll('.auth-tab').forEach(b => b.classList.toggle('active', b === btn));
            root.querySelectorAll('.register-only').forEach(el => el.classList.toggle('hidden', currentMode !== 'register'));
        });
    });

    const form = document.getElementById('auth-form');
    if (!form) return;
    form.addEventListener('submit', async (ev) => {
        ev.preventDefault();
        const fd = new FormData(ev.target);
        const email = fd.get('email');
        const password = fd.get('password');
        const display_name = fd.get('display_name') || '';
        const err = document.getElementById('auth-error');
        if (err) err.textContent = '';
        try {
            const res = currentMode === 'login'
                ? await api.login(email, password)
                : await api.register(email, password, display_name);
            setToken(res.token);
            hideAuthScreen();
            window.dispatchEvent(new CustomEvent('tv:authed'));
        } catch (e) {
            if (err) err.textContent = e.message || t('auth.error.fallback');
        }
    });
}
