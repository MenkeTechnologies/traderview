// Mortgage Payoff vs Invest — common dilemma. Given an existing mortgage
// + extra cash/month, compare: (a) put extra against principal (saves
// interest, frees up cash earlier) vs (b) invest the extra in a market
// portfolio at expected return r. Reports end-state net wealth at
// horizon — house equity + investment account in both paths.
// Includes mortgage-interest tax deduction adjustment for itemizers.


export async function renderMortgagePayoffVsInvest(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mortgage_payoff_vs_invest.title">// MORTGAGE PAYOFF vs INVEST</span></h1>
        <p class="muted small" data-i18n-html="view.mortgage_payoff_vs_invest.intro">
            Decision matrix for: <strong>extra cash/mo against mortgage</strong>
            vs <strong>same cash into the market</strong>. Compares end-state
            net wealth (house equity + investment account) across N years.
            If your mortgage rate (after itemization) is below your expected
            return, investing wins on expected value but loses on certainty.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Current mortgage balance $</span>
                    <input type="number" id="mpi-bal" step="1000" min="0" value="350000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Mortgage rate %</span>
                    <input type="number" id="mpi-rate" step="0.05" min="0" max="20" value="6.5" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Remaining term (months)</span>
                    <input type="number" id="mpi-term" step="1" min="12" max="480" value="300" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Extra $/month</span>
                    <input type="number" id="mpi-extra" step="50" min="0" value="500" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Expected market return %</span>
                    <input type="number" id="mpi-er" step="0.1" min="-10" max="30" value="7" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Marginal tax bracket %</span>
                    <input type="number" id="mpi-tax" step="1" min="0" max="50" value="22" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Itemize mortgage interest?</span>
                    <select id="mpi-item" style="width:100%">
                        <option value="0" selected>No (std deduction)</option>
                        <option value="1">Yes</option>
                    </select>
                </label>
                <label>
                    <span class="muted small">Horizon (years)</span>
                    <input type="number" id="mpi-horizon" step="1" min="1" max="40" value="25" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="mpi-run">⚡ Compare</button>
            <div id="mpi-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#mpi-bal, #mpi-rate, #mpi-term, #mpi-extra, #mpi-er, #mpi-tax, #mpi-item, #mpi-horizon').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#mpi-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const bal0 = parseFloat(mount.querySelector('#mpi-bal').value) || 0;
    const m_rate = parseFloat(mount.querySelector('#mpi-rate').value) / 100;
    const term = parseInt(mount.querySelector('#mpi-term').value, 10) || 1;
    const extra = parseFloat(mount.querySelector('#mpi-extra').value) || 0;
    const er = parseFloat(mount.querySelector('#mpi-er').value) / 100;
    const tax = parseFloat(mount.querySelector('#mpi-tax').value) / 100;
    const itemize = mount.querySelector('#mpi-item').value === '1';
    const horizon = Math.max(1, parseInt(mount.querySelector('#mpi-horizon').value, 10) || 1);
    const result = mount.querySelector('#mpi-result');

    const r_m = m_rate / 12;
    const i_m = Math.pow(1 + er, 1/12) - 1;
    const effRate = itemize ? m_rate * (1 - tax) : m_rate;
    const baseP_I = amort(bal0, r_m, term);
    const horizonMonths = horizon * 12;

    // PATH A — pay extra against mortgage.
    let balA = bal0;
    let cashFreedTotal = 0;
    let cashFreedInvested = 0;     // invested after mortgage payoff
    let interestPaidA = 0;
    let payoffMonthA = null;
    for (let m = 1; m <= horizonMonths; m++) {
        cashFreedInvested *= (1 + i_m);   // any "freed" cash compounds
        if (balA > 0) {
            const interest = balA * r_m;
            interestPaidA += interest;
            const pay = baseP_I + extra;
            const principal = pay - interest;
            balA = Math.max(0, balA - principal);
            if (balA === 0 && payoffMonthA == null) payoffMonthA = m;
        } else {
            // Mortgage gone — redirect (P+I + extra) into investments.
            cashFreedInvested += baseP_I + extra;
            cashFreedTotal += baseP_I + extra;
        }
    }
    // Net wealth in path A = cashFreedInvested (house equity is identical
    // either way at horizon assuming same home value; we compare just the
    // financial delta).

    // PATH B — invest the extra; mortgage stays on baseline.
    let balB = bal0;
    let investedB = 0;
    let interestPaidB = 0;
    let payoffMonthB = null;
    for (let m = 1; m <= horizonMonths; m++) {
        investedB *= (1 + i_m);
        if (balB > 0) {
            const interest = balB * r_m;
            interestPaidB += interest;
            const principal = baseP_I - interest;
            balB = Math.max(0, balB - principal);
            if (balB === 0 && payoffMonthB == null) payoffMonthB = m;
            investedB += extra;
        } else {
            // Mortgage gone — redirect P+I + extra into investments.
            investedB += baseP_I + extra;
        }
    }

    const wealthA = cashFreedInvested - balA;     // subtract remaining debt if any
    const wealthB = investedB - balB;
    const gap = wealthB - wealthA;
    const winner = gap > 0 ? 'INVEST' : gap < 0 ? 'PAYOFF' : 'TIE';
    const winnerCls = gap > 0 ? 'pos' : gap < 0 ? 'neg' : 'muted';

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Path A: Pay extra → mortgage</div><div class="value">$${fmt(wealthA, 0)}</div><div class="muted small">Payoff month ${payoffMonthA ?? '—'} · interest $${fmt(interestPaidA, 0)}</div></div>
            <div class="card"><div class="label">Path B: Invest extra</div><div class="value">$${fmt(wealthB, 0)}</div><div class="muted small">Payoff month ${payoffMonthB ?? '—'} · interest $${fmt(interestPaidB, 0)}</div></div>
            <div class="card"><div class="label">Winner</div><div class="value ${winnerCls}"><strong>${winner}</strong></div><div class="muted small">by $${fmt(Math.abs(gap), 0)} (${fmt(Math.abs(gap) / Math.max(1, Math.abs(wealthA)) * 100, 2)}%)</div></div>
            <div class="card"><div class="label">Effective mortgage rate</div><div class="value">${fmt(effRate * 100, 2)}%</div><div class="muted small">${itemize ? `after ${fmt(tax * 100, 0)}% itemized deduction` : 'no itemized adjustment'}</div></div>
            <div class="card"><div class="label">Vs expected return</div><div class="value ${er > effRate ? 'pos' : 'neg'}">${fmt(er * 100, 2)}%</div><div class="muted small">Spread ${fmt((er - effRate) * 100, 2)} pp</div></div>
        </div>
        <p class="muted small">
            Interest saved by payoff path: <strong>$${fmt(interestPaidB - interestPaidA, 0)}</strong>.
            Investing wins if the spread (expected return − effective mortgage rate) compounds
            to more than the guaranteed interest saved, which it does in expectation but not
            necessarily in any single outcome. Mortgage payoff is risk-free; the market is not.
        </p>
    `;
}

function amort(bal, r_m, n) {
    if (r_m === 0) return bal / n;
    const f = Math.pow(1 + r_m, n);
    return bal * r_m * f / (f - 1);
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
