// CSV import wizard — upload arbitrary CSV, drag headers to canonical
// fields, preview the parse, commit through dedupe-aware insert.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t, applyUiI18n } from '../i18n.js';
import { showToast } from '../toast.js';

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
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) { mount.innerHTML = `<p data-i18n="view.csv_wizard.hint.no_account_selected" class="boot">No account selected.</p>`; return; }
    mount.innerHTML = `
        <h1 class="view-title">${esc(t('view.csv_wizard.h1', { broker: acct.broker, name: acct.name }))}</h1>
        <p class="muted small" data-i18n="view.csv_wizard.hint.intro">Upload any CSV — Notion exports, Excel sheets, hand-rolled logs. The wizard auto-detects headers, you map them to the seven canonical fields below, preview the first 20 rows post-mapping, then commit. Re-imports of the same file are blocked at the imports.sha256 unique constraint; per-row dedupe relies on executions(broker_order_id, executed_at, symbol, side, qty, price).</p>

        <div class="chart-panel">
            <h2 data-i18n="view.csv_wizard.h2.1_upload_csv">1 — Upload CSV</h2>
            <input type="file" id="cw-file" accept=".csv,text/csv">
            <span id="cw-status" class="muted small" style="margin-left:8px;"></span>
        </div>

        <div id="cw-map"></div>
        <div id="cw-preview"></div>
        <div id="cw-commit"></div>
        <div id="cw-result"></div>
    `;
    mount.querySelector('#cw-file').addEventListener('change', async (e) => {
        const f = e.target.files[0];
        if (!f) return;
        const status = mount.querySelector('#cw-status');
        if (status) status.textContent = t('view.csv_wizard.status.parsing', { name: f.name });
        rawBytes = await f.arrayBuffer();
        if (!viewIsCurrent(tok)) return;
        try {
            const res = await fetch('/api/imports/csv-wizard/parse', {
                method: 'POST',
                headers: tokenHeaders({ 'Content-Type': 'text/csv' }),
                body: rawBytes,
            });
            if (!viewIsCurrent(tok)) return;
            if (!res.ok) {
                const txt = await res.text();
                throw new Error(txt || res.statusText);
            }
            parsedCsv = await res.json();
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#cw-status');
            if (status2) status2.textContent = t('view.csv_wizard.status.parsed', { rows: parsedCsv.total_rows, cols: parsedCsv.headers.length, sha: parsedCsv.sha256.slice(0, 12) });
            mapping = {};
            renderMap(mount);
            renderPreview(mount);
            renderCommit(acct.id, mount, tok);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#cw-status');
            if (status2) status2.textContent = t('common.error', { err: err.message });
        }
    });
}

function tokenHeaders(extra = {}) {
    const t = localStorage.getItem('tv-token') || '';
    return Object.assign({}, extra, t ? { 'Authorization': `Bearer ${t}` } : {});
}

