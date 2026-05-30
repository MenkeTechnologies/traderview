// Export hub — CSV downloads + printable tax-package HTML.
// All downloads go through the authenticated api client (bearer token) and
// arrive as blobs, then we trigger a synthetic <a download> click.
import { api, apiFetchBlob } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';

export async function renderExports(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) {
        mount.innerHTML = `<p data-i18n="view.exports.hint.no_account_selected_create_one_on_the_accounts_tab" class="boot">No account selected. Create one on the Accounts tab first.</p>`;
        return;
    }
    const now = new Date();
    const years = [];
    for (let y = now.getFullYear(); y >= now.getFullYear() - 5; y--) years.push(y);

    mount.innerHTML = `
        <h1 class="view-title">// EXPORTS — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p data-i18n="view.exports.hint.csv_downloads_use_rfc_4180_quoting_the_tax_package" class="muted small">CSV downloads use RFC 4180 quoting. The tax package opens a
            single-page printable HTML report in a new tab — save as PDF via your browser's
            print dialog (the embedded button auto-hides on print).</p>

        <div class="chart-panel">
            <h2 data-i18n="view.exports.h2.csv_downloads">CSV downloads</h2>
            <table class="trades">
                <thead><tr><th data-i18n="view.exports.th.dataset">Dataset</th><th data-i18n="view.exports.th.description">Description</th><th data-i18n="view.exports.th.action">Action</th></tr></thead>
                <tbody>
                    <tr>
                        <td data-i18n="view.exports.row.executions">Executions</td>
                        <td class="small muted" data-i18n="view.exports.row.executions_desc">Every raw fill: timestamp, side, qty, price, fee, broker order id.</td>
                        <td><button data-i18n="view.exports.btn.download_csv" class="btn" data-action="csv" data-path="/export/executions/${acct.id}.csv" data-name="executions-${acct.id}.csv">Download CSV</button></td>
                    </tr>
                    <tr>
                        <td data-i18n="view.exports.row.trades">Trades</td>
                        <td class="small muted" data-i18n="view.exports.row.trades_desc">Aggregated open→close positions with entry/exit avg, P/L, MFE/MAE, risk.</td>
                        <td><button data-i18n="view.exports.btn.download_csv_2" class="btn" data-action="csv" data-path="/export/trades/${acct.id}.csv" data-name="trades-${acct.id}.csv">Download CSV</button></td>
                    </tr>
                </tbody>
            </table>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.exports.h2.tax_lot_exports">Tax-lot exports</h2>
            <form id="tx-export-form" class="inline-form">
                <label><span data-i18n="view.exports.label.year">Year</span>
                    <select name="year">
                        ${years.map(y => `<option value="${y}" ${y === now.getFullYear() ? 'selected' : ''}>${y}</option>`).join('')}
                    </select>
                </label>
                <label><span data-i18n="view.exports.label.method">Method</span>
                    <select name="method">
                        <option data-i18n="view.exports.opt.fifo" value="fifo" selected>FIFO</option>
                        <option data-i18n="view.exports.opt.lifo" value="lifo">LIFO</option>
                    </select>
                </label>
            </form>
            <table class="trades">
                <thead><tr><th data-i18n="view.exports.th.output">Output</th><th data-i18n="view.exports.th.description_2">Description</th><th data-i18n="view.exports.th.action_2">Action</th></tr></thead>
                <tbody>
                    <tr>
                        <td data-i18n="view.exports.row.realized">Realized events (CSV)</td>
                        <td class="small muted" data-i18n="view.exports.row.realized_desc">Form 8949-style rows: acquired/disposed, basis, proceeds, gain/loss, wash-sale.</td>
                        <td><button data-i18n="view.exports.btn.download_csv_3" class="btn" data-action="tax-csv" data-which="realized">Download CSV</button></td>
                    </tr>
                    <tr>
                        <td data-i18n="view.exports.row.open_lots">Open lots (CSV)</td>
                        <td class="small muted" data-i18n="view.exports.row.open_lots_desc">Year-end open inventory: per-lot qty, cost-per-share, basis, holding-period.</td>
                        <td><button data-i18n="view.exports.btn.download_csv_4" class="btn" data-action="tax-csv" data-which="open">Download CSV</button></td>
                    </tr>
                    <tr>
                        <td data-i18n="view.exports.row.tax_package">Tax package (printable HTML → PDF)</td>
                        <td class="small muted" data-i18n="view.exports.row.tax_package_desc">One-page report: summary cards + realized table + open-lots table + §1091 notes.</td>
                        <td><button data-i18n="view.exports.btn.open_report" class="btn" data-action="tax-pkg">Open report</button></td>
                    </tr>
                </tbody>
            </table>
            <p data-i18n="view.exports.hint.tax_package_pdf_opens_in_a_new_tab_then_p_ctrl_p_s" class="muted small">Tax-package PDF: opens in a new tab, then ⌘P / Ctrl+P → "Save as PDF".</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.exports.h2.year_chart">Closed trades per year (last 5)</h2>
            <div id="ex-chart" style="width:100%;height:200px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.exports.h2.cumulative_chart">Cumulative closed trades — running activity total</h2>
            <div id="ex-cum-chart" style="width:100%;height:200px"></div>
        </div>
    `;

    void renderYearChart(acct.id, years, tok);
    void renderCumulativeChart(acct.id, years, tok);

    mount.querySelectorAll('button[data-action]').forEach(btn => {
        btn.addEventListener('click', async () => {
            const action = btn.dataset.action;
            const orig = btn.textContent;
            btn.textContent = t('common.status.fetching');
            btn.disabled = true;
            try {
                if (action === 'csv') {
                    await downloadBlob(btn.dataset.path, btn.dataset.name);
                } else if (action === 'tax-csv') {
                    const f = mount.querySelector('#tx-export-form');
                    if (!f) return;
                    const year = f.year.value, method = f.method.value;
                    const which = btn.dataset.which;
                    const path = `/export/tax-lots/${acct.id}/${which}.csv?year=${year}&method=${method}`;
                    const name = `tax-${which}-${year}-${acct.id}.csv`;
                    await downloadBlob(path, name);
                } else if (action === 'tax-pkg') {
                    const f = mount.querySelector('#tx-export-form');
                    if (!f) return;
                    const year = f.year.value, method = f.method.value;
                    const path = `/export/tax-package/${acct.id}.html?year=${year}&method=${method}`;
                    await openBlobInNewTab(path);
                }
            } catch (e) {
                showToast(t('common.error', { err: e.message }), { level: 'error' });
            } finally {
                if (viewIsCurrent(tok)) {
                    btn.textContent = orig;
                    btn.disabled = false;
                }
            }
        });
    });
}

