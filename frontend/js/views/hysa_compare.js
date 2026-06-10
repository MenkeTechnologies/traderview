// High-Yield Savings Account comparison.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const STATE = {
    deposit_usd: 10000,
    months: 12,
    banks: [
        { name: 'Ally',           apy_pct: 4.20, monthly_fee_usd: 0,  min_balance_usd: 0 },
        { name: 'Marcus',         apy_pct: 4.40, monthly_fee_usd: 0,  min_balance_usd: 0 },
        { name: 'Discover',       apy_pct: 4.25, monthly_fee_usd: 0,  min_balance_usd: 0 },
        { name: 'Capital One 360', apy_pct: 4.10, monthly_fee_usd: 0,  min_balance_usd: 0 },
        { name: 'CIT Platinum',   apy_pct: 4.85, monthly_fee_usd: 0,  min_balance_usd: 5000 },
    ],
};

export async function renderHysaCompare(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.hysa_compare.title">// HYSA COMPARE</span></h1>
        <p class="muted small" data-i18n-html="view.hysa_compare.intro">
            High-Yield Savings Account comparison: per-bank effective APY (accounting for
            monthly compounding), total interest, fees paid, net gain, and min-balance flag.
            Picks the winner by highest <strong>net gain</strong> after fees among the
            accounts where you meet the minimum.
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:8px;align-items:end;margin-bottom:12px">
                <label style="flex:0 0 200px">
                    <span class="muted small" data-i18n="view.hysa_compare.field.deposit">Deposit $</span>
                    <input type="number" id="hc-deposit" step="500" min="0" value="${STATE.deposit_usd}" style="width:100%">
                </label>
                <label style="flex:0 0 140px">
                    <span class="muted small" data-i18n="view.hysa_compare.field.months">Months</span>
                    <input type="number" id="hc-months" step="1" min="0" max="600" value="${STATE.months}" style="width:100%">
                </label>
                <button class="btn btn-sm primary" id="hc-run" data-shortcut="r" data-i18n="view.hysa_compare.btn.run">⚡ Compare</button>
            </div>
            <h2>${esc(t('view.hysa_compare.h2.banks'))}</h2>
            <table class="trades" id="hc-table">
                <thead><tr>
                    <th data-i18n="view.hysa_compare.th.name">Bank</th>
                    <th data-i18n="view.hysa_compare.th.apy">APY %</th>
                    <th data-i18n="view.hysa_compare.th.fee">Monthly fee $</th>
                    <th data-i18n="view.hysa_compare.th.min">Min balance $</th>
                    <th></th>
                </tr></thead>
                <tbody id="hc-body"></tbody>
            </table>
            <button class="btn btn-sm" id="hc-add" data-i18n="view.hysa_compare.btn.add">＋ Add bank</button>
            <div id="hc-result"></div>
        </div>
    `;
    drawRows(mount);
    mount.querySelector('#hc-deposit').addEventListener('input', e => {
        STATE.deposit_usd = parseFloat(e.target.value) || 0;
    });
    mount.querySelector('#hc-months').addEventListener('input', e => {
        STATE.months = parseInt(e.target.value, 10) || 0;
    });
    mount.querySelector('#hc-add').addEventListener('click', () => {
        STATE.banks.push({ name: 'New bank', apy_pct: 4.0, monthly_fee_usd: 0, min_balance_usd: 0 });
        drawRows(mount);
    });
    mount.querySelector('#hc-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function drawRows(mount) {
    const body = mount.querySelector('#hc-body');
    body.innerHTML = STATE.banks.map((b, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(b.name)}" style="width:100%"></td>
            <td><input type="number" step="0.05" min="0" max="30" data-k="apy_pct" data-i="${i}" value="${b.apy_pct}" style="width:100%"></td>
            <td><input type="number" step="1" min="0" data-k="monthly_fee_usd" data-i="${i}" value="${b.monthly_fee_usd}" style="width:100%"></td>
            <td><input type="number" step="500" min="0" data-k="min_balance_usd" data-i="${i}" value="${b.min_balance_usd}" style="width:100%"></td>
            <td><button class="btn btn-xs" data-del="${i}">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input').forEach(inp => {
        inp.addEventListener('input', () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            STATE.banks[i][k] = k === 'name' ? inp.value : (parseFloat(inp.value) || 0);
        });
    });
    body.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.banks.splice(parseInt(btn.dataset.del, 10), 1);
            drawRows(mount);
        });
    });
}

async function runCompute(mount) {
    const result = mount.querySelector('#hc-result');
    result.innerHTML = `<p class="muted">${esc(t('view.hysa_compare.status.computing'))}</p>`;
    try {
        const r = await api.request('/hysa-compare/compute', { method: 'POST', body: JSON.stringify(STATE) });
        const sorted = (r.banks || []).slice().sort((a, b) => b.net_gain_usd - a.net_gain_usd);
        result.innerHTML = `
            <div style="margin-top:1rem">
                <div class="muted small">${esc(t('view.hysa_compare.field.winner'))}</div>
                <strong class="pos" style="font-size:1.4em">${esc(r.winner_name || '—')}</strong>
                <span class="muted" style="margin-left:1rem">${esc(t('view.hysa_compare.field.winner_gain'))}: <strong class="pos">$${r.winner_net_gain_usd.toFixed(2)}</strong></span>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.hysa_compare.h2.results'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.hysa_compare.th.name">Bank</th>
                    <th data-i18n="view.hysa_compare.th.apy_h">APY</th>
                    <th data-i18n="view.hysa_compare.th.eff_apy">Effective APY</th>
                    <th data-i18n="view.hysa_compare.th.interest">Interest</th>
                    <th data-i18n="view.hysa_compare.th.fees">Fees</th>
                    <th data-i18n="view.hysa_compare.th.net">Net gain</th>
                    <th data-i18n="view.hysa_compare.th.balance">Final balance</th>
                    <th data-i18n="view.hysa_compare.th.met">Min met?</th>
                </tr></thead>
                <tbody>${sorted.map((b, idx) => {
                    const isWinner = b.name === r.winner_name;
                    return `<tr style="${isWinner ? 'background:rgba(57,255,20,0.08)' : ''}">
                        <td><strong>${esc(b.name)}</strong></td>
                        <td>${b.apy_pct.toFixed(2)}%</td>
                        <td>${b.effective_apy_pct.toFixed(3)}%</td>
                        <td class="pos">$${b.interest_earned_usd.toFixed(2)}</td>
                        <td class="${b.total_fees_usd > 0 ? 'neg' : ''}">$${b.total_fees_usd.toFixed(2)}</td>
                        <td><strong class="${b.net_gain_usd > 0 ? 'pos' : 'neg'}">$${b.net_gain_usd.toFixed(2)}</strong></td>
                        <td>$${b.final_balance_usd.toFixed(2)}</td>
                        <td class="${b.min_balance_met ? 'pos' : 'neg'}">${b.min_balance_met ? '✓' : '✗'}</td>
                    </tr>${idx === -1 ? '' : ''}`;
                }).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
