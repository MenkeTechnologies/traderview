// Yield to Call (YTC) — for callable bonds, the issuer can redeem at
// a stated price (often par or par + small premium) at specified dates.
// YTC = the IRR you'd earn if held to first call date and called at
// that price. For premium bonds (priced above par), YTC < YTM because
// you'd lose the premium upfront. For discount bonds, YTC > YTM because
// you'd pick up the discount sooner.
// Investors should evaluate using the LOWER of YTM and YTC ("yield to
// worst") to avoid being surprised by an early call.

import { esc } from '../util.js';

export async function renderYieldToCall(mount, _state) {
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
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Current price (% of par)</span>
                    <input type="number" id="ytc-price" step="0.1" min="20" max="200" value="106.5" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Coupon rate %/yr</span>
                    <input type="number" id="ytc-coupon" step="0.125" min="0" max="20" value="5.5" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Years to maturity</span>
                    <input type="number" id="ytc-mat" step="0.25" min="0.5" max="40" value="10" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Years to first call</span>
                    <input type="number" id="ytc-call" step="0.25" min="0.5" max="40" value="3" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Call price (% of par)</span>
                    <input type="number" id="ytc-callprice" step="0.5" min="80" max="120" value="100" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Payments / yr</span>
                    <select id="ytc-freq" style="width:100%">
                        <option value="1">Annual (1)</option>
                        <option value="2" selected>Semi-annual (2)</option>
                        <option value="4">Quarterly (4)</option>
                    </select>
                </label>
            </div>
            <button class="btn btn-sm primary" id="ytc-run">⚡ Compute</button>
            <div id="ytc-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#ytc-price, #ytc-coupon, #ytc-mat, #ytc-call, #ytc-callprice, #ytc-freq').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#ytc-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const pricePct = parseFloat(mount.querySelector('#ytc-price').value) || 0;
    const couponRate = parseFloat(mount.querySelector('#ytc-coupon').value) / 100;
    const matYears = parseFloat(mount.querySelector('#ytc-mat').value) || 0;
    const callYears = parseFloat(mount.querySelector('#ytc-call').value) || 0;
    const callPricePct = parseFloat(mount.querySelector('#ytc-callprice').value) || 0;
    const freq = parseInt(mount.querySelector('#ytc-freq').value, 10) || 2;
    const result = mount.querySelector('#ytc-result');

    const par = 1000;
    const price = par * pricePct / 100;
    const callPrice = par * callPricePct / 100;
    const cpn = par * couponRate / freq;

    const ytm = solveYield(price, cpn, par, matYears * freq, freq);
    const ytc = solveYield(price, cpn, callPrice, callYears * freq, freq);
    const ytw = Math.min(ytm, ytc);
    const verdict = ytc < ytm ? 'YTC' : ytc > ytm ? 'YTM' : 'TIE';
    const verdictCls = verdict === 'YTC' ? 'neg' : 'pos';

    const isPremium = pricePct > 100;
    const callRisk = isPremium ? '<strong>High call risk</strong> — bond trades above par. Issuer benefits by refinancing at lower rates and paying par. Plan on YTW.' : '<strong>Low call risk</strong> — bond trades at/below par. Issuer has no economic reason to call. YTM likely realized.';

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Yield to Maturity</div><div class="value">${fmt(ytm * 100, 3)}%</div><div class="muted small">If held ${fmt(matYears, 1)} yr</div></div>
            <div class="card"><div class="label">Yield to Call</div><div class="value">${fmt(ytc * 100, 3)}%</div><div class="muted small">If called yr ${fmt(callYears, 1)}</div></div>
            <div class="card"><div class="label">Yield to Worst</div><div class="value ${verdictCls}"><strong>${fmt(ytw * 100, 3)}%</strong></div><div class="muted small">min(YTM, YTC) = ${esc(verdict)}</div></div>
            <div class="card"><div class="label">Current yield</div><div class="value">${fmt((cpn * freq) / price * 100, 3)}%</div><div class="muted small">Coupon / price (ignores principal)</div></div>
            <div class="card"><div class="label">Premium / discount</div><div class="value ${isPremium ? 'neg' : 'pos'}">${isPremium ? '+' : ''}${fmt(pricePct - 100, 2)}</div><div class="muted small">vs par</div></div>
        </div>
        <p class="muted small">${callRisk}</p>
        <h3 class="section-title">Cash flow trace (assumes called)</h3>
        <table class="trades" data-table-key="ytc-cf">
            <thead><tr><th>Period</th><th>Years</th><th>Coupon</th><th>Principal</th><th>Total</th></tr></thead>
            <tbody>${(() => {
                const periods = Math.ceil(callYears * freq);
                const out = [];
                for (let i = 1; i <= periods; i++) {
                    const yrs = i / freq;
                    const principal = i === periods ? callPrice : 0;
                    out.push(`<tr>
                        <td>${i}</td>
                        <td class="muted">${fmt(yrs, 2)}</td>
                        <td>$${fmt(cpn, 2)}</td>
                        <td>${principal > 0 ? '<strong>$' + fmt(principal, 2) + '</strong>' : '—'}</td>
                        <td>$${fmt(cpn + principal, 2)}</td>
                    </tr>`);
                }
                return out.join('');
            })()}</tbody>
        </table>
    `;
}

function solveYield(price, cpn, redemption, n, freq) {
    // Bisection on YTM (annualized).
    const F = (y) => {
        const r = y / freq;
        if (Math.abs(r) < 1e-12) return cpn * n + redemption - price;
        const f = Math.pow(1 + r, n);
        return cpn * (1 - 1/f) / r + redemption / f - price;
    };
    let lo = -0.20, hi = 0.50;
    let fl = F(lo), fh = F(hi);
    if (fl * fh > 0) {
        for (let i = 0; i < 20 && fl * fh > 0; i++) { hi *= 1.5; fh = F(hi); }
        if (fl * fh > 0) return null;
    }
    for (let i = 0; i < 200; i++) {
        const mid = (lo + hi) / 2;
        const fm = F(mid);
        if (Math.abs(fm) < 1e-9) return mid;
        if (fl * fm < 0) { hi = mid; fh = fm; }
        else            { lo = mid; fl = fm; }
    }
    return (lo + hi) / 2;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
