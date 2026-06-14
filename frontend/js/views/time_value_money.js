// Time Value of Money Solver — generic PV/FV/PMT/N/R solver.
// User picks which variable to solve for; the other 4 are inputs.
// Same engine HP-12C / TI-BA-II / Excel use: PV·(1+r)^n + PMT·((1+r)^n − 1)/r + FV = 0
// with standard sign convention (outflows negative, inflows positive).

import { api } from '../api.js';
import { esc } from '../util.js';
import { applyUiI18n, t } from '../i18n.js';

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
                    <span class="muted small" data-i18n="view.time_value_money.field.solve">Solve for</span>
                    <select id="tvm-solve" style="width:100%">
                        <option value="fv" selected data-i18n="view.time_value_money.opt.solve.fv">FV — future value</option>
                        <option value="pv" data-i18n="view.time_value_money.opt.solve.pv">PV — present value</option>
                        <option value="pmt" data-i18n="view.time_value_money.opt.solve.pmt">PMT — periodic payment</option>
                        <option value="n" data-i18n="view.time_value_money.opt.solve.n">N — number of periods</option>
                        <option value="r" data-i18n="view.time_value_money.opt.solve.r">R — rate per period %</option>
                    </select>
                </label>
                <label>
                    <span class="muted small" data-i18n="view.time_value_money.field.ppy">Periods/year</span>
                    <select id="tvm-ppy" style="width:100%">
                        <option value="1" data-i18n="view.time_value_money.opt.ppy.1">Annual (1)</option>
                        <option value="2" data-i18n="view.time_value_money.opt.ppy.2">Semi-annual (2)</option>
                        <option value="4" data-i18n="view.time_value_money.opt.ppy.4">Quarterly (4)</option>
                        <option value="12" selected data-i18n="view.time_value_money.opt.ppy.12">Monthly (12)</option>
                        <option value="52" data-i18n="view.time_value_money.opt.ppy.52">Weekly (52)</option>
                        <option value="365" data-i18n="view.time_value_money.opt.ppy.365">Daily (365)</option>
                    </select>
                </label>
                <label>
                    <span class="muted small" data-i18n="view.time_value_money.field.when">PMT timing</span>
                    <select id="tvm-when" style="width:100%">
                        <option value="end" selected data-i18n="view.time_value_money.opt.when.end">End of period (ordinary)</option>
                        <option value="begin" data-i18n="view.time_value_money.opt.when.begin">Begin of period (annuity-due)</option>
                    </select>
                </label>
            </div>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.time_value_money.field.pv">PV (present value)</span>
                    <input type="number" id="tvm-pv" step="100" value="-10000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.time_value_money.field.fv">FV (future value)</span>
                    <input type="number" id="tvm-fv" step="100" value="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.time_value_money.field.pmt">PMT (per period)</span>
                    <input type="number" id="tvm-pmt" step="50" value="-200" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.time_value_money.field.n">N (total periods)</span>
                    <input type="number" id="tvm-n" step="1" min="1" value="360" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.time_value_money.field.r">R (annual rate %)</span>
                    <input type="number" id="tvm-r" step="0.1" value="7" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="tvm-run" data-i18n="view.time_value_money.btn.run">⚡ Solve</button>
            <div id="tvm-result" style="margin-top:12px"></div>
        </div>
    `;
    applyUiI18n(mount);
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

async function compute(mount) {
    const result = mount.querySelector('#tvm-result');
    const ppy = parseInt(mount.querySelector('#tvm-ppy').value, 10) || 12;
    const whenBegin = mount.querySelector('#tvm-when').value === 'begin';
    const body = {
        solve: mount.querySelector('#tvm-solve').value,
        periods_per_year: ppy,
        when_begin: whenBegin,
        pv: parseFloat(mount.querySelector('#tvm-pv').value) || 0,
        fv: parseFloat(mount.querySelector('#tvm-fv').value) || 0,
        pmt: parseFloat(mount.querySelector('#tvm-pmt').value) || 0,
        n: parseFloat(mount.querySelector('#tvm-n').value) || 0,
        annual_rate_pct: parseFloat(mount.querySelector('#tvm-r').value) || 0,
    };
    try {
        const r = await api.calcTimeValueMoney(body);
        renderResult(result, r, body);
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

function renderResult(result, r, body) {
    if (r.answer == null) {
        result.innerHTML = `<p class="muted">${esc(t('view.time_value_money.res.nosolution'))}</p>`;
        return;
    }
    const ppy = body.periods_per_year;
    const ans = r.answer;
    let display = '';
    if (r.solve === 'r') {
        display = `<div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px">
            <div class="card"><div class="label" data-i18n="view.time_value_money.res.perperiod">Per-period rate</div><div class="value pos">${fmt(ans * 100, 4)}%</div></div>
            <div class="card"><div class="label" data-i18n="view.time_value_money.res.nominal">Nominal annual</div><div class="value">${fmt(r.nominal_annual_pct, 4)}%</div></div>
            <div class="card"><div class="label" data-i18n="view.time_value_money.res.effective">Effective annual (compounded)</div><div class="value">${fmt(r.effective_annual_pct, 4)}%</div></div>
        </div>`;
    } else if (r.solve === 'n') {
        display = `<div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px">
            <div class="card"><div class="label" data-i18n="view.time_value_money.res.n">N (periods)</div><div class="value pos">${fmt(ans, 2)}</div></div>
            <div class="card"><div class="label" data-i18n="view.time_value_money.res.years">≈ years</div><div class="value">${fmt(ans / ppy, 2)}</div></div>
        </div>`;
    } else {
        const label = t('view.time_value_money.res.label.' + r.solve);
        display = `<div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:8px">
            <div class="card"><div class="label">${esc(label)}</div><div class="value ${ans >= 0 ? 'pos' : 'neg'}">$${fmt(ans, 2)}</div></div>
        </div>`;
    }
    const note = t('view.time_value_money.res.equation', {
        w: String(body.when_begin ? 1 : 0),
        r: fmt(body.annual_rate_pct / ppy, 4) + '%',
        ppy: String(ppy),
    });
    result.innerHTML = display + `<p class="muted small" style="margin-top:8px">${note}</p>`;
    applyUiI18n(result);
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
