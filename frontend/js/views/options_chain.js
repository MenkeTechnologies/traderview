// Full options chain UI — strike grid with bid/ask/IV plus computed Greeks.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderOptions(mount, _state, rest) {
    const tok = currentViewToken();
    const sym = (rest || '').toUpperCase() || 'SPY';
    mount.innerHTML = `
        <h1 class="view-title">// OPTIONS · ${esc(sym)}</h1>
        <form id="of" class="inline-form">
            <input name="sym" value="${esc(sym)}" style="text-transform:uppercase">
            <select name="exp" id="expsel"><option>loading…</option></select>
            <label>r <input name="r" value="0.045" type="number" step="any" style="width:80px"></label>
            <button class="primary" type="submit">Load</button>
        </form>
        <div id="oc-mount">loading…</div>
        <div class="chart-panel" style="margin-top:14px">
            <h2>Greeks calculator</h2>
            <form id="gf" class="inline-form">
                <select name="kind"><option value="call">call</option><option value="put">put</option></select>
                <label>S <input name="s" type="number" step="any" value="100"></label>
                <label>K <input name="k" type="number" step="any" value="100"></label>
                <label>T (yrs) <input name="t" type="number" step="any" value="0.25"></label>
                <label>σ (vol) <input name="sigma" type="number" step="any" value="0.30"></label>
                <label>r <input name="r" type="number" step="any" value="0.045"></label>
                <label>q <input name="q" type="number" step="any" value="0.0"></label>
                <label>mkt (opt'l) <input name="market_price" type="number" step="any" placeholder="for IV"></label>
                <button class="primary" type="submit">Compute</button>
            </form>
            <div id="g-out"></div>
        </div>
    `;
    const form = mount.querySelector('#of');
    const gform = mount.querySelector('#gf');
    let r = Number(form.r.value || 0.045);
    let expirations = [];
    let activeExp = null;

    async function reload() {
        const s = form.sym.value.trim().toUpperCase() || 'SPY';
        r = Number(form.r.value || 0.045);
        const ocm = mount.querySelector('#oc-mount');
        if (ocm) ocm.innerHTML = '<div class="boot">fetching chain…</div>';
        try {
            const chain = await api.options(s, activeExp);
            if (!viewIsCurrent(tok)) return;
            expirations = chain.expirations || [];
            if (!activeExp && expirations.length) activeExp = expirations[0];
            const sel = mount.querySelector('#expsel');
            if (sel) {
                sel.innerHTML = expirations.map(e =>
                    `<option value="${e}" ${e === activeExp ? 'selected' : ''}>${e}</option>`).join('');
                sel.onchange = () => { activeExp = sel.value; reload(); };
            }
            renderChain(chain, r, mount);
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const ocm2 = mount.querySelector('#oc-mount');
            if (ocm2) ocm2.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
        }
    }
    form.addEventListener('submit', (e) => { e.preventDefault(); activeExp = null; reload(); });
    await reload();
    if (!viewIsCurrent(tok)) return;

    gform.addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const params = {};
        for (const [k, v] of fd.entries()) if (v !== '') params[k] = v;
        const out = await api.greeksCalc(params);
        if (!viewIsCurrent(tok)) return;
        const g = out.greeks;
        const gOut = mount.querySelector('#g-out');
        if (!gOut) return;
        gOut.innerHTML = `
            <table class="trades" style="margin-top:8px">
                <tbody>
                    <tr><td>Price</td><td>${fmt(g.price, 4)}</td></tr>
                    <tr><td>Delta</td><td>${fmt(g.delta, 4)}</td></tr>
                    <tr><td>Gamma</td><td>${fmt(g.gamma, 5)}</td></tr>
                    <tr><td>Theta (per day)</td><td>${fmt(g.theta, 4)}</td></tr>
                    <tr><td>Vega (per 1 vol pt)</td><td>${fmt(g.vega, 4)}</td></tr>
                    <tr><td>Rho (per 1 rate pt)</td><td>${fmt(g.rho, 4)}</td></tr>
                    ${out.implied_vol != null
                        ? `<tr><td><strong>Implied vol (Newton)</strong></td><td><strong>${(out.implied_vol*100).toFixed(2)}%</strong></td></tr>`
                        : ''}
                </tbody>
            </table>`;
    });
}

