// Auth screen — shown only when /api/auth/me returns 401 in web mode.
// Desktop mode auto-logs in via the token injected by Tauri.

import { api, setToken } from './api.js';

let currentMode = 'login';

export function showAuthScreen() {
    document.getElementById('auth-screen').classList.remove('hidden');
    document.getElementById('app').classList.add('hidden');
    bindOnce();
}

export function hideAuthScreen() {
    document.getElementById('auth-screen').classList.add('hidden');
    document.getElementById('app').classList.remove('hidden');
}

function bindOnce() {
    const root = document.getElementById('auth-screen');
    if (root.dataset.bound === '1') return;
    root.dataset.bound = '1';

    root.querySelectorAll('.auth-tab').forEach(btn => {
        btn.addEventListener('click', () => {
            currentMode = btn.dataset.auth;
            root.querySelectorAll('.auth-tab').forEach(b => b.classList.toggle('active', b === btn));
            root.querySelectorAll('.register-only').forEach(el => el.classList.toggle('hidden', currentMode !== 'register'));
        });
    });

    document.getElementById('auth-form').addEventListener('submit', async (ev) => {
        ev.preventDefault();
        const fd = new FormData(ev.target);
        const email = fd.get('email');
        const password = fd.get('password');
        const display_name = fd.get('display_name') || '';
        const err = document.getElementById('auth-error');
        err.textContent = '';
        try {
            const res = currentMode === 'login'
                ? await api.login(email, password)
                : await api.register(email, password, display_name);
            setToken(res.token);
            hideAuthScreen();
            window.dispatchEvent(new CustomEvent('tv:authed'));
        } catch (e) {
            err.textContent = e.message || 'failed';
        }
    });
}
