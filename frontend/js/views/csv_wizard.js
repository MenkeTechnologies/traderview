// CSV import wizard — upload arbitrary CSV, drag headers to canonical
// fields, preview the parse, commit through dedupe-aware insert.

import { esc } from '../util.js';

const FIELDS = [
    { key: 'symbol',          required: true,  hint: 'AAPL, MSFT, …' },
    { key: 'side',            required: true,  hint: 'buy / sell / short / cover' },
    { key: 'qty',             required: true,  hint: 'positive number' },
    { key: 'price',           required: true,  hint: 'fill price' },
    { key: 'executed_at',     required: true,  hint: 'RFC3339 / YYYY-MM-DD HH:MM:SS / M/D/Y' },
    { key: 'fee',             required: false, hint: 'optional; defaults 0' },
    { key: 'broker_order_id', required: false, hint: 'optional dedupe key' },
];

let parsedCsv = null;       // ParsePreview from /parse
let rawBytes  = null;       // raw File bytes for /commit
let mapping = {};

export async function renderCsvWizard(mount, state) {
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) { mount.innerHTML = `<p class="boot">No account selected.</p>`; return; }
    mount.innerHTML = `
        <h1 class="view-title">// CSV WIZARD — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p class="muted small">Upload any CSV — Notion exports, Excel sheets, hand-rolled logs.
            The wizard auto-detects headers, you map them to the seven canonical fields below,
            preview the first 20 rows post-mapping, then commit. Re-imports of the same file are
            blocked at the imports.sha256 unique constraint; per-row dedupe relies on
            executions(broker_order_id, executed_at, symbol, side, qty, price).</p>

        <div class="chart-panel">
            <h2>1 — Upload CSV</h2>
            <input type="file" id="cw-file" accept=".csv,text/csv">
            <span id="cw-status" class="muted small" style="margin-left:8px;"></span>
        </div>

        <div id="cw-map"></div>
        <div id="cw-preview"></div>
        <div id="cw-commit"></div>
        <div id="cw-result"></div>
    `;
    document.getElementById('cw-file').addEventListener('change', async (e) => {
        const f = e.target.files[0];
        if (!f) return;
        const status = document.getElementById('cw-status');
        status.textContent = `parsing ${f.name}…`;
        rawBytes = await f.arrayBuffer();
        try {
            const res = await fetch('/api/imports/csv-wizard/parse', {
                method: 'POST',
                headers: tokenHeaders({ 'Content-Type': 'text/csv' }),
                body: rawBytes,
            });
            if (!res.ok) {
                const txt = await res.text();
                throw new Error(txt || res.statusText);
            }
            parsedCsv = await res.json();
            status.textContent = `${parsedCsv.total_rows} rows · ${parsedCsv.headers.length} columns · sha256 ${parsedCsv.sha256.slice(0, 12)}…`;
            mapping = {};
            renderMap();
            renderPreview();
            renderCommit(acct.id);
        } catch (err) {
            status.textContent = 'error: ' + err.message;
        }
    });
}

function tokenHeaders(extra = {}) {
    const t = localStorage.getItem('tv-token') || '';
    return Object.assign({}, extra, t ? { 'Authorization': `Bearer ${t}` } : {});
}

function renderMap() {
    if (!parsedCsv) return;
    const headers = parsedCsv.headers;
    const opts = (cur) =>
        `<option value="">(unmapped)</option>` +
        headers.map(h => `<option value="${esc(h)}" ${cur === h ? 'selected' : ''}>${esc(h)}</option>`).join('');
    document.getElementById('cw-map').innerHTML = `<div class="chart-panel">
        <h2>2 — Map columns</h2>
        <table class="trades" style="width:auto;">
            <thead><tr><th>Canonical field</th><th>CSV column</th><th>Hint</th></tr></thead>
            <tbody>
            ${FIELDS.map(f => `<tr>
                <td><strong>${esc(f.key)}</strong>${f.required ? ' <span class="neg">*</span>' : ''}</td>
                <td><select data-field="${f.key}">${opts(mapping[f.key] || autoGuess(f.key, headers))}</select></td>
                <td class="small muted">${esc(f.hint)}</td>
            </tr>`).join('')}
            </tbody>
        </table>
        <p class="muted small">Auto-guesses based on header names. Adjust as needed.</p>
    </div>`;
    document.querySelectorAll('#cw-map select[data-field]').forEach(sel => {
        const f = sel.dataset.field;
        if (!mapping[f] && sel.value) mapping[f] = sel.value;
        sel.addEventListener('change', () => {
            mapping[f] = sel.value || null;
            renderPreview();
        });
    });
}

