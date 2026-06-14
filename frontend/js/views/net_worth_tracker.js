// Net-worth tracker. Given a list of assets + liabilities (each with a
// category) plus optional monthly history, reports total NW,
// breakdowns by category, M/M and Y/Y deltas, debt-to-asset %,
// and status.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import * as enh from '../calc_enhance.js';

const STATE = {
    assets: [
        { name: 'Checking',  category: 'cash',      value_usd: 5000 },
        { name: 'Brokerage', category: 'stocks',    value_usd: 100000 },
        { name: 'Home',      category: 'realestate', value_usd: 400000 },
    ],
    liabilities: [
        { name: 'Mortgage',     category: 'loan', value_usd: 250000 },
        { name: 'Credit Card',  category: 'card', value_usd: 3000 },
    ],
    history: [],
};

export async function renderNetWorthTracker(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.net_worth_tracker.title">// NET WORTH TRACKER</span></h1>
        <p class="muted small" data-i18n-html="view.net_worth_tracker.intro">
            Personal-finance fundamentals: <strong>net worth = total assets − total liabilities</strong>.
            Add line items with categories; the report shows the breakdown by category,
            debt-to-asset %, and (if you paste in monthly history) month-over-month and
            year-over-year deltas.
        </p>
        <div class="chart-panel">
            <h2>${esc(t('view.net_worth_tracker.h2.assets'))}</h2>
            <table class="trades" id="nw-assets-table">
                <thead><tr>
                    <th data-i18n="view.net_worth_tracker.th.name">Name</th>
                    <th data-i18n="view.net_worth_tracker.th.category">Category</th>
                    <th data-i18n="view.net_worth_tracker.th.value">Value $</th>
                    <th></th>
                </tr></thead>
                <tbody id="nw-assets-body"></tbody>
            </table>
            <button class="btn btn-sm" id="nw-add-asset" data-i18n="view.net_worth_tracker.btn.add_asset">＋ Add asset</button>

            <h2 style="margin-top:1rem">${esc(t('view.net_worth_tracker.h2.liabilities'))}</h2>
            <table class="trades" id="nw-liab-table">
                <thead><tr>
                    <th data-i18n="view.net_worth_tracker.th.name">Name</th>
                    <th data-i18n="view.net_worth_tracker.th.category">Category</th>
                    <th data-i18n="view.net_worth_tracker.th.value">Value $</th>
                    <th></th>
                </tr></thead>
                <tbody id="nw-liab-body"></tbody>
            </table>
            <button class="btn btn-sm" id="nw-add-liab" data-i18n="view.net_worth_tracker.btn.add_liab">＋ Add liability</button>

            <h2 style="margin-top:1rem">${esc(t('view.net_worth_tracker.h2.history'))}</h2>
            <p class="muted small" data-i18n="view.net_worth_tracker.help.history">Optional. Paste one row per month, oldest first: <code>YYYY-MM,net_worth_usd</code>. Last row is treated as &quot;this month&quot; for deltas.</p>
            <textarea id="nw-history" rows="4" style="width:100%;font-family:monospace" placeholder="2025-06,95000&#10;2025-07,97000&#10;2025-08,99500"></textarea>

            <div style="margin-top:1rem">
                <button class="btn btn-sm primary" id="nw-run" data-shortcut="r" data-i18n="view.net_worth_tracker.btn.run">⚡ Compute Net Worth</button>
            </div>
            <div id="nw-result"></div>
        </div>
    `;
    drawAssets(mount);
    drawLiab(mount);
    mount.querySelector('#nw-add-asset').addEventListener('click', () => {
        STATE.assets.push({ name: 'New asset', category: 'cash', value_usd: 0 });
        drawAssets(mount);
    });
    mount.querySelector('#nw-add-liab').addEventListener('click', () => {
        STATE.liabilities.push({ name: 'New liability', category: 'loan', value_usd: 0 });
        drawLiab(mount);
    });
    mount.querySelector('#nw-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function drawAssets(mount) {
    const body = mount.querySelector('#nw-assets-body');
    body.innerHTML = STATE.assets.map((a, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(a.name)}" style="width:100%"></td>
            <td><input type="text" data-k="category" data-i="${i}" value="${esc(a.category)}" style="width:100%"></td>
            <td><input type="number" step="100" data-k="value_usd" data-i="${i}" value="${a.value_usd}" style="width:100%"></td>
            <td><button class="btn btn-xs" data-del="${i}" data-tip="common.tip.remove_row" data-i18n-aria-label="common.aria.remove" aria-label="Remove">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input').forEach(inp => {
        inp.addEventListener('input', () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            STATE.assets[i][k] = k === 'value_usd' ? (parseFloat(inp.value) || 0) : inp.value;
        });
    });
    body.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.assets.splice(parseInt(btn.dataset.del, 10), 1);
            drawAssets(mount);
        });
    });
}

function drawLiab(mount) {
    const body = mount.querySelector('#nw-liab-body');
    body.innerHTML = STATE.liabilities.map((a, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(a.name)}" style="width:100%"></td>
            <td><input type="text" data-k="category" data-i="${i}" value="${esc(a.category)}" style="width:100%"></td>
            <td><input type="number" step="100" data-k="value_usd" data-i="${i}" value="${a.value_usd}" style="width:100%"></td>
            <td><button class="btn btn-xs" data-del="${i}" data-tip="common.tip.remove_row" data-i18n-aria-label="common.aria.remove" aria-label="Remove">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input').forEach(inp => {
        inp.addEventListener('input', () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            STATE.liabilities[i][k] = k === 'value_usd' ? (parseFloat(inp.value) || 0) : inp.value;
        });
    });
    body.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.liabilities.splice(parseInt(btn.dataset.del, 10), 1);
            drawLiab(mount);
        });
    });
}

function parseHistory(text) {
    return text.split(/\r?\n/)
        .map(l => l.trim())
        .filter(l => l.length > 0)
        .map(l => {
            const [m, n] = l.split(',');
            return { month: (m || '').trim(), net_worth_usd: parseFloat(n) || 0 };
        });
}

async function runCompute(mount) {
    const result = mount.querySelector('#nw-result');
    STATE.history = parseHistory(mount.querySelector('#nw-history').value);
    result.innerHTML = `<p class="muted">${esc(t('view.net_worth_tracker.status.computing'))}</p>`;
    try {
        const r = await api.request('/net-worth-tracker/compute', { method: 'POST', body: JSON.stringify(STATE) });
        // Assets-by-category bar chart (where the net worth sits).
        const chart = enh.svgBarChart((r.by_asset_category || []).map(c => ({ label: c.category, value: c.total_usd })));
        const nwCls = r.net_worth_usd >= 0 ? 'pos' : 'neg';
        const momCls = r.mom_delta_usd == null ? 'muted' : r.mom_delta_usd >= 0 ? 'pos' : 'neg';
        const yoyCls = r.yoy_delta_usd == null ? 'muted' : r.yoy_delta_usd >= 0 ? 'pos' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.net_worth_tracker.field.net_worth'))}</div>
                    <strong class="${nwCls}" style="font-size:1.4em">$${(r.net_worth_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.net_worth_tracker.field.total_assets'))}</div>
                    <strong class="pos">$${(r.total_assets_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.net_worth_tracker.field.total_liab'))}</div>
                    <strong class="neg">$${(r.total_liabilities_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.net_worth_tracker.field.debt_to_asset'))}</div>
                    <strong>${r.debt_to_asset_pct.toFixed(1)}%</strong></div>
                <div><div class="muted small">${esc(t('view.net_worth_tracker.field.mom'))}</div>
                    <strong class="${momCls}">${r.mom_delta_usd == null ? '—' : '$' + (r.mom_delta_usd / 1000).toFixed(1) + 'K' + (r.mom_delta_pct == null ? '' : ' (' + r.mom_delta_pct.toFixed(1) + '%)')}</strong></div>
                <div><div class="muted small">${esc(t('view.net_worth_tracker.field.yoy'))}</div>
                    <strong class="${yoyCls}">${r.yoy_delta_usd == null ? '—' : '$' + (r.yoy_delta_usd / 1000).toFixed(1) + 'K' + (r.yoy_delta_pct == null ? '' : ' (' + r.yoy_delta_pct.toFixed(1) + '%)')}</strong></div>
                <div><div class="muted small">${esc(t('view.net_worth_tracker.field.status'))}</div>
                    <strong class="${r.status === 'positive' ? 'pos' : 'neg'}" style="text-transform:uppercase">${esc(t('view.net_worth_tracker.status.' + r.status) || r.status)}</strong></div>
            </div>
            ${chart}
            <div id="nw-tools" class="ce-toolbar"></div>
            <h2 style="margin-top:1rem">${esc(t('view.net_worth_tracker.h2.by_asset_cat'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.net_worth_tracker.th.category">Category</th>
                    <th data-i18n="view.net_worth_tracker.th.total">Total $</th>
                    <th data-i18n="view.net_worth_tracker.th.share">Share %</th>
                </tr></thead>
                <tbody>${(r.by_asset_category || []).map(c => `
                    <tr><td>${esc(c.category)}</td><td>$${(c.total_usd / 1000).toFixed(1)}K</td><td>${c.share_pct.toFixed(1)}%</td></tr>
                `).join('')}</tbody>
            </table>
            <h2 style="margin-top:1rem">${esc(t('view.net_worth_tracker.h2.by_liab_cat'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.net_worth_tracker.th.category">Category</th>
                    <th data-i18n="view.net_worth_tracker.th.total">Total $</th>
                    <th data-i18n="view.net_worth_tracker.th.share">Share %</th>
                </tr></thead>
                <tbody>${(r.by_liability_category || []).map(c => `
                    <tr><td>${esc(c.category)}</td><td>$${(c.total_usd / 1000).toFixed(1)}K</td><td>${c.share_pct.toFixed(1)}%</td></tr>
                `).join('')}</tbody>
            </table>
        `;
        // Asset + liability category export (Copy / CSV). No permalink — multi-row table state.
        enh.mountToolbar(mount.querySelector('#nw-tools'), {
            viewId: 'net-worth-tracker',
            link: false,
            filename: 'net-worth-tracker.csv',
            getRows: () => [['type', 'category', 'total_usd', 'share_pct'],
                ...(r.by_asset_category || []).map(c => ['asset', c.category, c.total_usd, c.share_pct]),
                ...(r.by_liability_category || []).map(c => ['liability', c.category, c.total_usd, c.share_pct]),
                ['net_worth', '', r.net_worth_usd, '']],
        });
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
