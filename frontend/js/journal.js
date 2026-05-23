import { api } from './api.js';

export async function renderJournalView(mount) {
    const today = new Date().toISOString().slice(0, 10);
    mount.innerHTML = `
        <div style="margin-bottom: 12px;">
            <input type="date" id="journal-date" value="${today}">
        </div>
        <div id="journal-list"></div>
    `;
    const dateInput = document.getElementById('journal-date');
    const list = document.getElementById('journal-list');
    const load = async () => {
        try {
            const entries = await api.journalForDay(dateInput.value);
            if (!entries.length) {
                list.innerHTML = '<p class="boot">No journal entries for this day.</p>';
                return;
            }
            list.innerHTML = entries.map(e =>
                `<div class="card"><div class="label">${e.created_at.slice(11, 16)}</div>
                 <pre style="white-space: pre-wrap; margin: 0;">${escapeHtml(e.body_md)}</pre></div>`
            ).join('');
        } catch (err) {
            list.textContent = err.message;
        }
    };
    dateInput.addEventListener('change', load);
    load();
}

function escapeHtml(s) {
    return s.replace(/[&<>"']/g, c => ({
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;'
    }[c]));
}