function autoGuess(field, headers) {
    const patterns = {
        symbol:          /^(ticker|symbol|sym|instrument)$/i,
        side:            /^(side|action|direction|buy_sell)$/i,
        qty:             /^(qty|quantity|shares|size|amount)$/i,
        price:           /^(price|fill_price|avg_price|exec_price)$/i,
        executed_at:     /^(time|date|datetime|executed_at|filled_at|timestamp|trade_date)$/i,
        fee:             /^(fee|commission|fees|cost)$/i,
        broker_order_id: /^(order_id|broker_order_id|orderid|order)$/i,
    };
    const re = patterns[field];
    if (!re) return '';
    return headers.find(h => re.test(h)) || '';
}

function renderPreview() {
    if (!parsedCsv) return;
    const headers = parsedCsv.headers;
    const mapped = FIELDS.filter(f => mapping[f.key]).map(f => ({
        field: f.key,
        idx: headers.indexOf(mapping[f.key]),
    }));
    document.getElementById('cw-preview').innerHTML = `<div class="chart-panel">
        <h2>3 — Preview (first 20 rows after mapping)</h2>
        ${mapped.length === 0
            ? '<p class="muted small">Map at least the required fields to preview.</p>'
            : `<table class="trades">
                <thead><tr>${mapped.map(m => `<th>${esc(m.field)}</th>`).join('')}</tr></thead>
                <tbody>
                ${parsedCsv.rows.slice(0, 20).map(row =>
                    `<tr>${mapped.map(m => `<td class="small">${esc(row[m.idx] || '')}</td>`).join('')}</tr>`
                ).join('')}
                </tbody>
            </table>`}
    </div>`;
}

function renderCommit(accountId) {
    document.getElementById('cw-commit').innerHTML = `<div class="chart-panel">
        <h2>4 — Commit</h2>
        <button class="primary" id="cw-go">Insert rows into account</button>
        <span id="cw-go-status" class="muted small" style="margin-left:8px;"></span>
    </div>`;
    document.getElementById('cw-go').addEventListener('click', async () => {
        const missing = FIELDS.filter(f => f.required && !mapping[f.key]);
        if (missing.length) {
            alert('missing required fields: ' + missing.map(f => f.key).join(', '));
            return;
        }
        const status = document.getElementById('cw-go-status');
        status.textContent = 'inserting…';
        try {
            const body = {
                symbol: mapping.symbol, side: mapping.side, qty: mapping.qty,
                price: mapping.price, executed_at: mapping.executed_at,
                fee: mapping.fee || null,
                broker_order_id: mapping.broker_order_id || null,
            };
            const res = await fetch(`/api/imports/csv-wizard/commit/${accountId}`, {
                method: 'POST',
                headers: tokenHeaders({
                    'Content-Type': 'text/csv',
                    'X-CSV-Mapping': JSON.stringify(body),
                }),
                body: rawBytes,
            });
            if (!res.ok) {
                const txt = await res.text();
                throw new Error(txt || res.statusText);
            }
            const r = await res.json();
            renderResult(r);
            status.textContent = '';
        } catch (err) { status.textContent = 'error: ' + err.message; }
    });
}

function renderResult(r) {
    document.getElementById('cw-result').innerHTML = `<div class="chart-panel">
        <h2>5 — Result</h2>
        <div class="cards">
            <div class="card"><div class="label">Inserted</div>
                <div class="value pos">${r.inserted}</div></div>
            <div class="card"><div class="label">Skipped (dedupe)</div>
                <div class="value">${r.skipped_dedupe}</div></div>
            <div class="card"><div class="label">Row failures</div>
                <div class="value ${r.failed_rows.length > 0 ? 'neg' : ''}">${r.failed_rows.length}</div></div>
            <div class="card"><div class="label">Import id</div>
                <div class="value small"><code>${esc(r.import_id)}</code></div></div>
        </div>
        ${r.failed_rows.length === 0 ? '' : `<table class="trades">
            <thead><tr><th>Row #</th><th>Reason</th></tr></thead>
            <tbody>
            ${r.failed_rows.slice(0, 50).map(f => `<tr>
                <td>${f.row_index + 1}</td>
                <td class="small neg">${esc(f.reason)}</td>
            </tr>`).join('')}
            </tbody>
        </table>`}
    </div>`;
}
