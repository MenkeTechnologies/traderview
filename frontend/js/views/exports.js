// Export hub — CSV downloads + printable tax-package HTML.
// All downloads go through the authenticated api client (bearer token) and
// arrive as blobs, then we trigger a synthetic <a download> click.
import { apiFetchBlob } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

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
                        <td>Executions</td>
                        <td class="small muted">Every raw fill: timestamp, side, qty, price, fee, broker order id.</td>
                        <td><button data-i18n="view.exports.btn.download_csv" class="btn" data-action="csv" data-path="/export/executions/${acct.id}.csv" data-name="executions-${acct.id}.csv">Download CSV</button></td>
                    </tr>
                    <tr>
                        <td>Trades</td>
                        <td class="small muted">Aggregated open→close positions with entry/exit avg, P/L, MFE/MAE, risk.</td>
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
                        <td>Realized events (CSV)</td>
                        <td class="small muted">Form 8949-style rows: acquired/disposed, basis, proceeds, gain/loss, wash-sale.</td>
                        <td><button data-i18n="view.exports.btn.download_csv_3" class="btn" data-action="tax-csv" data-which="realized">Download CSV</button></td>
                    </tr>
                    <tr>
                        <td>Open lots (CSV)</td>
                        <td class="small muted">Year-end open inventory: per-lot qty, cost-per-share, basis, holding-period.</td>
                        <td><button data-i18n="view.exports.btn.download_csv_4" class="btn" data-action="tax-csv" data-which="open">Download CSV</button></td>
                    </tr>
                    <tr>
                        <td>Tax package (printable HTML → PDF)</td>
                        <td class="small muted">One-page report: summary cards + realized table + open-lots table + §1091 notes.</td>
                        <td><button data-i18n="view.exports.btn.open_report" class="btn" data-action="tax-pkg">Open report</button></td>
                    </tr>
                </tbody>
            </table>
            <p data-i18n="view.exports.hint.tax_package_pdf_opens_in_a_new_tab_then_p_ctrl_p_s" class="muted small">Tax-package PDF: opens in a new tab, then ⌘P / Ctrl+P → "Save as PDF".</p>
        </div>
    `;

    mount.querySelectorAll('button[data-action]').forEach(btn => {
        btn.addEventListener('click', async () => {
            const action = btn.dataset.action;
            const orig = btn.textContent;
            btn.textContent = 'fetching…';
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
                alert(e.message);
            } finally {
                if (viewIsCurrent(tok)) {
                    btn.textContent = orig;
                    btn.disabled = false;
                }
            }
        });
    });
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
