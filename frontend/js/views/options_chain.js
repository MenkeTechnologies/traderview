// Full options chain UI — strike grid with bid/ask/IV plus computed Greeks.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n, t as tr } from '../i18n.js';

export async function renderOptions(mount, _state, rest) {
    const tok = currentViewToken();
    const sym = (rest || '').toUpperCase() || 'SPY';
    mount.innerHTML = `
        <h1 class="view-title">${esc(tr('view.options_chain.h1', { sym }))}</h1>
        <form id="of" class="inline-form">
            <input name="sym" value="${esc(sym)}" style="text-transform:uppercase">
            <select name="exp" id="expsel"><option data-i18n="view.options_chain.opt.loading">loading…</option></select>
            <label>r <input name="r" value="0.045" type="number" step="any" style="width:80px"></label>
            <button data-i18n="view.options_chain.btn.load" class="primary" type="submit">Load</button>
        </form>
        <div id="oc-mount"><span data-i18n="common.loading">loading…</span></div>
        <div class="chart-panel" style="margin-top:14px">
            <h2 data-i18n="view.options_chain.h2.greeks_calculator">Greeks calculator</h2>
            <form id="gf" class="inline-form">
                <select name="kind"><option data-i18n="view.options_chain.opt.call" value="call">call</option><option data-i18n="view.options_chain.opt.put" value="put">put</option></select>
                <label><span data-i18n="view.options_chain.label.s">S</span>
                    <input name="s" type="number" step="any" value="100"></label>
                <label><span data-i18n="view.options_chain.label.k">K</span>
                    <input name="k" type="number" step="any" value="100"></label>
                <label><span data-i18n="view.options_chain.label.t">T (yrs)</span>
                    <input name="t" type="number" step="any" value="0.25"></label>
                <label><span data-i18n="view.options_chain.label.sigma">σ (vol)</span>
                    <input name="sigma" type="number" step="any" value="0.30"></label>
                <label><span data-i18n="view.options_chain.label.r">r</span>
                    <input name="r" type="number" step="any" value="0.045"></label>
                <label><span data-i18n="view.options_chain.label.q">q</span>
                    <input name="q" type="number" step="any" value="0.0"></label>
                <label><span data-i18n="view.options_chain.label.mkt">mkt (opt'l)</span>
                    <input name="market_price" type="number" step="any" placeholder="for IV"
                           data-i18n-placeholder="view.options_chain.placeholder.for_iv"></label>
                <button data-i18n="view.options_chain.btn.compute" class="primary" type="submit">Compute</button>
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
        if (ocm) ocm.innerHTML = '<div class="boot" data-i18n="view.options_chain.status.fetching_chain">fetching chain…</div>';
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
                    <tr><td data-i18n="view.options_chain.row.price">Price</td><td>${fmt(g.price, 4)}</td></tr>
                    <tr><td data-i18n="view.options_chain.row.delta">Delta</td><td>${fmt(g.delta, 4)}</td></tr>
                    <tr><td data-i18n="view.options_chain.row.gamma">Gamma</td><td>${fmt(g.gamma, 5)}</td></tr>
                    <tr><td data-i18n="view.options_chain.row.theta_per_day">Theta (per day)</td><td>${fmt(g.theta, 4)}</td></tr>
                    <tr><td data-i18n="view.options_chain.row.vega_per_volpt">Vega (per 1 vol pt)</td><td>${fmt(g.vega, 4)}</td></tr>
                    <tr><td data-i18n="view.options_chain.row.rho_per_ratept">Rho (per 1 rate pt)</td><td>${fmt(g.rho, 4)}</td></tr>
                    ${out.implied_vol != null
                        ? `<tr><td><strong data-i18n="view.options_chain.row.implied_vol_newton">Implied vol (Newton)</strong></td><td><strong>${(out.implied_vol*100).toFixed(2)}%</strong></td></tr>`
                        : ''}
                </tbody>
            </table>`;
        try { applyUiI18n(gOut); } catch (_) {}
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
            <h2>${esc(tr('view.options_chain.h2.chain', { symbol: chain.symbol, spot: fmt(chain.spot), expiration: chain.expiration, days: (t*365).toFixed(0) }))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.options_chain.th.calls" colspan="6" style="text-align:center;color:var(--green)">CALLS</th>
                    <th data-i18n="view.options_chain.th.strike">Strike</th>
                    <th data-i18n="view.options_chain.th.puts" colspan="6" style="text-align:center;color:var(--red)">PUTS</th>
                </tr>
                <tr>
                    <th>Δ</th><th data-i18n="view.options_chain.th.iv">IV</th><th data-i18n="view.options_chain.th.oi">OI</th><th data-i18n="view.options_chain.th.vol">Vol</th><th data-i18n="view.options_chain.th.bid">Bid</th><th data-i18n="view.options_chain.th.ask">Ask</th>
                    <th></th>
                    <th data-i18n="view.options_chain.th.bid_2">Bid</th><th data-i18n="view.options_chain.th.ask_2">Ask</th><th data-i18n="view.options_chain.th.vol_2">Vol</th><th data-i18n="view.options_chain.th.oi_2">OI</th><th data-i18n="view.options_chain.th.iv_2">IV</th><th>Δ</th>
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
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.options_chain.h2.iv_smile">IV smile (calls vs puts by strike)</h2>
            <div id="oc-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.options_chain.h2.oi_chart">OI distribution by strike (calls vs puts)</h2>
            <div id="oc-oi-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.options_chain.hint.oi_chart" class="muted small">Open interest per strike. Reveals magnet strikes for pin risk and institutional positioning independent of IV. Large peaks = where the smart money's already parked.</p>
        </div>`;
    renderIvSmile(strikes, callBy, putBy, chain.spot);
    renderOiChart(strikes, callBy, putBy);
}

function renderOiChart(strikes, callBy, putBy) {
    const el = document.getElementById('oc-oi-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const xs = strikes.filter(k => Number.isFinite(k));
    if (xs.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.options_chain.empty_oi_chart">${esc(tr('view.options_chain.empty_oi_chart'))}</div>`;
        return;
    }
    const callOi = xs.map(k => {
        const c = callBy.get(k);
        const v = c && Number.isFinite(Number(c.open_interest)) ? Number(c.open_interest) : null;
        return v;
    });
    const putOi = xs.map(k => {
        const p = putBy.get(k);
        const v = p && Number.isFinite(Number(p.open_interest)) ? Number(p.open_interest) : null;
        return v;
    });
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { auto: true }, y: { auto: true } },
        series: [
            { label: tr('view.options_chain.chart.strike') },
            { label: tr('view.options_chain.chart.call_oi'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 10, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: tr('view.options_chain.chart.put_oi'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 10, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [xs, callOi, putOi], el);
}

function renderIvSmile(strikes, callBy, putBy, spot) {
    const el = document.getElementById('oc-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const xs = strikes.filter(k => Number.isFinite(k));
    if (xs.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.options_chain.empty_chart">${esc(tr('view.options_chain.empty_chart'))}</div>`;
        return;
    }
    const callIv = xs.map(k => {
        const c = callBy.get(k);
        return c && Number.isFinite(Number(c.implied_vol)) ? Number(c.implied_vol) * 100 : null;
    });
    const putIv = xs.map(k => {
        const p = putBy.get(k);
        return p && Number.isFinite(Number(p.implied_vol)) ? Number(p.implied_vol) * 100 : null;
    });
    const spotLine = xs.map(k => Number(k) === Number(spot) ? 0 : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { auto: true }, y: { auto: true } },
        series: [
            { label: tr('view.options_chain.chart.strike') },
            { label: tr('view.options_chain.chart.call_iv'),
              stroke: '#7af0a8', width: 1.4,
              points: { show: true, size: 6, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: tr('view.options_chain.chart.put_iv'),
              stroke: '#ff3860', width: 1.4,
              points: { show: true, size: 6, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 } ],
        legend: { show: true },
    }, [xs, callIv, putIv], el);
    void spotLine;
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
