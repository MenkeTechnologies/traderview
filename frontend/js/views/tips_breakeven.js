// TIPS Breakeven Inflation — the market-implied inflation rate over
// any tenor, derived from the spread between nominal Treasury yields
// and Treasury Inflation-Protected Securities yields of the same maturity:
//   breakeven = nominal yield − TIPS real yield
// If realized CPI > breakeven over the horizon, TIPS wins. If realized
// CPI < breakeven, nominal wins. Above breakeven means "the market is
// already pricing more inflation than I expect" — under-favors TIPS.

import { esc } from '../util.js';
import { t } from '../i18n.js';

const RECENT_HISTORY = [
    { date: 'Dec 2024', be5: 2.28, be10: 2.30, be30: 2.27 },
    { date: 'Jun 2024', be5: 2.21, be10: 2.27, be30: 2.30 },
    { date: 'Dec 2023', be5: 2.16, be10: 2.18, be30: 2.20 },
    { date: 'Jun 2023', be5: 2.21, be10: 2.28, be30: 2.27 },
    { date: 'Dec 2022', be5: 2.30, be10: 2.30, be30: 2.30 },
    { date: 'Jun 2022', be5: 2.71, be10: 2.65, be30: 2.50 },
    { date: 'Mar 2022', be5: 3.59, be10: 2.90, be30: 2.50 },
    { date: 'Dec 2020', be5: 1.96, be10: 1.99, be30: 2.05 },
    { date: 'Dec 2010', be5: 1.66, be10: 2.27, be30: 2.50 },
];

export async function renderTipsBreakeven(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.tips_breakeven.title">// TIPS BREAKEVEN INFLATION</span></h1>
        <p class="muted small" data-i18n-html="view.tips_breakeven.intro">
            Market-implied inflation expectation, derived as
            <code>nominal Treasury yield − TIPS real yield</code> at the same
            maturity. If realized CPI runs <strong>above</strong> the breakeven,
            TIPS outperform nominals. If <strong>below</strong>, nominals win.
            Long Treasury / TIPS breakeven is the cleanest market read on
            long-run inflation expectations — better than survey data.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Nominal Treasury yield %</span>
                    <input type="number" id="tb-nom" step="0.01" min="0" max="20" value="4.20" style="width:100%">
                </label>
                <label>
                    <span class="muted small">TIPS real yield %</span>
                    <input type="number" id="tb-real" step="0.01" min="-5" max="10" value="1.90" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Your CPI forecast %/yr</span>
                    <input type="number" id="tb-cpi" step="0.1" min="0" max="20" value="2.5" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Position $</span>
                    <input type="number" id="tb-pos" step="1000" min="0" value="100000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Tenor (years)</span>
                    <input type="number" id="tb-tenor" step="1" min="1" max="40" value="10" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="tb-run">⚡ Compute</button>
            <div id="tb-result" style="margin-top:12px"></div>
            <h3 class="section-title" style="margin-top:18px">Recent breakeven history</h3>
            <table class="trades" data-table-key="tb-hist">
                <thead><tr><th>Date</th><th>5y BE %</th><th>10y BE %</th><th>30y BE %</th></tr></thead>
                <tbody>${RECENT_HISTORY.map(h => `<tr>
                    <td>${esc(h.date)}</td>
                    <td>${fmt(h.be5, 2)}%</td>
                    <td><strong>${fmt(h.be10, 2)}%</strong></td>
                    <td>${fmt(h.be30, 2)}%</td>
                </tr>`).join('')}</tbody>
            </table>
            <p class="muted small">
                Federal Reserve has a long-stated 2% PCE target. Breakeven values consistently
                above 2.30% suggest the market sees the Fed losing the inflation fight (or
                accepting a higher target). Below 2.0% suggests deflation fears.
            </p>
        </div>
    `;
    mount.querySelectorAll('#tb-nom, #tb-real, #tb-cpi, #tb-pos, #tb-tenor').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#tb-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const nom = parseFloat(mount.querySelector('#tb-nom').value) / 100;
    const real = parseFloat(mount.querySelector('#tb-real').value) / 100;
    const cpi = parseFloat(mount.querySelector('#tb-cpi').value) / 100;
    const pos = parseFloat(mount.querySelector('#tb-pos').value) || 0;
    const tenor = Math.max(1, parseInt(mount.querySelector('#tb-tenor').value, 10) || 10);
    const result = mount.querySelector('#tb-result');

    const be = nom - real;
    const gap = cpi - be;     // positive = your CPI above market = TIPS favored

    // Total nominal return: pos × (1 + nom)^tenor
    const nominalEnd = pos * Math.pow(1 + nom, tenor);
    // TIPS return: real coupons + CPI accretion on principal
    const tipsRealEnd = pos * Math.pow(1 + real, tenor);            // real terms
    const tipsNominalEnd = tipsRealEnd * Math.pow(1 + cpi, tenor);  // approximated
    const tipsAdvantage = tipsNominalEnd - nominalEnd;

    const verdict = gap > 0.0025 ? 'TIPS favored — your CPI > market breakeven'
                  : gap < -0.0025 ? 'Nominal favored — your CPI < market breakeven'
                  : 'Roughly even — both bonds near indifferent point';
    const verdictCls = gap > 0.0025 ? 'pos' : gap < -0.0025 ? 'neg' : 'muted';

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Breakeven inflation</div><div class="value">${fmt(be * 100, 2)}%</div><div class="muted small">nominal − real</div></div>
            <div class="card"><div class="label">Your CPI forecast</div><div class="value">${fmt(cpi * 100, 2)}%</div></div>
            <div class="card"><div class="label">Forecast vs market</div><div class="value ${verdictCls}">${gap >= 0 ? '+' : ''}${fmt(gap * 100, 2)} pp</div></div>
            <div class="card"><div class="label">Verdict</div><div class="value ${verdictCls}">${esc(verdict)}</div></div>
        </div>
        <h3 class="section-title">${tenor}-year terminal value comparison</h3>
        <table class="trades" data-table-key="tb-comp">
            <thead><tr><th>Bond</th><th>Yield used</th><th>End value ($${fmt(pos, 0)})</th><th>Vs alternative</th></tr></thead>
            <tbody>
                <tr>
                    <td><strong>Nominal Treasury</strong></td>
                    <td>${fmt(nom * 100, 2)}% nominal</td>
                    <td>$${fmt(nominalEnd, 0)}</td>
                    <td class="${tipsAdvantage > 0 ? 'neg' : 'pos'}">${tipsAdvantage >= 0 ? 'loses by' : 'wins by'} $${fmt(Math.abs(tipsAdvantage), 0)}</td>
                </tr>
                <tr>
                    <td><strong>TIPS</strong></td>
                    <td>${fmt(real * 100, 2)}% real + CPI accretion</td>
                    <td>$${fmt(tipsNominalEnd, 0)}</td>
                    <td class="${tipsAdvantage > 0 ? 'pos' : 'neg'}">${tipsAdvantage >= 0 ? 'wins by' : 'loses by'} $${fmt(Math.abs(tipsAdvantage), 0)}</td>
                </tr>
            </tbody>
        </table>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
