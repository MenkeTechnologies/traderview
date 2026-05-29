// ABC correction pattern detector view — Elliott Wave style.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSwingsBlob, swingsToBlob, validateInputs, buildBody, localDetect,
    statusBadge, biasMixBadge, strengthBadge, summarizeSwings,
    makeDemoInput,
    fmtPrice, fmtRatio, fmtPct, fmtInt,
} from '../_abc_pattern_inputs.js';

let state = { ...makeDemoInput('bearish-classic') };

export async function renderAbcPattern(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.abc.h1.title" class="view-title">// ABC PATTERN</h1>

        <div class="chart-panel" data-context-scope="abc-pattern">
            <h2 data-i18n="view.abc.h2.swings">Swing pivots
                <small data-i18n="view.abc.h2.swings_hint" class="muted">(one swing per line: index price kind; kind = high|low)</small></h2>
            <textarea id="abc-blob" rows="6"
                      data-tip="view.abc.tip.swings"
                      placeholder="0 150 high\n10 130 low\n20 155 high">${esc(swingsToBlob(state.swings))}</textarea>

            <div class="inline-form">
                <label data-i18n="view.abc.label.min_b">Min B retrace</label>
                <input id="abc-min-b" type="number" step="0.01" min="0" max="1" value="${state.min_b_retrace}"
                       data-tip="view.abc.tip.min_b">
                <label data-i18n="view.abc.label.max_b">Max B retrace</label>
                <input id="abc-max-b" type="number" step="0.01" min="0" max="1" value="${state.max_b_retrace}"
                       data-tip="view.abc.tip.max_b">
                <label data-i18n="view.abc.label.min_c">Min C extension</label>
                <input id="abc-min-c" type="number" step="0.05" min="0.01" value="${state.min_c_extension}"
                       data-tip="view.abc.tip.min_c">
                <button data-i18n="view.abc.btn.compute" id="abc-run" class="primary"
                        data-tip="view.abc.tip.compute" type="button">Detect</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.abc.btn.demo_bear" id="abc-d1" class="secondary" type="button">Demo: bearish</button>
                <button data-i18n="view.abc.btn.demo_bull" id="abc-d2" class="secondary" type="button">Demo: bullish</button>
                <button data-i18n="view.abc.btn.demo_weak" id="abc-d3" class="secondary" type="button">Demo: weak C</button>
                <button data-i18n="view.abc.btn.demo_nonalt" id="abc-d4" class="secondary" type="button">Demo: non-alt</button>
                <button data-i18n="view.abc.btn.demo_multi" id="abc-d5" class="secondary" type="button">Demo: multi-event</button>
                <button data-i18n="view.abc.btn.demo_strong" id="abc-d6" class="secondary" type="button">Demo: very strong</button>
                <button data-i18n="view.abc.btn.demo_zero" id="abc-d7" class="secondary" type="button">Demo: zero leg</button>
                <button data-i18n="view.abc.btn.demo_tight" id="abc-d8" class="secondary" type="button">Demo: tight config</button>
            </div>
            <p data-i18n="view.abc.hint.about" class="muted">Detects 3-pivot ABC corrections (A high → B low → C high, or mirror). B's location must satisfy ab / (ab+bc) ∈ [min_b_retrace, max_b_retrace]; C must extend bc ≥ ab · min_c_extension.</p>
        </div>

        <div id="abc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.abc.h2.events">Detected ABC events</h2>
            <div id="abc-events"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.abc.h2.stats">Swing summary</h2>
            <div id="abc-stats"></div>
        </div>

        <div id="abc-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('abc-blob').value  = swingsToBlob(state.swings);
        document.getElementById('abc-min-b').value = state.min_b_retrace;
        document.getElementById('abc-max-b').value = state.max_b_retrace;
        document.getElementById('abc-min-c').value = state.min_c_extension;
    };
    document.getElementById('abc-d1').addEventListener('click', () => { loadDemo('bearish-classic'); void compute(tok); });
    document.getElementById('abc-d2').addEventListener('click', () => { loadDemo('bullish-classic'); void compute(tok); });
    document.getElementById('abc-d3').addEventListener('click', () => { loadDemo('weak-c');          void compute(tok); });
    document.getElementById('abc-d4').addEventListener('click', () => { loadDemo('non-alternating'); void compute(tok); });
    document.getElementById('abc-d5').addEventListener('click', () => { loadDemo('multi-events');    void compute(tok); });
    document.getElementById('abc-d6').addEventListener('click', () => { loadDemo('very-strong');     void compute(tok); });
    document.getElementById('abc-d7').addEventListener('click', () => { loadDemo('zero-leg');        void compute(tok); });
    document.getElementById('abc-d8').addEventListener('click', () => { loadDemo('tight-config');    void compute(tok); });
    document.getElementById('abc-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseSwingsBlob(document.getElementById('abc-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.abc.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    state.swings = p.swings;
    state.min_b_retrace   = Number(document.getElementById('abc-min-b').value);
    state.max_b_retrace   = Number(document.getElementById('abc-max-b').value);
    state.min_c_extension = Number(document.getElementById('abc-min-c').value);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localDetect(state.swings, state);
    renderSummary(local, true);
    renderEvents(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyAbcPattern(buildBody(state));
    } catch (e) {
        showErr(`${t('view.abc.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderEvents(resp);
    renderStats();
}

function renderSummary(report, pending) {
    const local = localDetect(state.swings, state);
    const parityOk = report
        && local.events.length === report.events.length
        && local.events.every((ev, i) => {
            const o = report.events[i];
            return ev.bias === o.bias
                && ev.a_idx === o.a_idx
                && ev.b_idx === o.b_idx
                && ev.c_idx === o.c_idx
                && Math.abs(ev.c_extension_ratio - o.c_extension_ratio) < 1e-9;
        });
    const sBadge = statusBadge(report);
    const mBadge = biasMixBadge(report);
    const last = report && report.events.length ? report.events[report.events.length - 1] : null;
    const strBadge = strengthBadge(last);
    const localTag = pending ? ` (${t('view.abc.tag.local')})` : '';
    document.getElementById('abc-summary').innerHTML = [
        card(t('view.abc.card.status'),    t(sBadge.key) + localTag, sBadge.cls),
        card(t('view.abc.card.mix'),       t(mBadge.key), mBadge.cls),
        card(t('view.abc.card.last_strength'), t(strBadge.key), strBadge.cls),
        card(t('view.abc.card.events'),    fmtInt(report ? report.events.length : 0)),
        card(t('view.abc.card.last_c_ext'), fmtRatio(last ? last.c_extension_ratio : NaN)),
        card(t('view.abc.card.last_ab'),    fmtPrice(last ? last.ab_length : NaN)),
        card(t('view.abc.card.last_bc'),    fmtPrice(last ? last.bc_length : NaN)),
        card(t('view.abc.card.parity'),
             parityOk ? t('view.abc.tag.ok') : t('view.abc.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderEvents(report) {
    const wrap = document.getElementById('abc-events');
    if (!report || !report.events.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.abc.no_events">${esc(t('view.abc.no_events'))}</div>`;
        return;
    }
    const rows = report.events.map((ev, i) => {
        const biasCls = ev.bias === 'bullish' ? 'pos' : 'neg';
        const biasKey = ev.bias === 'bullish' ? 'view.abc.bias.bullish' : 'view.abc.bias.bearish';
        return `<tr>
            <td>${i + 1}</td>
            <td class="${biasCls}">${esc(t(biasKey))}</td>
            <td>${fmtInt(ev.a_idx)}</td>
            <td>${fmtInt(ev.b_idx)}</td>
            <td>${fmtInt(ev.c_idx)}</td>
            <td>${esc(fmtPrice(ev.ab_length))}</td>
            <td>${esc(fmtPrice(ev.bc_length))}</td>
            <td>${esc(fmtPct(ev.b_retrace_pct))}</td>
            <td>${esc(fmtRatio(ev.c_extension_ratio))}</td>
        </tr>`;
    }).join('');
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th>
                <th data-i18n="view.abc.col.bias">Bias</th>
                <th>A</th><th>B</th><th>C</th>
                <th data-i18n="view.abc.col.ab">|AB|</th>
                <th data-i18n="view.abc.col.bc">|BC|</th>
                <th data-i18n="view.abc.col.bretrace">B retrace</th>
                <th data-i18n="view.abc.col.cext">C ext</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
    `;
}

function renderStats() {
    const wrap = document.getElementById('abc-stats');
    const s = summarizeSwings(state.swings);
    if (s.count === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.abc.empty">${esc(t('view.abc.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.abc.col.metric">Metric</th>
                <th data-i18n="view.abc.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.abc.row.count">Swings</td>        <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.abc.row.highs">Highs</td>         <td>${fmtInt(s.highs)}</td></tr>
                <tr><td data-i18n="view.abc.row.lows">Lows</td>           <td>${fmtInt(s.lows)}</td></tr>
                <tr><td data-i18n="view.abc.row.min">Min price</td>       <td>${esc(fmtPrice(s.min_price))}</td></tr>
                <tr><td data-i18n="view.abc.row.max">Max price</td>       <td>${esc(fmtPrice(s.max_price))}</td></tr>
                <tr><td data-i18n="view.abc.row.span">Span</td>           <td>${esc(fmtPrice(s.span))}</td></tr>
            </tbody>
        </table>
    `;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('abc-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('abc-err').style.display = 'none'; }
