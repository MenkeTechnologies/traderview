// VWAP Slippage view — institutional TCA "did I beat VWAP?" benchmark.
//
// Long fill BELOW VWAP = positive slippage (you got a discount on entry).
// Short fill ABOVE VWAP = positive (you got premium on entry).

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, validateInputs, buildBody,
    localVwap, rollingVwap, unwrapResponse, decToNum,
    makeDemoData, fmtN, fmtBps, fmtVol,
} from '../_vwap_slippage_inputs.js';

let state = { side: 'long', fillPrice: 100, barText: '' };

export async function renderVwapSlippage(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// VWAP SLIPPAGE · TCA BENCHMARK</h1>

        <div class="chart-panel">
            <h2>Trade</h2>
            <div class="inline-form">
                <label>Side
                    <select id="vw-side">
                        <option value="long"  ${state.side === 'long'  ? 'selected' : ''}>Long (buy entry)</option>
                        <option value="short" ${state.side === 'short' ? 'selected' : ''}>Short (sell entry)</option>
                    </select></label>
                <label>Fill price
                    <input id="vw-fill" type="number" step="any" min="0" value="${state.fillPrice}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Bars during the trade's open window</h2>
            <p class="muted">One line per bar: <code>typical_price volume</code>.
                Typical = (high+low+close)/3. Pre-computed by the caller so this
                view stays agnostic about whether you're feeding 1-second, 1-minute,
                or 1-hour bars. Demo loads 200 bars with an intentional below-VWAP long fill.</p>
            <textarea id="vw-bars" rows="6" placeholder="100.05 1200&#10;100.08 850&#10;..."></textarea>
            <div class="inline-form">
                <button id="vw-demo" class="secondary" type="button">Load demo (200 bars, fill beats VWAP)</button>
                <button id="vw-clear" class="secondary" type="button">Clear</button>
                <button id="vw-run" class="primary" type="button">Analyze</button>
            </div>
        </div>

        <div id="vw-errors" class="boot" style="display:none"></div>
        <div id="vw-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Typical price + rolling VWAP + fill reference</h2>
            <div id="vw-chart" style="height:280px"></div>
            <p class="muted">Cyan = typical price per bar. Yellow = rolling VWAP (the
                benchmark). Magenta dashed = your fill price. For LONG entries you want
                magenta BELOW yellow at trade close; for SHORT entries you want it ABOVE.</p>
        </div>

        <div id="vw-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('vw-demo').addEventListener('click', () => {
        const { side, fill_price, bars } = makeDemoData(42);
        document.getElementById('vw-side').value = side;
        document.getElementById('vw-fill').value = fill_price;
        document.getElementById('vw-bars').value =
            bars.map(b => `${b.typical} ${b.volume}`).join('\n');
    });
    document.getElementById('vw-clear').addEventListener('click', () => {
        document.getElementById('vw-bars').value = '';
    });
    document.getElementById('vw-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.side = document.getElementById('vw-side').value;
    state.fillPrice = Number(document.getElementById('vw-fill').value);
    state.barText = document.getElementById('vw-bars').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('vw-errors');
    errs.style.display = 'none';
    const { bars, errors } = parseBarBlob(state.barText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (bars.length === 0) return;
    }
    const err = validateInputs(state.side, state.fillPrice, bars);
    if (err) { showErr(err); return; }
    let resp;
    try {
        resp = await api.microVwapSlippage(buildBody(state.side, state.fillPrice, bars));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    const unwrapped = unwrapResponse(resp);
    if (!unwrapped.ok) { showErr(`Backend: ${unwrapped.reason}`); return; }
    renderSummary(unwrapped.result, bars);
    renderChart(bars, decToNum(unwrapped.result.vwap), state.fillPrice);
}

function renderSummary(r, bars) {
    const vwap = decToNum(r.vwap);
    const fill = decToNum(r.fill_price);
    const slipDollars = decToNum(r.slippage_dollars);
    const localChk = localVwap(bars);
    const totalVol = bars.reduce((a, b) => a + (b.volume || 0), 0);
    document.getElementById('vw-summary').innerHTML = [
        card('VWAP (backend)', fmtN(vwap)),
        card('VWAP (local)',   fmtN(localChk),
            Math.abs(vwap - localChk) < 1e-6 ? 'pos' : 'neg'),
        card('Fill price',     fmtN(fill)),
        card('Slippage $',     fmtN(slipDollars),
            slipDollars > 0 ? 'pos' : slipDollars < 0 ? 'neg' : ''),
        card('Slippage bps',   fmtBps(r.slippage_bps),
            r.slippage_bps > 0 ? 'pos' : r.slippage_bps < 0 ? 'neg' : ''),
        card('Beat VWAP?',     r.beat_vwap ? 'YES' : 'NO',
            r.beat_vwap ? 'pos' : 'neg'),
        card('Bars',           String(bars.length)),
        card('Total volume',   fmtVol(totalVol)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(bars, vwap, fillPrice) {
    if (!window.uPlot) return;
    const el = document.getElementById('vw-chart');
    const xs = bars.map((_, i) => i);
    const typ = bars.map(b => b.typical);
    const roll = rollingVwap(bars);
    const vwapYs = xs.map(() => vwap);
    const fillYs = xs.map(() => fillPrice);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar #' },
            { label: 'typical', stroke: '#00e5ff', width: 1.0,
              fill: '#00e5ff14', points: { show: false } },
            { label: 'rolling VWAP', stroke: '#ffd84a', width: 1.2,
              points: { show: false } },
            { label: 'final VWAP', stroke: '#ff9f1a', width: 1.0,
              dash: [4, 4], points: { show: false } },
            { label: 'fill', stroke: '#ff3860', width: 1.0,
              dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, typ, roll, vwapYs, fillYs], el);
}

function showErr(msg) {
    const el = document.getElementById('vw-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('vw-err').style.display = 'none'; }
