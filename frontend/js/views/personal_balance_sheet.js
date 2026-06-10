// Personal balance sheet (GAAP-style). Splits assets / liabilities by
// current vs non-current / long-term, computes equity, working capital,
// current/quick ratios, debt-to-equity, solvency status.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const STATE = {
    assets: [
        { name: 'Cash + checking', value_usd: 5000,   is_current: true,  is_liquid: true },
        { name: 'Brokerage cash',  value_usd: 10000,  is_current: true,  is_liquid: true },
        { name: 'Brokerage stocks', value_usd: 90000, is_current: true,  is_liquid: false },
        { name: '401k / IRA',      value_usd: 100000, is_current: false, is_liquid: false },
        { name: 'Home (FMV)',      value_usd: 400000, is_current: false, is_liquid: false },
    ],
    liabilities: [
        { name: 'Credit card',     value_usd: 3000,   is_current: true },
        { name: 'Auto loan',       value_usd: 12000,  is_current: false },
        { name: 'Mortgage',        value_usd: 250000, is_current: false },
    ],
};

export async function renderPersonalBalanceSheet(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.personal_balance_sheet.title">// PERSONAL BALANCE SHEET</span></h1>
        <p class="muted small" data-i18n-html="view.personal_balance_sheet.intro">
            GAAP-style split into <strong>current</strong> (convertible to cash within 12 months)
            vs <strong>non-current / long-term</strong> assets and liabilities. Computes
            <strong>equity</strong>, <strong>working capital</strong>, current ratio, quick ratio,
            and debt-to-equity. Status = <strong>solvent</strong> (equity &gt; 0 and working capital
            &gt; 0) / <strong>illiquid</strong> (equity &gt; 0 but working capital ≤ 0) /
            <strong>insolvent</strong> (equity ≤ 0).
        </p>
        <div class="chart-panel">
            <h2>${esc(t('view.personal_balance_sheet.h2.assets'))}</h2>
            <table class="trades" id="pbs-assets-table">
                <thead><tr>
                    <th data-i18n="view.personal_balance_sheet.th.name">Name</th>
                    <th data-i18n="view.personal_balance_sheet.th.value">Value $</th>
                    <th data-i18n="view.personal_balance_sheet.th.is_current">Current?</th>
                    <th data-i18n="view.personal_balance_sheet.th.is_liquid">Liquid?</th>
                    <th></th>
                </tr></thead>
                <tbody id="pbs-assets-body"></tbody>
            </table>
            <button class="btn btn-sm" id="pbs-add-asset" data-i18n="view.personal_balance_sheet.btn.add_asset">＋ Add asset</button>

            <h2 style="margin-top:1rem">${esc(t('view.personal_balance_sheet.h2.liabilities'))}</h2>
            <table class="trades" id="pbs-liab-table">
                <thead><tr>
                    <th data-i18n="view.personal_balance_sheet.th.name">Name</th>
                    <th data-i18n="view.personal_balance_sheet.th.value">Value $</th>
                    <th data-i18n="view.personal_balance_sheet.th.is_current">Current?</th>
                    <th></th>
                </tr></thead>
                <tbody id="pbs-liab-body"></tbody>
            </table>
            <button class="btn btn-sm" id="pbs-add-liab" data-i18n="view.personal_balance_sheet.btn.add_liab">＋ Add liability</button>

            <div style="margin-top:1rem">
                <button class="btn btn-sm primary" id="pbs-run" data-shortcut="r" data-i18n="view.personal_balance_sheet.btn.run">⚡ Compute Balance Sheet</button>
            </div>
            <div id="pbs-result"></div>
        </div>
    `;
    drawAssets(mount);
    drawLiab(mount);
    mount.querySelector('#pbs-add-asset').addEventListener('click', () => {
        STATE.assets.push({ name: 'New asset', value_usd: 0, is_current: true, is_liquid: false });
        drawAssets(mount);
    });
    mount.querySelector('#pbs-add-liab').addEventListener('click', () => {
        STATE.liabilities.push({ name: 'New liability', value_usd: 0, is_current: false });
        drawLiab(mount);
    });
    mount.querySelector('#pbs-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function drawAssets(mount) {
    const body = mount.querySelector('#pbs-assets-body');
    body.innerHTML = STATE.assets.map((a, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(a.name)}" style="width:100%"></td>
            <td><input type="number" step="100" data-k="value_usd" data-i="${i}" value="${a.value_usd}" style="width:100%"></td>
            <td style="text-align:center"><input type="checkbox" data-k="is_current" data-i="${i}" ${a.is_current ? 'checked' : ''}></td>
            <td style="text-align:center"><input type="checkbox" data-k="is_liquid" data-i="${i}" ${a.is_liquid ? 'checked' : ''}></td>
            <td><button class="btn btn-xs" data-del="${i}" data-tip="common.tip.remove_row" data-i18n-aria-label="common.aria.remove" aria-label="Remove">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input').forEach(inp => {
        inp.addEventListener('input', () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            if (k === 'value_usd') STATE.assets[i][k] = parseFloat(inp.value) || 0;
            else if (k === 'is_current' || k === 'is_liquid') STATE.assets[i][k] = inp.checked;
            else STATE.assets[i][k] = inp.value;
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
    const body = mount.querySelector('#pbs-liab-body');
    body.innerHTML = STATE.liabilities.map((a, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(a.name)}" style="width:100%"></td>
            <td><input type="number" step="100" data-k="value_usd" data-i="${i}" value="${a.value_usd}" style="width:100%"></td>
            <td style="text-align:center"><input type="checkbox" data-k="is_current" data-i="${i}" ${a.is_current ? 'checked' : ''}></td>
            <td><button class="btn btn-xs" data-del="${i}" data-tip="common.tip.remove_row" data-i18n-aria-label="common.aria.remove" aria-label="Remove">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input').forEach(inp => {
        inp.addEventListener('input', () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            if (k === 'value_usd') STATE.liabilities[i][k] = parseFloat(inp.value) || 0;
            else if (k === 'is_current') STATE.liabilities[i][k] = inp.checked;
            else STATE.liabilities[i][k] = inp.value;
        });
    });
    body.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.liabilities.splice(parseInt(btn.dataset.del, 10), 1);
            drawLiab(mount);
        });
    });
}

async function runCompute(mount) {
    const result = mount.querySelector('#pbs-result');
    result.innerHTML = `<p class="muted">${esc(t('view.personal_balance_sheet.status.computing'))}</p>`;
    try {
        const r = await api.request('/personal-balance-sheet/compute', { method: 'POST', body: JSON.stringify(STATE) });
        const eqCls = r.equity_usd >= 0 ? 'pos' : 'neg';
        const wcCls = r.working_capital_usd >= 0 ? 'pos' : 'neg';
        const statusCls = r.status === 'solvent' ? 'pos' : r.status === 'illiquid' ? '' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.personal_balance_sheet.field.equity'))}</div>
                    <strong class="${eqCls}" style="font-size:1.4em">$${(r.equity_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.personal_balance_sheet.field.total_assets'))}</div>
                    <strong class="pos">$${(r.total_assets_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.personal_balance_sheet.field.total_liab'))}</div>
                    <strong class="neg">$${(r.total_liabilities_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.personal_balance_sheet.field.working_capital'))}</div>
                    <strong class="${wcCls}">$${(r.working_capital_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.personal_balance_sheet.field.current_ratio'))}</div>
                    <strong>${r.current_ratio == null ? '∞' : r.current_ratio.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.personal_balance_sheet.field.quick_ratio'))}</div>
                    <strong>${r.quick_ratio == null ? '∞' : r.quick_ratio.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.personal_balance_sheet.field.debt_to_equity'))}</div>
                    <strong>${r.debt_to_equity == null ? '∞' : r.debt_to_equity.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.personal_balance_sheet.field.status'))}</div>
                    <strong class="${statusCls}" style="text-transform:uppercase">${esc(t('view.personal_balance_sheet.status.' + r.status) || r.status)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.personal_balance_sheet.h2.breakdown'))}</h2>
            <table class="trades">
                <tbody>
                    <tr><td><strong>${esc(t('view.personal_balance_sheet.row.current_assets'))}</strong></td><td>$${(r.current_assets_usd / 1000).toFixed(1)}K</td></tr>
                    <tr><td><strong>${esc(t('view.personal_balance_sheet.row.non_current_assets'))}</strong></td><td>$${(r.non_current_assets_usd / 1000).toFixed(1)}K</td></tr>
                    <tr><td><strong>${esc(t('view.personal_balance_sheet.row.current_liabilities'))}</strong></td><td>$${(r.current_liabilities_usd / 1000).toFixed(1)}K</td></tr>
                    <tr><td><strong>${esc(t('view.personal_balance_sheet.row.long_term_liabilities'))}</strong></td><td>$${(r.long_term_liabilities_usd / 1000).toFixed(1)}K</td></tr>
                </tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
