// Footprint chart view — Sierra Chart-class per-bar bid/ask volume +
// delta visualization. The Bookmap / ATAS / Jigsaw display: each
// price-time bar renders as a stacked column of cells (one per price
// level) showing `bid_vol × ask_vol`, with the delta color-coded.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { api } from '../api.js';
import {
    parseTickBlob, validateInputs, buildBody,
    deltaCls, summarize, imbalanceHotspots,
    makeDemoTicks, fmtN, fmtPrice, fmtSigned,
} from '../_footprint_inputs.js';

let state = { tickText: '', tickSize: 0.05 };

export async function renderFootprint(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// FOOTPRINT · BID/ASK PER PRICE LEVEL</h1>

        <div class="chart-panel">
            <h2>Classified tick stream</h2>
            <p class="muted">One tick per line: <code>bar_id price volume side</code>
                where side ∈ {buy, sell, uncertain}. Demo loads 4 bars with engineered
                patterns: balanced churn → absorption at low → drive up → rejection at high.</p>
            <textarea id="fp-ticks" rows="8" placeholder="0 100.00 50 buy&#10;0 100.00 50 sell&#10;..."></textarea>
            <div class="inline-form">
                <label>Tick size (price quantization)
                    <input id="fp-ts" type="number" step="any" min="0" value="${state.tickSize}"></label>
                <button id="fp-demo" class="secondary" type="button">Load demo (4 bars, 4 patterns)</button>
                <button id="fp-clear" class="secondary" type="button">Clear</button>
                <button id="fp-run" class="primary" type="button">Build footprint</button>
            </div>
        </div>

        <div id="fp-errors" class="boot" style="display:none"></div>
        <div id="fp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Footprint bars</h2>
            <div id="fp-grid" class="fp-grid"></div>
            <p class="muted">Each column = one bar. Each row = one price level. Cells show
                <strong>bid × ask</strong>; row color is the per-cell delta (green = ask
                won, red = bid won). Gold-highlighted row = bar POC (most-traded level).</p>
        </div>

        <div class="chart-panel">
            <h2>Imbalance hotspots (largest abs(delta) cells)</h2>
            <div id="fp-hotspots"></div>
        </div>

        <div id="fp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('fp-demo').addEventListener('click', () => {
        const t = makeDemoTicks();
        document.getElementById('fp-ticks').value =
            t.map(x => `${x.bar_id} ${x.price} ${x.classified.volume} ${x.classified.side}`).join('\n');
    });
    document.getElementById('fp-clear').addEventListener('click', () => {
        document.getElementById('fp-ticks').value = '';
    });
    document.getElementById('fp-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.tickText = document.getElementById('fp-ticks').value;
    state.tickSize = Number(document.getElementById('fp-ts').value);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('fp-errors');
    errs.style.display = 'none';
    const { ticks, errors } = parseTickBlob(state.tickText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (ticks.length === 0) return;
    }
    const err = validateInputs(ticks, state.tickSize);
    if (err) { showErr(err); return; }
    let report;
    try {
        report = await api.microFootprint(buildBody(ticks, state.tickSize));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report);
    renderGrid(report);
    renderHotspots(report);
}

function renderSummary(r) {
    const s = summarize(r);
    document.getElementById('fp-summary').innerHTML = [
        card('Bars',         String(s.barCount)),
        card('Total volume', fmtN(s.totalVolume)),
        card('Net delta',    fmtSigned(s.totalDelta), s.totalDelta >= 0 ? 'pos' : 'neg'),
        card('Max |bar delta|', fmtN(s.maxAbsDelta)),
        card('Last POC',     s.lastPoc != null ? fmtPrice(s.lastPoc, state.tickSize) : '—'),
        card('Tick size',    fmtN(state.tickSize, 4)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderGrid(report) {
    const wrap = document.getElementById('fp-grid');
    const bars = (report && report.bars) || [];
    if (!bars.length) { wrap.innerHTML = '<div class="muted">No bars.</div>'; return; }
    // Build a UNION price-axis spanning every level seen across all bars
    // so each bar column aligns on the same vertical row grid.
    const priceSet = new Set();
    for (const b of bars) for (const c of (b.cells || [])) priceSet.add(c.price);
    const allPrices = [...priceSet].sort((a, b) => b - a);  // top-down (highest first)

    const headerCells = bars.map(b => `<th>Bar ${b.bar_id}</th>`).join('');
    const rowsHtml = allPrices.map(p => {
        const cells = bars.map(b => {
            const cell = (b.cells || []).find(c => c.price === p);
            if (!cell) return `<td class="fp-cell fp-empty"></td>`;
            const isPoc = b.poc_price === p;
            const cls = `fp-cell ${deltaCls(cell.delta)} ${isPoc ? 'fp-poc' : ''}`;
            return `<td class="${cls}">
                <span class="fp-bid">${esc(fmtN(cell.bid_volume))}</span>
                <span class="fp-ask">${esc(fmtN(cell.ask_volume))}</span>
                <span class="fp-delta">${esc(fmtSigned(cell.delta))}</span>
            </td>`;
        }).join('');
        return `<tr><th>${esc(fmtPrice(p, state.tickSize))}</th>${cells}</tr>`;
    }).join('');
    const footerCells = bars.map(b => `<th>
        <div class="fp-foot-vol">${esc(fmtN(b.total_volume))} vol</div>
        <div class="fp-foot-delta ${deltaCls(b.total_delta)}">${esc(fmtSigned(b.total_delta))} Δ</div>
        <div class="fp-foot-poc">POC ${esc(fmtPrice(b.poc_price, state.tickSize))}</div>
    </th>`).join('');
    wrap.innerHTML = `
        <table class="fp-table">
            <thead><tr><th>Price</th>${headerCells}</tr></thead>
            <tbody>${rowsHtml}</tbody>
            <tfoot><tr><th></th>${footerCells}</tr></tfoot>
        </table>
    `;
}

function renderHotspots(report) {
    const wrap = document.getElementById('fp-hotspots');
    const hots = imbalanceHotspots(report, 8);
    if (!hots.length) { wrap.innerHTML = '<div class="muted">No cells.</div>'; return; }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th>Bar</th><th>Price</th><th>Bid vol</th>
                <th>Ask vol</th><th>Δ</th>
            </tr></thead>
            <tbody>
                ${hots.map((h, i) => `<tr>
                    <td>${i + 1}</td>
                    <td>${h.bar_id}</td>
                    <td>${esc(fmtPrice(h.price, state.tickSize))}</td>
                    <td>${esc(fmtN(h.bid))}</td>
                    <td>${esc(fmtN(h.ask))}</td>
                    <td class="${deltaCls(h.delta)}">${esc(fmtSigned(h.delta))}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('fp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('fp-err').style.display = 'none'; }
