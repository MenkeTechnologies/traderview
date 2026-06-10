// Time Value of Money Solver — generic PV/FV/PMT/N/R solver.
// User picks which variable to solve for; the other 4 are inputs.
// Same engine HP-12C / TI-BA-II / Excel use: PV·(1+r)^n + PMT·((1+r)^n − 1)/r + FV = 0
// with standard sign convention (outflows negative, inflows positive).

import { esc } from '../util.js';
import { t } from '../i18n.js';

const SOLVE = ['fv', 'pv', 'pmt', 'n', 'r'];

export async function renderTimeValueMoney(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.time_value_money.title">// TIME VALUE OF MONEY</span></h1>
        <p class="muted small" data-i18n-html="view.time_value_money.intro">
            Generic financial calculator. Pick which variable to <strong>solve</strong>;
            fill the other four. Same engine as HP-12C / TI BA-II / Excel.
            Sign convention: <strong>outflows negative, inflows positive</strong>
            (e.g. depositing $1,000 today → PV = -1000; receiving $X future → FV = +X).
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:8px;margin-bottom:12px;flex-wrap:wrap;align-items:end">
                <label>
                    <span class="muted small">Solve for</span>
                    <select id="tvm-solve" style="width:100%">
                        <option value="fv" selected>FV — future value</option>
                        <option value="pv">PV — present value</option>
                        <option value="pmt">PMT — periodic payment</option>
                        <option value="n">N — number of periods</option>
                        <option value="r">R — rate per period %</option>
                    </select>
                </label>
                <label>
                    <span class="muted small">Periods/year</span>
                    <select id="tvm-ppy" style="width:100%">
                        <option value="1">Annual (1)</option>
                        <option value="2">Semi-annual (2)</option>
                        <option value="4">Quarterly (4)</option>
                        <option value="12" selected>Monthly (12)</option>
                        <option value="52">Weekly (52)</option>
                        <option value="365">Daily (365)</option>
                    </select>
                </label>
                <label>
                    <span class="muted small">PMT timing</span>
                    <select id="tvm-when" style="width:100%">
                        <option value="end" selected>End of period (ordinary)</option>
                        <option value="begin">Begin of period (annuity-due)</option>
                    </select>
                </label>
            </div>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">PV (present value)</span>
                    <input type="number" id="tvm-pv" step="100" value="-10000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">FV (future value)</span>
                    <input type="number" id="tvm-fv" step="100" value="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">PMT (per period)</span>
                    <input type="number" id="tvm-pmt" step="50" value="-200" style="width:100%">
                </label>
                <label>
                    <span class="muted small">N (total periods)</span>
                    <input type="number" id="tvm-n" step="1" min="1" value="360" style="width:100%">
                </label>
                <label>
                    <span class="muted small">R (annual rate %)</span>
                    <input type="number" id="tvm-r" step="0.1" value="7" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="tvm-run">⚡ Solve</button>
            <div id="tvm-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelector('#tvm-solve').addEventListener('change', () => updateDisabled(mount));
    mount.querySelectorAll('#tvm-pv, #tvm-fv, #tvm-pmt, #tvm-n, #tvm-r, #tvm-solve, #tvm-ppy, #tvm-when').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#tvm-run').addEventListener('click', () => compute(mount));
    updateDisabled(mount);
    compute(mount);
}

function updateDisabled(mount) {
    const solve = mount.querySelector('#tvm-solve').value;
    for (const k of SOLVE) {
        const el = mount.querySelector(`#tvm-${k}`);
        el.disabled = (k === solve);
        el.parentElement.style.opacity = (k === solve) ? '0.35' : '1';
    }
}

