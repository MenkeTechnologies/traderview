// Elizabeth Warren's 50/30/20 rule: split after-tax income into
// 50% needs, 30% wants, 20% savings + debt principal. Each input
// row classifies into one bucket. Compute reports per-bucket actual
// vs ideal + delta + status.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const STATE = {
    net_monthly_income_usd: 5000,
    rows: [
        { name: 'Rent / mortgage',  bucket: 'needs',   amount_usd: 1800 },
        { name: 'Groceries',        bucket: 'needs',   amount_usd: 500 },
        { name: 'Utilities',        bucket: 'needs',   amount_usd: 180 },
        { name: 'Insurance',        bucket: 'needs',   amount_usd: 200 },
        { name: 'Dining out',       bucket: 'wants',   amount_usd: 400 },
        { name: 'Streaming',        bucket: 'wants',   amount_usd: 60 },
        { name: 'Hobbies',          bucket: 'wants',   amount_usd: 300 },
        { name: '401k',             bucket: 'savings', amount_usd: 800 },
        { name: 'Roth IRA',         bucket: 'savings', amount_usd: 200 },
    ],
};

export async function renderFiftyThirtyTwenty(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.fifty_thirty_twenty.title">// 50/30/20 BUDGET RULE</span></h1>
        <p class="muted small" data-i18n-html="view.fifty_thirty_twenty.intro">
            Elizabeth Warren's 50/30/20 rule (<em>All Your Worth</em>, 2005). Split after-tax
            (net) income: <strong>50% needs</strong> (rent, groceries, utilities, insurance,
            minimum debt), <strong>30% wants</strong> (dining out, streaming, hobbies),
            <strong>20% savings</strong> (emergency fund, retirement, extra debt principal).
            For needs/wants <em>under</em> ideal is on-track; for savings <em>at-or-above</em>
            ideal is on-track.
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:8px;align-items:end;margin-bottom:12px">
                <label style="flex:0 0 240px">
                    <span class="muted small" data-i18n="view.fifty_thirty_twenty.field.income">Net monthly income $</span>
                    <input type="number" id="fty-income" step="100" min="0" value="${STATE.net_monthly_income_usd}" style="width:100%">
                </label>
                <button class="btn btn-sm primary" id="fty-run" data-shortcut="r" data-i18n="view.fifty_thirty_twenty.btn.run">⚡ Compute Split</button>
            </div>
            <h2>${esc(t('view.fifty_thirty_twenty.h2.rows'))}</h2>
            <table class="trades" id="fty-table">
                <thead><tr>
                    <th data-i18n="view.fifty_thirty_twenty.th.name">Item</th>
                    <th data-i18n="view.fifty_thirty_twenty.th.bucket">Bucket</th>
                    <th data-i18n="view.fifty_thirty_twenty.th.amount">Amount $</th>
                    <th></th>
                </tr></thead>
                <tbody id="fty-body"></tbody>
            </table>
            <button class="btn btn-sm" id="fty-add" data-i18n="view.fifty_thirty_twenty.btn.add">＋ Add row</button>
            <div id="fty-result"></div>
        </div>
    `;
    drawRows(mount);
    mount.querySelector('#fty-income').addEventListener('input', e => {
        STATE.net_monthly_income_usd = parseFloat(e.target.value) || 0;
    });
    mount.querySelector('#fty-add').addEventListener('click', () => {
        STATE.rows.push({ name: 'New row', bucket: 'wants', amount_usd: 0 });
        drawRows(mount);
    });
    mount.querySelector('#fty-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function drawRows(mount) {
    const body = mount.querySelector('#fty-body');
    body.innerHTML = STATE.rows.map((r, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(r.name)}" style="width:100%"></td>
            <td>
                <select data-k="bucket" data-i="${i}">
                    <option value="needs"   ${r.bucket === 'needs'   ? 'selected' : ''}>Needs (50%)</option>
                    <option value="wants"   ${r.bucket === 'wants'   ? 'selected' : ''}>Wants (30%)</option>
                    <option value="savings" ${r.bucket === 'savings' ? 'selected' : ''}>Savings (20%)</option>
                </select>
            </td>
            <td><input type="number" step="25" min="0" data-k="amount_usd" data-i="${i}" value="${r.amount_usd}" style="width:100%"></td>
            <td><button class="btn btn-xs" data-del="${i}">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input,select').forEach(inp => {
        const ev = inp.tagName === 'SELECT' ? 'change' : 'input';
        inp.addEventListener(ev, () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            STATE.rows[i][k] = k === 'amount_usd' ? (parseFloat(inp.value) || 0) : inp.value;
        });
    });
    body.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.rows.splice(parseInt(btn.dataset.del, 10), 1);
            drawRows(mount);
        });
    });
}

async function runCompute(mount) {
    const result = mount.querySelector('#fty-result');
    result.innerHTML = `<p class="muted">${esc(t('view.fifty_thirty_twenty.status.computing'))}</p>`;
    try {
        const r = await api('/fifty-thirty-twenty/compute', { method: 'POST', body: JSON.stringify(STATE) });
        const overallCls = r.overall_status === 'on-track' ? 'pos' : 'neg';
        const bucketRow = b => `
            <tr>
                <td><strong style="text-transform:uppercase">${esc(t('view.fifty_thirty_twenty.bucket.' + b.bucket) || b.bucket)}</strong></td>
                <td>${b.ideal_pct.toFixed(0)}%</td>
                <td>$${b.ideal_usd.toFixed(0)}</td>
                <td>$${b.actual_usd.toFixed(0)}</td>
                <td>${b.actual_pct.toFixed(1)}%</td>
                <td class="${b.delta_usd > 0 ? 'neg' : 'pos'}">${b.delta_usd >= 0 ? '+' : ''}$${b.delta_usd.toFixed(0)}</td>
                <td class="${b.status === 'on-track' ? 'pos' : 'neg'}" style="text-transform:uppercase"><strong>${esc(t('view.fifty_thirty_twenty.status.' + b.status.replace('-', '_')) || b.status)}</strong></td>
            </tr>
        `;
        result.innerHTML = `
            <div style="margin-top:1rem">
                <div class="muted small">${esc(t('view.fifty_thirty_twenty.field.overall'))}</div>
                <strong class="${overallCls}" style="font-size:1.4em;text-transform:uppercase">${esc(t('view.fifty_thirty_twenty.status.' + r.overall_status.replace('-', '_')) || r.overall_status)}</strong>
                <span class="muted" style="margin-left:1rem">${esc(t('view.fifty_thirty_twenty.field.unallocated'))}: <strong>$${r.unallocated_usd.toFixed(0)}</strong></span>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.fifty_thirty_twenty.h2.buckets'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.fifty_thirty_twenty.th.bucket_h">Bucket</th>
                    <th data-i18n="view.fifty_thirty_twenty.th.ideal_pct">Ideal %</th>
                    <th data-i18n="view.fifty_thirty_twenty.th.ideal_usd">Ideal $</th>
                    <th data-i18n="view.fifty_thirty_twenty.th.actual_usd">Actual $</th>
                    <th data-i18n="view.fifty_thirty_twenty.th.actual_pct">Actual %</th>
                    <th data-i18n="view.fifty_thirty_twenty.th.delta">Delta</th>
                    <th data-i18n="view.fifty_thirty_twenty.th.status">Status</th>
                </tr></thead>
                <tbody>
                    ${bucketRow(r.needs)}
                    ${bucketRow(r.wants)}
                    ${bucketRow(r.savings)}
                </tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