function renderMap(mount) {
    if (!parsedCsv) return;
    const headers = parsedCsv.headers;
    const opts = (cur) =>
        `<option data-i18n="view.csv_wizard.opt.unmapped" value="">(unmapped)</option>` +
        headers.map(h => `<option value="${esc(h)}" ${cur === h ? 'selected' : ''}>${esc(h)}</option>`).join('');
    const mapEl = mount.querySelector('#cw-map');
    if (!mapEl) return;
    mapEl.innerHTML = `<div class="chart-panel">
        <h2 data-i18n="view.csv_wizard.h2.2_map_columns">2 — Map columns</h2>
        <table class="trades" style="width:auto;">
            <thead><tr><th data-i18n="view.csv_wizard.th.canonical_field">Canonical field</th><th data-i18n="view.csv_wizard.th.csv_column">CSV column</th><th data-i18n="view.csv_wizard.th.hint">Hint</th></tr></thead>
            <tbody>
            ${FIELDS.map(f => `<tr>
                <td><strong>${esc(f.key)}</strong>${f.required ? ' <span class="neg">*</span>' : ''}</td>
                <td><select data-field="${f.key}">${opts(mapping[f.key] || autoGuess(f.key, headers))}</select></td>
                <td class="small muted">${esc(t(`view.csv_wizard.field.${f.key}.hint`))}</td>
            </tr>`).join('')}
            </tbody>
        </table>
        <p data-i18n="view.csv_wizard.hint.auto_guesses_based_on_header_names_adjust_as_neede" class="muted small">Auto-guesses based on header names. Adjust as needed.</p>
    </div>`;
    mapEl.querySelectorAll('select[data-field]').forEach(sel => {
        const f = sel.dataset.field;
        if (!mapping[f] && sel.value) mapping[f] = sel.value;
        sel.addEventListener('change', () => {
            mapping[f] = sel.value || null;
            renderPreview(mount);
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

function renderPreview(mount) {
    if (!parsedCsv) return;
    const headers = parsedCsv.headers;
    const mapped = FIELDS.filter(f => mapping[f.key]).map(f => ({
        field: f.key,
        idx: headers.indexOf(mapping[f.key]),
    }));
    const el = mount.querySelector('#cw-preview');
    if (!el) return;
    el.innerHTML = `<div class="chart-panel">
        <h2 data-i18n="view.csv_wizard.h2.3_preview_first_20_rows_after_mapping">3 — Preview (first 20 rows after mapping)</h2>
        ${mapped.length === 0
            ? '<p data-i18n="view.csv_wizard.hint.map_at_least_the_required_fields_to_preview" class="muted small">Map at least the required fields to preview.</p>'
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

function renderCommit(accountId, mount, tok) {
    const commitEl = mount.querySelector('#cw-commit');
    if (!commitEl) return;
    commitEl.innerHTML = `<div class="chart-panel">
        <h2 data-i18n="view.csv_wizard.h2.4_commit">4 — Commit</h2>
        <button data-i18n="view.csv_wizard.btn.insert_rows_into_account" class="primary" id="cw-go">Insert rows into account</button>
        <span id="cw-go-status" class="muted small" style="margin-left:8px;"></span>
    </div>`;
    mount.querySelector('#cw-go').addEventListener('click', async () => {
        const missing = FIELDS.filter(f => f.required && !mapping[f.key]);
        if (missing.length) {
            showToast(t('view.csv_wizard.alert.missing_fields', { fields: missing.map(f => f.key).join(', ') }), { level: 'warning' });
            return;
        }
        const status = mount.querySelector('#cw-go-status');
        if (status) status.textContent = t('common.status.inserting');
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
            if (!viewIsCurrent(tok)) return;
            if (!res.ok) {
                const txt = await res.text();
                throw new Error(txt || res.statusText);
            }
            const r = await res.json();
            if (!viewIsCurrent(tok)) return;
            renderResult(r, mount);
            const status2 = mount.querySelector('#cw-go-status');
            if (status2) status2.textContent = '';
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#cw-go-status');
            if (status2) status2.textContent = t('common.error', { err: err.message });
        }
    });
}

function renderResult(r, mount) {
    const el = mount.querySelector('#cw-result');
    if (!el) return;
    el.innerHTML = `<div class="chart-panel">
        <h2 data-i18n="view.csv_wizard.h2.5_result">5 — Result</h2>
        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.csv_wizard.card.inserted">Inserted</div>
                <div class="value pos">${r.inserted}</div></div>
            <div class="card"><div class="label" data-i18n="view.csv_wizard.card.skipped_dedupe">Skipped (dedupe)</div>
                <div class="value">${r.skipped_dedupe}</div></div>
            <div class="card"><div class="label" data-i18n="view.csv_wizard.card.row_failures">Row failures</div>
                <div class="value ${r.failed_rows.length > 0 ? 'neg' : ''}">${r.failed_rows.length}</div></div>
            <div class="card"><div class="label" data-i18n="view.csv_wizard.card.import_id">Import id</div>
                <div class="value small"><code>${esc(r.import_id)}</code></div></div>
        </div>
        ${r.failed_rows.length === 0 ? '' : `<table class="trades">
            <thead><tr><th data-i18n="view.csv_wizard.th.row">Row #</th><th data-i18n="view.csv_wizard.th.reason">Reason</th></tr></thead>
            <tbody>
            ${r.failed_rows.slice(0, 50).map(f => `<tr>
                <td>${f.row_index + 1}</td>
                <td class="small neg">${esc(f.reason)}</td>
            </tr>`).join('')}
            </tbody>
        </table>`}
    </div>`;
    try { applyUiI18n(el); } catch (_) {}
}