function compute(mount) {
    const solve = mount.querySelector('#tvm-solve').value;
    const ppy = parseInt(mount.querySelector('#tvm-ppy').value, 10) || 12;
    const when = mount.querySelector('#tvm-when').value === 'begin' ? 1 : 0;
    const pv = parseFloat(mount.querySelector('#tvm-pv').value) || 0;
    const fv = parseFloat(mount.querySelector('#tvm-fv').value) || 0;
    const pmt = parseFloat(mount.querySelector('#tvm-pmt').value) || 0;
    const n = parseFloat(mount.querySelector('#tvm-n').value) || 0;
    const r_ann = parseFloat(mount.querySelector('#tvm-r').value) / 100;
    const r = r_ann / ppy;
    const result = mount.querySelector('#tvm-result');

    let answer = null;
    let label = '';
    try {
        switch (solve) {
            case 'fv':  answer = solveFv(pv, pmt, n, r, when);  label = 'FV (future value)'; break;
            case 'pv':  answer = solvePv(fv, pmt, n, r, when);  label = 'PV (present value)'; break;
            case 'pmt': answer = solvePmt(pv, fv, n, r, when);  label = 'PMT (per period)'; break;
            case 'n':   answer = solveN(pv, fv, pmt, r, when);  label = 'N (total periods)'; break;
            case 'r':
                answer = solveR(pv, fv, pmt, n, when);
                label = 'R (per-period %)';
                break;
        }
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(e.message || String(e))}</p>`;
        return;
    }
    if (answer == null || !Number.isFinite(answer)) {
        result.innerHTML = `<p class="muted">No closed-form solution with these inputs (rate=0 + zero net?). Try adjusting.</p>`;
        return;
    }

    let display = '';
    if (solve === 'r') {
        const r_eff_ann = (Math.pow(1 + answer, ppy) - 1) * 100;
        const r_nom_ann = answer * ppy * 100;
        display = `<div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px">
            <div class="card"><div class="label">Per-period rate</div><div class="value pos">${fmt(answer * 100, 4)}%</div></div>
            <div class="card"><div class="label">Nominal annual</div><div class="value">${fmt(r_nom_ann, 4)}%</div></div>
            <div class="card"><div class="label">Effective annual (compounded)</div><div class="value">${fmt(r_eff_ann, 4)}%</div></div>
        </div>`;
    } else if (solve === 'n') {
        display = `<div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px">
            <div class="card"><div class="label">N (periods)</div><div class="value pos">${fmt(answer, 2)}</div></div>
            <div class="card"><div class="label">≈ years</div><div class="value">${fmt(answer / ppy, 2)}</div></div>
        </div>`;
    } else {
        display = `<div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:8px">
            <div class="card"><div class="label">${esc(label)}</div><div class="value ${answer >= 0 ? 'pos' : 'neg'}">$${fmt(answer, 2)}</div></div>
        </div>`;
    }
    result.innerHTML = display + `<p class="muted small" style="margin-top:8px">
        Equation: <code>PV·(1+r)^n + PMT·(1+r·${when})·((1+r)^n − 1)/r + FV = 0</code>
        with r = ${fmt(r * 100, 4)}%/period, ${ppy}×/yr compounding.
    </p>`;
}

function solveFv(pv, pmt, n, r, when) {
    if (r === 0) return -(pv + pmt * n);
    const f = Math.pow(1 + r, n);
    return -(pv * f + pmt * (1 + r * when) * (f - 1) / r);
}
function solvePv(fv, pmt, n, r, when) {
    if (r === 0) return -(fv + pmt * n);
    const f = Math.pow(1 + r, n);
    return -(fv + pmt * (1 + r * when) * (f - 1) / r) / f;
}
function solvePmt(pv, fv, n, r, when) {
    if (r === 0) return -(pv + fv) / n;
    const f = Math.pow(1 + r, n);
    return -(pv * f + fv) * r / ((1 + r * when) * (f - 1));
}
function solveN(pv, fv, pmt, r, when) {
    if (r === 0) {
        if (pmt === 0) return null;
        return -(pv + fv) / pmt;
    }
    // From PV·(1+r)^n + PMT·(1+r·w)·((1+r)^n − 1)/r + FV = 0
    // Let A = PMT·(1+r·w)/r ;  (1+r)^n · (PV + A) = A − FV
    const A = pmt * (1 + r * when) / r;
    const num = A - fv;
    const den = pv + A;
    if (den === 0 || (num / den) <= 0) return null;
    return Math.log(num / den) / Math.log(1 + r);
}
function solveR(pv, fv, pmt, n, when) {
    // No closed form — bisection on the residual function f(r).
    const F = (r) => {
        if (Math.abs(r) < 1e-12) return pv + pmt * n + fv;
        const f = Math.pow(1 + r, n);
        return pv * f + pmt * (1 + r * when) * (f - 1) / r + fv;
    };
    let lo = -0.99, hi = 1.0;
    // Bracket-walk: expand hi until sign change.
    let fl = F(lo), fh = F(hi);
    if (fl * fh > 0) {
        for (let i = 0; i < 20 && fl * fh > 0; i++) {
            hi *= 2;
            fh = F(hi);
        }
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
