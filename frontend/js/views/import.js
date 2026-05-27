import { api } from '../api.js';
import { fmt, fmtDateTime, esc } from '../util.js';

export async function renderImportView(mount, state) {
    if (!state.accountId) {
        mount.innerHTML = '<p class="boot">Create an account first (Accounts tab).</p>';
        return;
    }
    const [sources, history] = await Promise.all([
        api.importSources(),
        api.importList(state.accountId),
    ]);
    mount.innerHTML = `
        <h1 class="view-title">// IMPORT</h1>
        <div class="chart-panel">
            <h2>New import</h2>
            <div class="import-form">
                <label>Broker
                    <select id="source">
                        ${sources.sources.map(s => `<option value="${s}">${esc(s)}</option>`).join('')}
                    </select>
                </label>
                <div class="dropzone" id="drop">Drop CSV here, or click to pick.</div>
                <input type="file" id="file" accept=".csv,text/csv" hidden>
                <button class="primary" id="go">Upload</button>
            </div>
            <pre id="import-result" class="result"></pre>
        </div>

        <div class="chart-panel">
            <h2>History</h2>
            ${history.length ? `
                <table class="trades">
                    <thead><tr><th>When</th><th>Source</th><th>Filename</th>
                    <th>Rows</th><th>SHA256</th></tr></thead>
                    <tbody>${history.map(h => `
                        <tr><td>${fmtDateTime(h.imported_at)}</td>
                        <td>${esc(h.source)}</td>
                        <td>${esc(h.filename)}</td>
                        <td>${h.row_count}</td>
                        <td class="muted">${esc(h.sha256.slice(0, 8))}…</td></tr>
                    `).join('')}</tbody></table>` : '<p class="muted">No imports yet.</p>'}
        </div>
    `;

    const drop = document.getElementById('drop');
    const fileInput = document.getElementById('file');
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

    document.getElementById('go').addEventListener('click', async () => {
        const f = fileInput.files[0];
        if (!f) { alert('pick a file'); return; }
        const src = document.getElementById('source').value;
        try {
            const r = await api.upload(state.accountId, src, f);
            document.getElementById('import-result').textContent =
                `parsed=${r.parsed} inserted=${r.inserted} duplicates=${r.duplicates} trades=${r.trades_rolled}`;
            renderImportView(mount, state);
        } catch (e) {
            document.getElementById('import-result').textContent = 'Error: ' + e.message;
        }
        void fmt;
    });
}
