// Compound Interest Calculator — foundational PV + PMT → FV.
// Supports daily/weekly/monthly/quarterly/annual compounding,
// regular contributions, and an optional contribution growth rate
// (e.g. 3%/yr to match raises). Year-by-year balance + interest table.


export async function renderCompoundInterest(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.compound_interest.title">// COMPOUND INTEREST</span></h1>
        <p class="muted small" data-i18n="view.compound_interest.intro">
            FV of a present value plus periodic contributions across N years
            at compound rate r, with optional annual contribution escalator
            (e.g. 3%/yr to track raises).
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Starting principal $</span>
                    <input type="number" id="ci-pv" step="100" min="0" value="10000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Annual rate %</span>
                    <input type="number" id="ci-rate" step="0.1" min="-20" max="50" value="7" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Years</span>
                    <input type="number" id="ci-years" step="1" min="1" max="80" value="30" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Periodic contribution $</span>
                    <input type="number" id="ci-pmt" step="100" min="0" value="500" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Compounding</span>
                    <select id="ci-compound" style="width:100%">
                        <option value="1">Annually (1×/yr)</option>
                        <option value="4">Quarterly (4×/yr)</option>
                        <option value="12" selected>Monthly (12×/yr)</option>
                        <option value="52">Weekly (52×/yr)</option>
                        <option value="365">Daily (365×/yr)</option>
                    </select>
                </label>
                <label>
                    <span class="muted small">Contribution growth %/yr</span>
                    <input type="number" id="ci-growth" step="0.1" min="-10" max="20" value="0" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="ci-run">⚡ Compute</button>
            <div id="ci-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#ci-pv, #ci-rate, #ci-years, #ci-pmt, #ci-compound, #ci-growth').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#ci-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const pv = parseFloat(mount.querySelector('#ci-pv').value) || 0;
    const r_ann = parseFloat(mount.querySelector('#ci-rate').value) / 100;
    const years = parseInt(mount.querySelector('#ci-years').value, 10) || 0;
    const pmt = parseFloat(mount.querySelector('#ci-pmt').value) || 0;
    const periods = parseInt(mount.querySelector('#ci-compound').value, 10) || 12;
    const g_ann = parseFloat(mount.querySelector('#ci-growth').value) / 100;
    const result = mount.querySelector('#ci-result');
    if (years <= 0) {
        result.innerHTML = `<p class="muted">Enter a positive horizon.</p>`;
        return;
    }
    const r_per = r_ann / periods;
    let balance = pv;
    let totalContrib = pv;
    let currentPmt = pmt;
    const rows = [];
    for (let y = 1; y <= years; y++) {
        let yearContrib = 0;
        let yearInterest = 0;
        for (let p = 0; p < periods; p++) {
            const interest = balance * r_per;
            yearInterest += interest;
            balance += interest + currentPmt;
            yearContrib += currentPmt;
            totalContrib += currentPmt;
        }
        rows.push({ year: y, contrib: yearContrib, interest: yearInterest, balance });
        currentPmt = currentPmt * (1 + g_ann);
    }
    const final = balance;
    const totalInterest = final - totalContrib;
    const annualEffective = r_ann > 0 ? (Math.pow(1 + r_per, periods) - 1) : 0;
    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Final balance</div><div class="value pos">$${fmt(final, 0)}</div></div>
            <div class="card"><div class="label">Total contributed</div><div class="value">$${fmt(totalContrib, 0)}</div></div>
            <div class="card"><div class="label">Total interest earned</div><div class="value pos">$${fmt(totalInterest, 0)}</div><div class="muted small">${fmt((totalInterest / totalContrib) * 100, 1)}% over contributions</div></div>
            <div class="card"><div class="label">Effective annual rate</div><div class="value">${fmt(annualEffective * 100, 3)}%</div><div class="muted small">vs ${fmt(r_ann * 100, 2)}% nominal</div></div>
        </div>
        <table class="trades" data-table-key="ci-rows">
            <thead><tr>
                <th>Year</th>
                <th>Contributed</th>
                <th>Interest earned</th>
                <th>End balance</th>
                <th>Interest %</th>
            </tr></thead>
            <tbody>${rows.map(r => `<tr>
                <td>${r.year}</td>
                <td>$${fmt(r.contrib, 0)}</td>
                <td class="pos">$${fmt(r.interest, 0)}</td>
                <td><strong>$${fmt(r.balance, 0)}</strong></td>
                <td class="muted">${r.contrib + r.interest > 0 ? fmt((r.interest / (r.contrib + r.interest)) * 100, 1) + '%' : '—'}</td>
            </tr>`).join('')}</tbody>
        </table>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