async function renderYearChart(accountId, years, tok) {
    const el = document.getElementById('ex-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = `<div class="muted" data-i18n="common.loading">${esc(t('common.loading'))}</div>`;
    let trades;
    try {
        trades = await api.trades(accountId, { status: 'closed', limit: 2000 });
    } catch (_) {
        trades = [];
    }
    if (!viewIsCurrent(tok)) return;
    const counts = new Map(years.map(y => [y, 0]));
    for (const tr of trades || []) {
        const iso = tr.closed_at || tr.opened_at;
        if (!iso) continue;
        const y = Number(String(iso).slice(0, 4));
        if (counts.has(y)) counts.set(y, counts.get(y) + 1);
    }
    el.innerHTML = '';
    const labels = years.slice().reverse().map(String);
    const ys = labels.map(y => counts.get(Number(y)) || 0);
    if (ys.reduce((a, b) => a + b, 0) < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.exports.empty_chart">${esc(t('view.exports.empty_chart'))}</div>`;
        return;
    }
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.exports.chart.year') },
            { label: t('view.exports.chart.count'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 14, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

async function renderCumulativeChart(accountId, years, tok) {
    const el = document.getElementById('ex-cum-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = `<div class="muted" data-i18n="common.loading">${esc(t('common.loading'))}</div>`;
    let trades;
    try {
        trades = await api.trades(accountId, { status: 'closed', limit: 2000 });
    } catch (_) {
        trades = [];
    }
    if (!viewIsCurrent(tok)) return;
    const counts = new Map(years.map(y => [y, 0]));
    for (const tr of trades || []) {
        const iso = tr.closed_at || tr.opened_at;
        if (!iso) continue;
        const y = Number(String(iso).slice(0, 4));
        if (counts.has(y)) counts.set(y, counts.get(y) + 1);
    }
    el.innerHTML = '';
    const labels = years.slice().reverse().map(String);
    const per = labels.map(y => counts.get(Number(y)) || 0);
    if (per.reduce((a, b) => a + b, 0) < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.exports.empty_cum_chart">${esc(t('view.exports.empty_cum_chart'))}</div>`;
        return;
    }
    let acc = 0;
    const cum = per.map(n => { acc += n; return acc; });
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.exports.chart.year') },
            { label: t('view.exports.chart.cum_count'),
              stroke: '#7af0a8', width: 1.6,
              fill: 'rgba(122,240,168,0.10)',
              points: { show: true, size: 8, fill: '#7af0a8', stroke: '#7af0a8' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, cum], el);
}

async function downloadBlob(path, filename) {
    const blob = await apiFetchBlob(path);
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 60_000);
}

async function openBlobInNewTab(path) {
    const blob = await apiFetchBlob(path);
    const url = URL.createObjectURL(blob);
    window.open(url, '_blank', 'noopener,noreferrer');
    setTimeout(() => URL.revokeObjectURL(url), 5 * 60_000);
}
