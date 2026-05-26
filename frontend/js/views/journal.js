import { api } from '../api.js';
import { fmtDateTime, md, esc } from '../util.js';

export async function renderJournalView(mount, _state, day) {
    if (!day) day = new Date().toISOString().slice(0, 10);
    const entries = await api.journalForDay(day);
    mount.innerHTML = `
        <h1 class="view-title">// JOURNAL · <input type="date" id="journal-day" value="${day}"></h1>
        <div id="entries">${entries.map(e => `
            <div class="journal-entry">
                <div class="meta">${fmtDateTime(e.created_at)} ${e.mood !== null ? `· mood ${e.mood}` : ''}</div>
                <div class="body">${md(e.body_md)}</div>
                <button class="link" data-del="${e.id}">delete</button>
            </div>
        `).join('') || '<p class="muted">No entries for this day yet.</p>'}</div>
        <div class="chart-panel">
            <h2>New entry</h2>
            <select id="mood">
                <option value="">no mood</option>
                <option value="-2">-2 frustrated</option>
                <option value="-1">-1 off</option>
                <option value="0">0 neutral</option>
                <option value="1">+1 focused</option>
                <option value="2">+2 confident</option>
            </select>
            <textarea id="body" placeholder="What happened today? Setups taken / missed? Process notes?"></textarea>
            <button class="primary" id="save">Save</button>
        </div>
    `;
    document.getElementById('journal-day').addEventListener('change', (e) => {
        window.location.hash = `journal/${e.target.value}`;
    });
    document.getElementById('save').addEventListener('click', async () => {
        const body_md = document.getElementById('body').value.trim();
        if (!body_md) return;
        const mood = document.getElementById('mood').value;
        await api.createJournal({
            day,
            body_md,
            mood: mood === '' ? null : Number(mood),
        });
        renderJournalView(mount, _state, day);
    });
    document.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteJournal(b.dataset.del);
            renderJournalView(mount, _state, day);
        }));
    void esc;
}
