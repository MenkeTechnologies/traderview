// Yield to Call (YTC) — for callable bonds, the issuer can redeem at a stated
// price on stated dates. YTC is the IRR if held to first call; YTM if held to
// maturity; yield-to-worst is the lower of the two. Computation runs server-side
// via /calc/yield-to-call (traderview-core::yield_to_call) — a faithful port of
// the former client-side bisection solver, Python-pinned and unit-tested.
// Class-based styling (no inline styles) for release-WebKit correctness.

import { api } from '../api.js';
import { applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import { esc } from '../util.js';
import * as enh from '../calc_enhance.js';

const fmt = (n, d) => (n == null || !Number.isFinite(Number(n)) ? '—' : Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d }));
const VIEW = 'yield-to-call';
let lastReport = null;
let lastBody = null;

export async function renderYieldToCall(mount, _state) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.yield_to_call.title">// YIELD TO CALL · YTC vs YTM vs YTW</span></h1>
        <p class="muted small" data-i18n-html="view.yield_to_call.intro">
            Callable bonds: issuer can redeem at a stated price on stated dates.
            <strong>YTC</strong> is the IRR assuming the bond is called at the
            first call date. <strong>YTM</strong> assumes held to maturity.
            <strong>Yield-to-worst (YTW)</strong> is the lower of YTM and any
            YTC — that's the yield you should evaluate against, since the
            issuer will only call when it benefits them.
        </p>
        <div class="chart-panel">
            <div class="re-grid">
                <label><span class="muted small">Current price (% of par)</span>
                    <input type="number" id="ytc-price" step="0.1" min="20" max="200" value="106.5"></label>
                <label><span class="muted small">Coupon rate %/yr</span>
                    <input type="number" id="ytc-coupon" step="0.125" min="0" max="20" value="5.5"></label>
                <label><span class="muted small">Years to maturity</span>
                    <input type="number" id="ytc-mat" step="0.25" min="0.5" max="40" value="10"></label>
                <label><span class="muted small">Years to first call</span>
                    <input type="number" id="ytc-call" step="0.25" min="0.5" max="40" value="3"></label>
                <label><span class="muted small">Call price (% of par)</span>
                    <input type="number" id="ytc-callprice" step="0.5" min="80" max="120" value="100"></label>
                <label><span class="muted small">Payments / yr</span>
                    <select id="ytc-freq">
                        <option value="1">Annual (1)</option>
                        <option value="2" selected>Semi-annual (2)</option>
                        <option value="4">Quarterly (4)</option>
                    </select></label>
            </div>
            <button class="btn btn-sm primary" id="ytc-run">⚡ Compute</button>
            <div id="ytc-tools" class="ce-toolbar"></div>
            <div id="ytc-result" class="re-result"></div>
        </div>
    `;
    applyUiI18n(mount);

    const num = (id) => parseFloat(mount.querySelector(id).value) || 0;
    const readBody = () => ({
        price_pct: num('#ytc-price'),
        coupon_rate_pct: num('#ytc-coupon'),
        years_to_maturity: num('#ytc-mat'),
        years_to_call: num('#ytc-call'),
        call_price_pct: num('#ytc-callprice'),
        coupons_per_year: parseInt(mount.querySelector('#ytc-freq').value, 10) || 2,
    });
    const compute = async () => {
        const body = readBody();
        try {
            const r = await api.calcYieldToCall(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body);
        } catch (err) {
            showToast(err.message || 'Could not compute the yields.', { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#ytc-tools'), {
        viewId: VIEW, link: false, filename: 'yield-to-call.csv',
        getRows: () => reportRows(lastReport),
    });
    mount.querySelectorAll('#ytc-price, #ytc-coupon, #ytc-mat, #ytc-call, #ytc-callprice, #ytc-freq').forEach(el => {
        el.addEventListener('input', debounce(compute, 250));
    });
    mount.querySelector('#ytc-run').addEventListener('click', compute);
    compute();
}

function reportRows(r) {
    if (!r || !r.valid) return [];
    return [
        ['metric', 'value'],
        ['ytm_pct', r.ytm_pct == null ? '' : r.ytm_pct],
        ['ytc_pct', r.ytc_pct == null ? '' : r.ytc_pct],
        ['ytw_pct', r.ytw_pct == null ? '' : r.ytw_pct],
        ['current_yield_pct', r.current_yield_pct],
        ['premium_to_par_pct', r.premium_to_par_pct],
        ['verdict', r.verdict],
    ];
}

function renderResult(mount, r, body) {
    const result = mount.querySelector('#ytc-result');
    if (!r.valid) {
        result.innerHTML = `<p class="muted">Enter a positive price.</p>`;
        return;
    }
    const verdictCls = r.verdict === 'YTC' ? 'neg' : 'pos';
    const isPremium = r.is_premium;
    const callRisk = isPremium
        ? '<strong>High call risk</strong> — bond trades above par. Issuer benefits by refinancing at lower rates and paying par. Plan on YTW.'
        : '<strong>Low call risk</strong> — bond trades at/below par. Issuer has no economic reason to call. YTM likely realized.';
    // The three yields side by side — the decision is the lowest (yield-to-worst).
    const chart = enh.svgBarChart([
        { label: 'YTM', value: r.ytm_pct || 0 },
        { label: 'YTC', value: r.ytc_pct || 0 },
        { label: 'YTW', value: r.ytw_pct || 0 },
    ]);
    // Cash flow trace (assumes called) — a schedule of the entered terms.
    const freq = body.coupons_per_year;
    const cpn = 1000 * (body.coupon_rate_pct / 100) / freq;
    const callPrice = 1000 * body.call_price_pct / 100;
    const periods = Math.ceil(body.years_to_call * freq);
    let rows = '';
    for (let i = 1; i <= periods; i++) {
        const yrs = i / freq;
        const principal = i === periods ? callPrice : 0;
        rows += `<tr><td>${i}</td><td class="muted">${fmt(yrs, 2)}</td><td>$${fmt(cpn, 2)}</td><td>${principal > 0 ? '<strong>$' + fmt(principal, 2) + '</strong>' : '—'}</td><td>$${fmt(cpn + principal, 2)}</td></tr>`;
    }
    result.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label">Yield to Maturity</div><div class="value">${fmt(r.ytm_pct, 3)}%</div><div class="muted small">If held ${fmt(body.years_to_maturity, 1)} yr</div></div>
            <div class="card"><div class="label">Yield to Call</div><div class="value">${fmt(r.ytc_pct, 3)}%</div><div class="muted small">If called yr ${fmt(body.years_to_call, 1)}</div></div>
            <div class="card"><div class="label">Yield to Worst</div><div class="value ${verdictCls}"><strong>${fmt(r.ytw_pct, 3)}%</strong></div><div class="muted small">min(YTM, YTC) = ${esc(r.verdict)}</div></div>
            <div class="card"><div class="label">Current yield</div><div class="value">${fmt(r.current_yield_pct, 3)}%</div><div class="muted small">Coupon / price (ignores principal)</div></div>
            <div class="card"><div class="label">Premium / discount</div><div class="value ${isPremium ? 'neg' : 'pos'}">${isPremium ? '+' : ''}${fmt(r.premium_to_par_pct, 2)}</div><div class="muted small">vs par</div></div>
        </div>
        ${chart}
        <p class="muted small">${callRisk}</p>
        <h3 class="section-title">Cash flow trace (assumes called)</h3>
        <table class="trades" data-table-key="ytc-cf">
            <thead><tr><th>Period</th><th>Years</th><th>Coupon</th><th>Principal</th><th>Total</th></tr></thead>
            <tbody>${rows}</tbody>
        </table>
    `;
}
