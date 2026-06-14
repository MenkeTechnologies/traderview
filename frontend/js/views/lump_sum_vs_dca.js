// Lump-Sum vs DCA — classic Vanguard "lump-sum beats DCA ~2/3 of the time"
// comparison. Given a total to invest, a horizon, an expected return,
// and a DCA cadence: simulates terminal value of (a) all-in on day 1 vs
// (b) drip across the cadence, then reports the gap and the breakeven
// market return below which DCA pulls ahead.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderLumpSumVsDca(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lump_sum_vs_dca.title">// LUMP-SUM vs DCA</span></h1>
        <p class="muted small" data-i18n-html="view.lump_sum_vs_dca.intro">
            Vanguard 2012 paper: lump-sum beats DCA <strong>~67%</strong> of the time on
            a US 60/40 portfolio, with average end-value <strong>~2.4% higher</strong>
            over a 10-year horizon — because time in market typically beats waiting,
            and DCA leaves capital in cash earning less. DCA still wins emotionally
            (lower regret variance) and statistically in down or sideways markets.
            Adjust expected return below 0%/yr to see DCA's break-even.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.lump_sum_vs_dca.field.total">Total to invest $</span>
                    <input type="number" id="ls-total" step="1000" min="0" value="120000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.lump_sum_vs_dca.field.months">DCA over N months</span>
                    <input type="number" id="ls-months" step="1" min="2" max="120" value="12" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.lump_sum_vs_dca.field.horizon">Horizon (months)</span>
                    <input type="number" id="ls-horizon" step="1" min="2" max="600" value="120" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.lump_sum_vs_dca.field.return">Expected annual return %</span>
                    <input type="number" id="ls-return" step="0.5" min="-30" max="50" value="7" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.lump_sum_vs_dca.field.cash_rate">Cash drag rate %</span>
                    <input type="number" id="ls-cash" step="0.1" min="-5" max="10" value="4" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="ls-run" data-shortcut="r" data-i18n="view.lump_sum_vs_dca.btn.run">⚡ Compute</button>
            <div id="ls-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelector('#ls-run').addEventListener('click', () => runCompute(mount));
    runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#ls-result');
    const body = {
        total_usd: parseFloat(mount.querySelector('#ls-total').value) || 0,
        dca_months: parseInt(mount.querySelector('#ls-months').value, 10) || 0,
        horizon_months: parseInt(mount.querySelector('#ls-horizon').value, 10) || 0,
        expected_annual_return_pct: parseFloat(mount.querySelector('#ls-return').value) || 0,
        cash_rate_pct: parseFloat(mount.querySelector('#ls-cash').value) || 0,
    };
    if (body.total_usd <= 0 || body.dca_months < 2 || body.horizon_months < body.dca_months) {
        result.innerHTML = `<p class="muted">${esc(t('view.lump_sum_vs_dca.empty.invalid'))}</p>`;
        return;
    }
    try {
        const r = await api.calcLumpSumVsDca(body);
        renderResult(result, r, body);
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

function renderResult(result, r, body) {
    const winnerCls = r.gap_usd > 0 ? 'pos' : r.gap_usd < 0 ? 'neg' : 'muted';
    const winnerLabel = r.gap_usd > 0 ? t('view.lump_sum_vs_dca.winner.lump')
        : r.gap_usd < 0 ? t('view.lump_sum_vs_dca.winner.dca')
            : t('view.lump_sum_vs_dca.winner.tie');
    const gapStr = t('view.lump_sum_vs_dca.card.gap', {
        amount: '$' + fmt(Math.abs(r.gap_usd), 0),
        pct: (r.gap_pct >= 0 ? '+' : '') + fmt(r.gap_pct, 2) + '%',
    });
    const breakeven = r.breakeven_return_pct == null ? '—' : fmt(r.breakeven_return_pct, 2) + '%/yr';
    const assumption = t('view.lump_sum_vs_dca.assumption', {
        total: '$' + fmt(body.total_usd, 0),
        horizon: String(body.horizon_months),
        ret: fmt(body.expected_annual_return_pct, 1) + '%',
        permonth: '$' + fmt(body.total_usd / body.dca_months, 0),
        months: String(body.dca_months),
        cash: fmt(body.cash_rate_pct, 1) + '%',
    });
    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
            <div class="card">
                <div class="label" data-i18n="view.lump_sum_vs_dca.card.lump">Lump-sum end value</div>
                <div class="value">$${fmt(r.lump_end_usd, 0)}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.lump_sum_vs_dca.card.dca">DCA end value</div>
                <div class="value">$${fmt(r.dca_end_usd, 0)}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.lump_sum_vs_dca.card.winner">Winner</div>
                <div class="value ${winnerCls}"><strong>${esc(winnerLabel)}</strong></div>
                <div class="muted small">${esc(gapStr)}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.lump_sum_vs_dca.card.breakeven">Break-even market return</div>
                <div class="value">${esc(breakeven)}</div>
                <div class="muted small" data-i18n="view.lump_sum_vs_dca.card.breakeven_hint">below this, DCA wins (cash drag less of a penalty)</div>
            </div>
        </div>
        <p class="muted small">${esc(assumption)}</p>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
