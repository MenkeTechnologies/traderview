import { api } from '../api.js';
import { fmt, fmtDateTime, esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';

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
                    <select id="source" data-tip="view.import.tip.broker">
                        ${sources.sources.map(s => `<option value="${s}">${esc(s)}</option>`).join('')}
                    </select>
                </label>
                <div class="dropzone" id="drop" data-i18n="view.import.dropzone" data-tip="view.import.tip.dropzone" data-shortcut="import_pick_file">Drop CSV here, or click to pick.</div>
                <input type="file" id="file" accept=".csv,text/csv" hidden>
                <button data-i18n="view.import.btn.upload" data-tip="view.import.tip.upload" data-shortcut="import_upload" class="primary" id="go">Upload</button>
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

        <div class="chart-panel">
            <h2 data-i18n="view.import.h2.rows_chart">Rows per import (chronological)</h2>
            <div id="imp-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.import.h2.broker_chart">Total rows imported per broker</h2>
            <div id="imp-broker-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.import.hint.broker_chart" class="muted small">Cumulative row count per source across all uploads. Reveals which broker dominates your data lake and which sources have been silent.</p>
        </div>
    `;
    renderRowsChart(history);
    renderBrokerChart(history);

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
        if (!f) { showToast(t('view.import.alert.pick_a_file'), { level: 'warning' }); return; }
        const src = mount.querySelector('#source').value;
        try {
            const r = await api.upload(state.accountId, src, f);
            if (!viewIsCurrent(tok)) return;
            const out = mount.querySelector('#import-result');
            if (out) out.textContent =
                `parsed=${r.parsed} inserted=${r.inserted} duplicates=${r.duplicates} trades=${r.trades_rolled}`;
            showToast(t('view.import.toast.uploaded', {
                inserted: r.inserted, duplicates: r.duplicates, trades: r.trades_rolled,
            }), { level: r.inserted > 0 ? 'success' : 'info' });
            renderImportView(mount, state);
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const out = mount.querySelector('#import-result');
            if (out) out.textContent = t('common.error', { err: e.message });
            showToast(t('toast.error.api', { err: e.message }), { level: 'error' });
        }
        void fmt;
    });
}

function renderRowsChart(history) {
    const el = document.getElementById('imp-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (history || [])
        .filter(h => Number.isFinite(Number(h.row_count)))
        .sort((a, b) => new Date(a.imported_at) - new Date(b.imported_at));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.import.empty_chart">${esc(t('view.import.empty_chart'))}</div>`;
        return;
    }
    const labels = valid.map(h => new Date(h.imported_at).toLocaleDateString(undefined, { month: '2-digit', day: '2-digit' }));
    const ys = valid.map(h => Number(h.row_count));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.import.chart.import_idx') },
            { label: t('view.import.chart.rows'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderBrokerChart(history) {
    const el = document.getElementById('imp-broker-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const totals = new Map();
    for (const h of (history || [])) {
        const rows = Number(h.row_count);
        if (!Number.isFinite(rows)) continue;
        const src = h.source || '?';
        totals.set(src, (totals.get(src) || 0) + rows);
    }
    if (totals.size < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.import.empty_broker_chart">${esc(t('view.import.empty_broker_chart'))}</div>`;
        return;
    }
    const pairs = Array.from(totals.entries()).sort((a, b) => b[1] - a[1]);
    const labels = pairs.map(([s]) => s);
    const ys = pairs.map(([, n]) => n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.import.chart.broker_idx') },
            { label: t('view.import.chart.total_rows'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}
