// Faber-style sector momentum rotation strategy. Each month, score 11
// sector ETFs by N-month total return, pick top K, hold equal-weighted
// for one month, rebalance. Mebane Faber 2007 / 2013 — backtested ~10-12%
// annualised with lower drawdown than buy-and-hold S&P.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderSectorRotationStrategy(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sector_rotation_strategy.title">// SECTOR MOMENTUM ROTATION (FABER)</span></h1>
        <p class="muted small" data-i18n-html="view.sector_rotation_strategy.intro">
            Mebane Faber's tactical asset allocation: every month, score 11 sector
            SPDR ETFs (XLK / XLF / XLV / XLE / XLY / XLP / XLI / XLB / XLU /
            XLRE / XLC) by N-month total-return momentum, pick top K, hold
            equal-weighted for the month, rebalance. Backtests ~10-12% annualised
            with lower drawdown than buy-and-hold S&P. Edge persists because
            most retail investors don't follow a systematic monthly rule.
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label>
                    <span class="muted small" data-i18n="view.sector_rotation_strategy.field.lookback">Lookback months</span>
                    <input type="number" id="srs-lookback" step="1" min="1" max="24" value="6" style="width:80px">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.sector_rotation_strategy.field.top_k">Top K</span>
                    <input type="number" id="srs-topk" step="1" min="1" max="11" value="3" style="width:80px">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.sector_rotation_strategy.field.days_back">Days back</span>
                    <input type="number" id="srs-days" step="30" min="180" max="3650" value="730" style="width:80px">
                </label>
                <button class="btn btn-sm primary" id="srs-run" data-shortcut="r" data-i18n="view.sector_rotation_strategy.btn.run">⚡ Run Strategy</button>
                <span class="muted small" id="srs-meta"></span>
            </div>
            <div id="srs-summary"></div>
            <h2 style="margin-top:1rem" data-i18n="view.sector_rotation_strategy.h2.current">Current Ranking</h2>
            <div id="srs-current"></div>
            <h2 style="margin-top:1rem" data-i18n="view.sector_rotation_strategy.h2.history">Monthly History</h2>
            <div id="srs-history"></div>
        </div>
    `;
    mount.querySelector('#srs-run').addEventListener('click', () => runStrategy(mount));
    await runStrategy(mount);
}

async function runStrategy(mount) {
    const summary = mount.querySelector('#srs-summary');
    const current = mount.querySelector('#srs-current');
    const history = mount.querySelector('#srs-history');
    const lookback = parseInt(mount.querySelector('#srs-lookback').value, 10) || 6;
    const topK = parseInt(mount.querySelector('#srs-topk').value, 10) || 3;
    const days = parseInt(mount.querySelector('#srs-days').value, 10) || 730;
    summary.innerHTML = `<p class="muted">${esc(t('view.sector_rotation_strategy.status.running'))}</p>`;
    current.innerHTML = '';
    history.innerHTML = '';
    try {
        const r = await api(`/sector-rotation-strategy/run?days_back=${days}&lookback_months=${lookback}&top_k=${topK}`);
        const sharpeCls = r.annualised_sharpe >= 1.0 ? 'pos' : r.annualised_sharpe >= 0.5 ? '' : 'muted';
        const ciCls = r.sharpe_ci_lo_95 > 0 ? 'pos' : 'neg';
        summary.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px">
                <div><div class="muted small">${esc(t('view.sector_rotation_strategy.field.n_months'))}</div>
                    <strong>${r.n_months}</strong></div>
                <div><div class="muted small">${esc(t('view.sector_rotation_strategy.field.ann_return'))}</div>
                    <strong class="${r.annualised_return_pct >= 0 ? 'pos' : 'neg'}">${r.annualised_return_pct.toFixed(2)}%</strong></div>
                <div><div class="muted small">${esc(t('view.sector_rotation_strategy.field.ann_vol'))}</div>
                    <strong>${r.annualised_vol_pct.toFixed(2)}%</strong></div>
                <div><div class="muted small">${esc(t('view.sector_rotation_strategy.field.sharpe'))}</div>
                    <strong class="${sharpeCls}">${r.annualised_sharpe.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.sector_rotation_strategy.field.sharpe_ci'))}</div>
                    <strong class="${ciCls}">[${r.sharpe_ci_lo_95.toFixed(2)}, ${r.sharpe_ci_hi_95.toFixed(2)}]</strong></div>
                <div><div class="muted small">${esc(t('view.sector_rotation_strategy.field.max_dd'))}</div>
                    <strong class="neg">${r.max_drawdown_pct.toFixed(2)}%</strong></div>
            </div>
        `;
        if (r.current_momentum_ranking && r.current_momentum_ranking.length) {
            current.innerHTML = `
                <table class="trades">
                    <thead><tr>
                        <th>Sector ETF</th>
                        <th>${lookback}m Momentum %</th>
                        <th>Pick this month?</th>
                    </tr></thead>
                    <tbody>${r.current_momentum_ranking.map((s, i) => `
                        <tr>
                            <td><strong>${esc(s.symbol)}</strong></td>
                            <td class="${s.momentum_pct >= 0 ? 'pos' : 'neg'}">${s.momentum_pct.toFixed(2)}%</td>
                            <td class="${i < topK ? 'pos' : 'muted'}">${i < topK ? '✓ TOP-' + (i + 1) : '—'}</td>
                        </tr>
                    `).join('')}</tbody>
                </table>
            `;
        }
        if (r.monthly_picks && r.monthly_picks.length) {
            const recent = r.monthly_picks.slice(-24).reverse();
            history.innerHTML = `
                <table class="trades">
                    <thead><tr>
                        <th>Period End</th>
                        <th>Picks</th>
                        <th>Realized Return %</th>
                    </tr></thead>
                    <tbody>${recent.map(p => {
                        const retCls = p.realized_return_pct >= 0 ? 'pos' : 'neg';
                        return `<tr>
                            <td class="muted small">${esc(p.period_end)}</td>
                            <td><strong>${(p.picks || []).map(esc).join(', ')}</strong></td>
                            <td class="${retCls}">${p.realized_return_pct >= 0 ? '+' : ''}${p.realized_return_pct.toFixed(2)}%</td>
                        </tr>`;
                    }).join('')}</tbody>
                </table>
            `;
        }
    } catch (e) {
        summary.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