function renderChain(chain, r, mount) {
    // T = (expiration midnight - now) / yearSeconds
    const expMs = new Date(chain.expiration + 'T16:00:00').getTime();
    const t = Math.max(0.0005, (expMs - Date.now()) / (365.25 * 86400_000));
    const strikes = Array.from(new Set([
        ...chain.calls.map(c => c.strike),
        ...chain.puts.map(p => p.strike),
    ])).sort((a, b) => a - b);

    const callBy = new Map(chain.calls.map(c => [c.strike, c]));
    const putBy  = new Map(chain.puts.map(p => [p.strike, p]));

    const ocm = mount.querySelector('#oc-mount');
    if (!ocm) return;
    ocm.innerHTML = `
        <div class="chart-panel">
            <h2>${esc(chain.symbol)} · spot ${fmt(chain.spot)} · exp ${esc(chain.expiration)} · T = ${(t*365).toFixed(0)}d</h2>
            <table class="trades">
                <thead><tr>
                    <th colspan="6" style="text-align:center;color:var(--green)">CALLS</th>
                    <th>Strike</th>
                    <th colspan="6" style="text-align:center;color:var(--red)">PUTS</th>
                </tr>
                <tr>
                    <th>Δ</th><th>IV</th><th>OI</th><th>Vol</th><th>Bid</th><th>Ask</th>
                    <th></th>
                    <th>Bid</th><th>Ask</th><th>Vol</th><th>OI</th><th>IV</th><th>Δ</th>
                </tr></thead>
                <tbody>${strikes.map(k => {
                    const c = callBy.get(k);
                    const p = putBy.get(k);
                    const itmC = c && c.in_the_money;
                    const itmP = p && p.in_the_money;
                    const cellsC = c ? row(c, 'call', chain.spot, k, t, r) : ['—','—','—','—','—','—'];
                    const cellsP = p ? row(p, 'put',  chain.spot, k, t, r) : ['—','—','—','—','—','—'];
                    return `<tr>
                        <td class="${itmC ? 'pos' : ''}">${cellsC[0]}</td>
                        <td>${cellsC[1]}</td>
                        <td>${cellsC[2]}</td>
                        <td>${cellsC[3]}</td>
                        <td>${cellsC[4]}</td>
                        <td>${cellsC[5]}</td>
                        <td style="background:var(--bg-secondary);text-align:center"><strong>${fmt(k, 2)}</strong></td>
                        <td>${cellsP[5]}</td>
                        <td>${cellsP[4]}</td>
                        <td>${cellsP[3]}</td>
                        <td>${cellsP[2]}</td>
                        <td>${cellsP[1]}</td>
                        <td class="${itmP ? 'pos' : ''}">${cellsP[0]}</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
        </div>`;
}

function row(c, kind, s, k, t, r) {
    const bid = c.bid; const ask = c.ask;
    const iv = c.implied_vol;
    // Compute Greeks client-side from Yahoo's IV.
    let delta = '';
    if (iv) {
        const sigma = Number(iv);
        const sqrtT = Math.sqrt(t);
        const d1 = (Math.log(s / k) + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrtT);
        const nd1 = cdf(d1);
        delta = kind === 'call' ? nd1.toFixed(3) : (nd1 - 1).toFixed(3);
    }
    return [
        delta,
        iv ? (iv * 100).toFixed(1) + '%' : '—',
        c.open_interest ?? '—',
        c.volume ?? '—',
        bid != null ? fmt(bid, 2) : '—',
        ask != null ? fmt(ask, 2) : '—',
    ];
}

function cdf(x) {
    const a1 = 0.254829592, a2 = -0.284496736, a3 = 1.421413741,
          a4 = -1.453152027, a5 = 1.061405429, p = 0.3275911;
    const sign = x < 0 ? -1 : 1;
    const ax = Math.abs(x) / Math.SQRT2;
    const tt = 1.0 / (1.0 + p * ax);
    const y = 1.0 - (((((a5 * tt + a4) * tt) + a3) * tt + a2) * tt + a1) * tt * Math.exp(-ax * ax);
    return 0.5 * (1.0 + sign * y);
}
