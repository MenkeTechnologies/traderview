import { api } from '../api.js';
import { fmt, fmtDateTime, esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

export async function renderImportView(mount, state) {
    const tok = currentViewToken();
    if (!state.accountId) {
        mount.innerHTML = '<p data-i18n="view.import.hint.create_an_account_first_accounts_tab" class="boot">Create an account first (Accounts tab).</p>';
        return;
    }
    const [sources, history] = await Promise.all([
        api.importSources(),
        api.importList(state.accountId),
    ]);
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.import.h1.import" class="view-title">// IMPORT</h1>
        <div class="chart-panel">
            <h2 data-i18n="view.import.h2.new_import">New import</h2>
            <div class="import-form">
                <label><span data-i18n="view.import.label.broker">Broker</span>
                    <select id="source">
                        ${sources.sources.map(s => `<option value="${s}">${esc(s)}</option>`).join('')}
                    </select>
                </label>
                <div class="dropzone" id="drop" data-i18n="view.import.dropzone">Drop CSV here, or click to pick.</div>
                <input type="file" id="file" accept=".csv,text/csv" hidden>
                <button data-i18n="view.import.btn.upload" class="primary" id="go">Upload</button>
            </div>
            <pre id="import-result" class="result"></pre>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.import.h2.history">History</h2>
            ${history.length ? `
                <table class="trades">
                    <thead><tr><th data-i18n="view.import.th.when">When</th><th data-i18n="view.import.th.source">Source</th><th data-i18n="view.import.th.filename">Filename</th>
                    <th data-i18n="view.import.th.rows">Rows</th><th data-i18n="view.import.th.sha256">SHA256</th></tr></thead>
                    <tbody>${history.map(h => `
                        <tr><td>${fmtDateTime(h.imported_at)}</td>
                        <td>${esc(h.source)}</td>
                        <td>${esc(h.filename)}</td>
                        <td>${h.row_count}</td>
                        <td class="muted">${esc(h.sha256.slice(0, 8))}…</td></tr>
                    `).join('')}</tbody></table>` : '<p data-i18n="view.import.hint.no_imports_yet" class="muted">No imports yet.</p>'}
        </div>
    `;

    const drop = mount.querySelector('#drop');
    const fileInput = mount.querySelector('#file');
    drop.addEventListener('click', () => fileInput.click());
    drop.addEventListener('dragover', (e) => { e.preventDefault(); drop.classList.add('dragover'); });
    drop.addEventListener('dragleave', () => drop.classList.remove('dragover'));
    drop.addEventListener('drop', (e) => {
        e.preventDefault();
        drop.classList.remove('dragover');
        fileInput.files = e.dataTransfer.files;
        drop.textContent = e.dataTransfer.files[0]?.name || '';
    });
    fileInput.addEventListener('change', () => {
        drop.textContent = fileInput.files[0]?.name || '';
    });

    mount.querySelector('#go').addEventListener('click', async () => {
        const f = fileInput.files[0];
        if (!f) { alert(t('view.import.alert.pick_a_file')); return; }
        const src = mount.querySelector('#source').value;
        try {
            const r = await api.upload(state.accountId, src, f);
            if (!viewIsCurrent(tok)) return;
            const out = mount.querySelector('#import-result');
            if (out) out.textContent =
                `parsed=${r.parsed} inserted=${r.inserted} duplicates=${r.duplicates} trades=${r.trades_rolled}`;
            renderImportView(mount, state);
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const out = mount.querySelector('#import-result');
            if (out) out.textContent = t('common.error', { err: e.message });
        }
        void fmt;
    });
}
