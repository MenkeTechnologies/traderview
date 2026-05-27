// Fill quality analytics — bar-level slippage approximation since we
// don't cache intraday bid/ask. Shows fill-in-range %, slippage vs HLC/3
// typical price, and groupings by symbol / size / hour-of-day.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';

export async function renderFillQuality(mount, state) {
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) { mount.innerHTML = `<p class="boot">No account selected.</p>`; return; }
    mount.innerHTML = `
        <h1 class="view-title">// FILL QUALITY — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p class="muted small">Bar-level approximation — TraderView caches daily OHLC, not
            intraday bid/ask, so this is a directional metric not tick-perfect slippage.
            <strong>fill-in-range</strong> places the fill on the day's high-low line; the
            <strong>fill-efficiency</strong> column flips that for buys (lower = better) and
            sells (higher = better) so 100% always means "best possible fill within the day's
            range". <strong>Slippage bps</strong> = deviation from typical price (HLC/3),
            sign-flipped so positive = worse than typical for that side.</p>

        <div id="fq-out"><div class="boot">loading…</div></div>
    `;
    try {
        const r = await api.fillQuality(acct.id);
        render(r);
    } catch (e) {
        document.getElementById('fq-out').innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function render(r) {
    const o = r.overall;
    const effCls   = o.avg_fill_efficiency >= 0.6 ? 'pos' : o.avg_fill_efficiency >= 0.4 ? '' : 'neg';
    const slipCls  = o.avg_slippage_bps <= 5 ? 'pos' : o.avg_slippage_bps <= 25 ? '' : 'neg';
    document.getElementById('fq-out').innerHTML = `
        <div class="cards">
            <div class="card"><div class="label">Sampled fills</div>
                <div class="value">${o.samples}</div>
                ${r.skipped_no_bar > 0 ? `<div class="small muted">${r.skipped_no_bar} skipped (no bar)</div>` : ''}
            </div>
            <div class="card"><div class="label">Avg fill efficiency</div>
                <div class="value ${effCls}">${(o.avg_fill_efficiency * 100).toFixed(1)}%</div>
                <div class="small muted">100% = best in day's range</div></div>
            <div class="card"><div class="label">Avg slippage</div>
                <div class="value ${slipCls}">${(o.avg_slippage_bps >= 0 ? '+' : '') + o.avg_slippage_bps.toFixed(1)} bps</div>
                <div class="small muted">vs HLC/3 typical, side-adj</div></div>
            <div class="card"><div class="label">Median slippage</div>
                <div class="value">${o.median_slippage_bps.toFixed(1)} bps</div></div>
            <div class="card"><div class="label">Worst / Best</div>
                <div class="value small"><span class="neg">+${o.worst_slippage_bps.toFixed(1)}</span> /
                    <span class="pos">${o.best_slippage_bps.toFixed(1)}</span></div></div>
        </div>

        <div class="panel-grid" style="display:grid;grid-template-columns:1fr 1fr;gap:10px;">
            ${bucketPanel('By symbol (worst slippage first)', r.by_symbol.slice(0, 15))}
            ${bucketPanel('By size bucket', r.by_size)}
        </div>
        ${bucketPanel('By hour of day (ET)', r.by_hour_et)}

        <div class="chart-panel">
            <h2>Latest 50 fills</h2>
            ${sampleTable(r.samples.slice(0, 50))}
        </div>
    `;
}

function bucketPanel(title, rows) {
    return `<div class="chart-panel">
        <h2>${esc(title)}</h2>
        ${rows.length === 0
            ? '<p class="muted small">no data</p>'
            : `<table class="trades">
                <thead><tr>
                    <th>Bucket</th><th>N</th><th>Avg eff</th>
                    <th>Avg slip (bps)</th><th>Median</th><th>Worst</th>
                </tr></thead>
                <tbody>
                ${rows.map(b => {
                    const slipCls = b.avg_slippage_bps <= 5 ? 'pos' :
                                    b.avg_slippage_bps <= 25 ? '' : 'neg';
                    return `<tr>
                        <td>${esc(b.key)}</td>
                        <td>${b.samples}</td>
                        <td>${(b.avg_fill_efficiency * 100).toFixed(1)}%</td>
                        <td class="${slipCls}">${(b.avg_slippage_bps >= 0 ? '+' : '') + b.avg_slippage_bps.toFixed(1)}</td>
                        <td>${b.median_slippage_bps.toFixed(1)}</td>
                        <td class="neg">${b.worst_slippage_bps.toFixed(1)}</td>
                    </tr>`;
                }).join('')}
                </tbody>
            </table>`}
    </div>`;
}

function sampleTable(samples) {
    if (!samples.length) return '<p class="muted small">no fills</p>';
    return `<table class="trades">
        <thead><tr>
            <th>When</th><th>Symbol</th><th>Side</th><th>Qty</th>
            <th>Fill</th><th>Day O/H/L/C</th>
            <th>Typical</th><th>In-range</th><th>Eff</th><th>Slip bps</th>
        </tr></thead>
        <tbody>
        ${samples.map(s => {
            const slipCls = s.slippage_bps <= 5 ? 'pos' : s.slippage_bps <= 25 ? '' : 'neg';
            return `<tr>
                <td class="small">${new Date(s.executed_at).toLocaleString()}</td>
                <td>${esc(s.symbol)}</td>
                <td class="small">${esc(s.side)}</td>
                <td>${fmt(s.qty, 0)}</td>
                <td>${fmt(s.fill_price, s.fill_price < 10 ? 4 : 2)}</td>
                <td class="small muted">${fmt(s.bar_open, 2)}/${fmt(s.bar_high, 2)}/${fmt(s.bar_low, 2)}/${fmt(s.bar_close, 2)}</td>
                <td>${fmt(s.typical_price, 2)}</td>
                <td>${(s.fill_in_range * 100).toFixed(0)}%</td>
                <td class="${s.fill_efficiency >= 0.6 ? 'pos' : s.fill_efficiency >= 0.4 ? '' : 'neg'}">${(s.fill_efficiency * 100).toFixed(0)}%</td>
                <td class="${slipCls}">${(s.slippage_bps >= 0 ? '+' : '') + s.slippage_bps.toFixed(1)}</td>
            </tr>`;
        }).join('')}
        </tbody></table>`;
}
