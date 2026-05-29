// Signal Decomposition view — paste a series, pick EMD / Wavelet / SSA,
// see the components stacked as separate subplots.
//
// Each method yields a different component list (IMFs+residual for EMD,
// details+approximation for Wavelet, trend+noise for SSA). The helper
// normalizes them into `[{ label, color, data }]` so the renderer is
// method-agnostic.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    METHODS, parseSeries, validateInputs, defaultOpts,
    reconstructionResidual,
} from '../_signal_decomposition_inputs.js';

const DEFAULT_TEXT = `# Demo: 128 points of trend + 2 cycles + noise. Pick a method below.
${synthDemoSeries(128).join('\n')}
`;

function synthDemoSeries(n) {
    let s = 0xFEED_BEEFn;
    const rand = () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(s >> 11n) / 2 ** 53;
    };
    const out = [];
    for (let i = 0; i < n; i++) {
        const trend  = 100 + 0.1 * i;
        const fast   = 2.0 * Math.sin(i * 0.5);
        const slow   = 5.0 * Math.sin(i * 0.05);
        const noise  = (rand() - 0.5) * 1.5;
        out.push((trend + fast + slow + noise).toFixed(3));
    }
    return out;
}

let state = {
    text: DEFAULT_TEXT,
    methodId: 'emd',
    opts: defaultOpts('emd'),
};

export async function renderSignalDecomposition(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 class="view-title">// SIGNAL DECOMPOSITION</h1>

        <div class="chart-panel">
            <h2>Method</h2>
            <div class="inline-form">
                <label>Decomposition
                    <select id="sd-method">
                        ${Object.entries(METHODS).map(([id, m]) =>
                            `<option value="${id}" ${id === state.methodId ? 'selected' : ''}>${esc(m.label)}</option>`
                        ).join('')}
                    </select></label>
                <button id="sd-run" class="primary" type="button">Decompose</button>
            </div>
            <div id="sd-opts" class="inline-form" style="margin-top:8px"></div>
        </div>

        <div class="chart-panel">
            <h2>Series</h2>
            <textarea id="sd-text" rows="8"
                style="width:100%;font-family:monospace;font-size:13px">${esc(state.text)}</textarea>
        </div>

        <div id="sd-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="sd-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Components</h2>
            <div id="sd-components"></div>
            <p class="muted">
                Each subplot is one component. For EMD: high-freq IMFs first, residual last.
                For Wavelet: detail levels first, smooth approximation last. For SSA: trend
                on top, noise residual below.
            </p>
        </div>

        <div id="sd-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    renderOptsForm();
    wireForm(mount, tok);
    void fmt;
}

function renderOptsForm() {
    const wrap = document.getElementById('sd-opts');
    const method = METHODS[state.methodId];
    wrap.innerHTML = method.fields.map(f => {
        const v = state.opts[f.key];
        const step = f.integer ? '1' : 'any';
        const min = f.min != null ? `min="${f.min}"` : '';
        const max = f.max != null ? `max="${f.max}"` : '';
        return `<label>${esc(f.label)}
            <input type="number" step="${step}" ${min} ${max}
                   value="${v}" data-opt="${esc(f.key)}"></label>`;
    }).join('');
    wrap.querySelectorAll('input[data-opt]').forEach(el => {
        el.addEventListener('change', e => {
            const k = e.target.dataset.opt;
            const f = method.fields.find(x => x.key === k);
            state.opts[k] = f.integer ? parseInt(e.target.value, 10) : Number(e.target.value);
        });
    });
}

function wireForm(mount, tok) {
    document.getElementById('sd-method').addEventListener('change', e => {
        state.methodId = e.target.value;
        state.opts = defaultOpts(state.methodId);
        renderOptsForm();
        document.getElementById('sd-components').innerHTML = '';
        document.getElementById('sd-summary').innerHTML = '';
    });
    document.getElementById('sd-run').addEventListener('click', () => {
        state.text = document.getElementById('sd-text').value;
        void decompose(mount, tok);
    });
}

async function decompose(mount, tok) {
    hideErrs();
    const parsed = parseSeries(state.text);
    if (parsed.errors.length) renderParseErrors(parsed.errors);

    const err = validateInputs(state.methodId, parsed.value, state.opts);
    if (err) { showErr(err); return; }

    const method = METHODS[state.methodId];
    let res;
    try {
        res = await api[method.endpoint](method.buildBody(parsed.value, state.opts));
        if (!res) throw new Error('decomposition returned null (input out of domain)');
    } catch (e) {
        showErr(`API error: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const components = method.toComponents(res);
    if (!components || components.length === 0) {
        showErr('backend returned no components');
        return;
    }

    renderSummary(parsed.value, components, res);
    renderComponents(parsed.value, components);
}

function renderSummary(series, components, res) {
    const cards = [];
    cards.push(card('Series length', String(series.length)));
    cards.push(card('Components', String(components.length)));
    if (state.methodId === 'emd' && Array.isArray(res.iterations)) {
        const totalIter = res.iterations.reduce((a, b) => a + b, 0);
        cards.push(card('Total sift iterations', String(totalIter)));
    }
    if (state.methodId === 'wavelet' && res.used_length != null) {
        cards.push(card('Used length (2^L)', String(res.used_length)));
    }
    if (state.methodId === 'ssa' && Array.isArray(res.singular_values)) {
        const top = res.singular_values[0] ?? NaN;
        const total = res.singular_values.reduce((a, b) => a + b, 0);
        const topPct = total > 0 ? (top / total * 100) : NaN;
        cards.push(card('1st singular value share',
            Number.isFinite(topPct) ? `${topPct.toFixed(1)}%` : '—'));
    }
    const recon = reconstructionResidual(series, components);
    if (recon != null) {
        cards.push(card('Max reconstruction error',
            recon < 1e-9 ? '< 1e-9 (exact)' : recon.toExponential(2)));
    }
    document.getElementById('sd-summary').innerHTML = cards.join('');
}

function card(label, value) {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value">${esc(value)}</div>
    </div>`;
}

function renderComponents(series, components) {
    const wrap = document.getElementById('sd-components');
    // Pre-create a chart container per component + one for the original
    // series at the top.
    wrap.innerHTML = `
        <div class="sd-row">
            <div class="sd-row-label">original</div>
            <div id="sd-chart-orig" class="sd-chart-cell"></div>
        </div>
        ${components.map((c, i) =>
            `<div class="sd-row">
                <div class="sd-row-label">${esc(c.label)}</div>
                <div id="sd-chart-${i}" class="sd-chart-cell"></div>
            </div>`
        ).join('')}
    `;
    if (!window.uPlot) return;
    drawMini('sd-chart-orig', series, '#aab');
    components.forEach((c, i) => drawMini(`sd-chart-${i}`, c.data, c.color));
}

function drawMini(elId, ys, stroke) {
    const el = document.getElementById(elId);
    if (!el || !Array.isArray(ys) || ys.length === 0) return;
    const xs = ys.map((_, i) => i);
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 80,
        scales: { x: {}, y: {} },
        series: [
            { label: 'idx' },
            { label: 'value', stroke, width: 1.5,
              fill: stroke === '#aab' ? undefined : `${stroke}1A`,
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 30 },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: false },
    }, [xs, ys], el);
}

function renderParseErrors(errors) {
    const el = document.getElementById('sd-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">… (+${errors.length - 20} more)</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('sd-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('sd-parse-errors').style.display = 'none';
    document.getElementById('sd-err').style.display = 'none';
}
