// TWAP view — time-weighted execution analyzer.
//
// Complements VWAP Slippage: VWAP weights by volume (right tool for
// active aggressive orders), TWAP weights equally by time (right tool
// for passive limit working orders where time-in-market matters more
// than volume-participation).

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseTypicals, validateInputs, buildBody,
    localTwap, rollingTwap, decToNum, unwrapResponse,
    makeDemoData, fmtN, fmtBps,
} from '../_twap_inputs.js';

let state = { side: 'long', fillPrice: 100, typicalsText: '' };

export async function renderTwap(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// TWAP · TIME-WEIGHTED EXECUTION</h1>

        <div class="chart-panel">
            <h2>Trade</h2>
            <div class="inline-form">
                <label>Side
                    <select id="tw-side">
                        <option value="long"  ${state.side === 'long'  ? 'selected' : ''}>Long (buy entry)</option>
                        <option value="short" ${state.side === 'short' ? 'selected' : ''}>Short (sell entry)</option>
                    </select></label>
                <label>Fill price
                    <input id="tw-fill" type="number" step="any" min="0" value="${state.fillPrice}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Typical prices over the exposure window</h2>
            <p class="muted">One value per line. Typical = (H+L+C)/3 from each bar.
                Demo loads 200 typicals with a long fill engineered to beat the mean.</p>
            <textarea id="tw-typ" rows="6" placeholder="100.05&#10;100.08&#10;..."></textarea>
            <div class="inline-form">
                <button id="tw-demo" class="secondary" type="button">Load demo (200 prices, fill beats TWAP)</button>
                <button id="tw-clear" class="secondary" type="button">Clear</button>
                <button id="tw-run" class="primary" type="button">Analyze</button>
            </div>
        </div>

        <div id="tw-errors" class="boot" style="display:none"></div>
        <div id="tw-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Typical price + rolling TWAP + fill reference</h2>
            <div id="tw-chart" style="height:280px"></div>
            <p class="muted">Cyan = typical. Yellow = rolling TWAP (arithmetic mean to bar i).
                Magenta dashed = fill. Long entries want magenta below yellow; shorts the inverse.</p>
        </div>

        <div id="tw-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('tw-demo').addEventListener('click', () => {
        const { side, fill_price, typicals } = makeDemoData(42);
        document.getElementById('tw-side').value = side;
        document.getElementById('tw-fill').value = fill_price;
        document.getElementById('tw-typ').value = typicals.join('\n');
    });
    document.getElementById('tw-clear').addEventListener('click', () => {
        document.getElementById('tw-typ').value = '';
    });
    document.getElementById('tw-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.side = document.getElementById('tw-side').value;
    state.fillPrice = Number(document.getElementById('tw-fill').value);
    state.typicalsText = document.getElementById('tw-typ').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('tw-errors');
    errs.style.display = 'none';
    const { value: typicals, errors } = parseTypicals(state.typicalsText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (typicals.length === 0) return;
    }
    const err = validateInputs(state.side, state.fillPrice, typicals);
    if (err) { showErr(err); return; }
    let resp;
    try {
        resp = await api.microTwap(buildBody(state.side, state.fillPrice, typicals));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    const unwrapped = unwrapResponse(resp);
    if (!unwrapped.ok) { showErr(`Backend: ${unwrapped.reason}`); return; }
    renderSummary(unwrapped.result, typicals);
    renderChart(typicals, decToNum(unwrapped.result.twap), state.fillPrice);
}

function renderSummary(r, typicals) {
    const twap = decToNum(r.twap);
    const fill = decToNum(r.fill_price);
    const slipDollars = decToNum(r.slippage_dollars);
    const localChk = localTwap(typicals);
    document.getElementById('tw-summary').innerHTML = [
        card('TWAP (backend)', fmtN(twap)),
        card('TWAP (local)',   fmtN(localChk),
            Math.abs(twap - localChk) < 1e-6 ? 'pos' : 'neg'),
        card('Fill price',     fmtN(fill)),
        card('Slippage $',     fmtN(slipDollars),
            slipDollars > 0 ? 'pos' : slipDollars < 0 ? 'neg' : ''),
        card('Slippage bps',   fmtBps(r.slippage_bps),
            r.slippage_bps > 0 ? 'pos' : r.slippage_bps < 0 ? 'neg' : ''),
        card('Beat TWAP?',     r.beat_twap ? 'YES' : 'NO',
            r.beat_twap ? 'pos' : 'neg'),
        card('Bars',           String(typicals.length)),
        card('Mean−Fill Δ',    fmtN(localChk - state.fillPrice)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(typicals, twap, fillPrice) {
    if (!window.uPlot) return;
    const el = document.getElementById('tw-chart');
    const xs = typicals.map((_, i) => i);
    const roll = rollingTwap(typicals);
    const twapYs = xs.map(() => twap);
    const fillYs = xs.map(() => fillPrice);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar #' },
            { label: 'typical',      stroke: '#00e5ff', width: 1.0,
              fill: '#00e5ff14', points: { show: false } },
            { label: 'rolling TWAP', stroke: '#ffd84a', width: 1.2,
              points: { show: false } },
            { label: 'final TWAP',   stroke: '#ff9f1a', width: 1.0,
              dash: [4, 4], points: { show: false } },
            { label: 'fill',         stroke: '#ff3860', width: 1.0,
              dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, typicals, roll, twapYs, fillYs], el);
}

function showErr(msg) {
    const el = document.getElementById('tw-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('tw-err').style.display = 'none'; }
