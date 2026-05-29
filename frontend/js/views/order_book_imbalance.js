// Order Book Imbalance view — single-snapshot directional-pressure gauge.
//
// Inputs:
//   bid_sizes  — top-of-book bid sizes (level 1 first)
//   ask_sizes  — top-of-book ask sizes (level 1 first)
//   levels     — how many top levels to aggregate
//
// Visualizes the [-1, 1] imbalance scalar as a divergent bar (cyan bid /
// magenta ask), and the raw per-level bid vs ask sizes as a side-by-side
// table so the trader can see WHICH level dominates.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSizes, validateInputs, buildBody,
    alignLevels, biasBadge, makeDemoBook,
    fmtN, fmtImbalance,
} from '../_order_book_imbalance_inputs.js';

import { t } from '../i18n.js';
let state = { bidText: '', askText: '', levels: 5 };

export async function renderOrderBookImbalance(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.order_book_imbalance.h1.order_book_imbalance" class="view-title">// ORDER-BOOK IMBALANCE</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.order_book_imbalance.h2.bid_sizes_top_of_book_first">Bid sizes (top-of-book first)</h2>
            <textarea id="obi-bid" rows="4" placeholder="500&#10;380&#10;290&#10;220&#10;180"></textarea>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.order_book_imbalance.h2.ask_sizes">Ask sizes</h2>
            <textarea id="obi-ask" rows="4" placeholder="120&#10;100&#10;80&#10;60&#10;50"></textarea>
            <div class="inline-form">
                <label><span data-i18n="view.order_book_imbalance.label.levels">Levels (top-N to aggregate)</span>
                    <input id="obi-lvls" type="number" step="1" min="1" max="50" value="${state.levels}"></label>
                <button data-i18n="view.order_book_imbalance.btn.demo_balanced" id="obi-balanced" class="secondary" type="button">Demo: balanced</button>
                <button data-i18n="view.order_book_imbalance.btn.demo_bid_pressure" id="obi-bidp" class="secondary" type="button">Demo: bid pressure</button>
                <button data-i18n="view.order_book_imbalance.btn.demo_ask_pressure" id="obi-askp" class="secondary" type="button">Demo: ask pressure</button>
                <button data-i18n="view.order_book_imbalance.btn.clear" id="obi-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.order_book_imbalance.btn.compute" id="obi-run" class="primary" type="button">Compute</button>
            </div>
            <p data-i18n="view.order_book_imbalance.hint.imbalance_bid_ask_bid_ask_over_top_n_levels_range_" class="muted">Imbalance = (Σbid − Σask) / (Σbid + Σask) over top-N levels.
                Range [-1, 1]. Used by HFT firms as a microsecond-scale directional signal.</p>
        </div>

        <div id="obi-errors" class="boot" style="display:none"></div>
        <div id="obi-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.order_book_imbalance.h2.imbalance_gauge">Imbalance gauge</h2>
            <div id="obi-gauge"></div>
            <p data-i18n="view.order_book_imbalance.hint.cyan_bid_skew_magenta_ask_skew_midline_balanced_qu" class="muted">Cyan = bid skew. Magenta = ask skew. Midline = balanced.
                Quartile marks at ±0.1 (bid/ask) and ±0.3 (strong).</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.order_book_imbalance.h2.per_level_breakdown">Per-level breakdown</h2>
            <div id="obi-table"></div>
        </div>

        <div id="obi-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        const { bid_sizes, ask_sizes } = makeDemoBook(kind);
        document.getElementById('obi-bid').value = bid_sizes.join('\n');
        document.getElementById('obi-ask').value = ask_sizes.join('\n');
    };
    document.getElementById('obi-balanced').addEventListener('click', () => loadDemo('balanced'));
    document.getElementById('obi-bidp').addEventListener('click', () => loadDemo('bid-pressure'));
    document.getElementById('obi-askp').addEventListener('click', () => loadDemo('ask-pressure'));
    document.getElementById('obi-clear').addEventListener('click', () => {
        document.getElementById('obi-bid').value = '';
        document.getElementById('obi-ask').value = '';
    });
    document.getElementById('obi-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.bidText = document.getElementById('obi-bid').value;
    state.askText = document.getElementById('obi-ask').value;
    state.levels = parseInt(document.getElementById('obi-lvls').value, 10);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('obi-errors');
    errs.style.display = 'none';
    const { value: bidSizes, errors: bidErrs } = parseSizes(state.bidText);
    const { value: askSizes, errors: askErrs } = parseSizes(state.askText);
    const allErrs = [
        ...bidErrs.map(e => ({ ...e, src: 'bid' })),
        ...askErrs.map(e => ({ ...e, src: 'ask' })),
    ];
    if (allErrs.length) {
        const head = allErrs.slice(0, 8).map(e =>
            `[${e.src}] line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = allErrs.length > 8 ? `<br>… and ${allErrs.length - 8} more.` : '';
        errs.innerHTML = `<strong>${allErrs.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
    }
    const err = validateInputs(bidSizes, askSizes, state.levels);
    if (err) { showErr(err); return; }
    let res;
    try {
        res = await api.microOrderBookImbalance(buildBody(bidSizes, askSizes, state.levels));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res, bidSizes.length, askSizes.length);
    renderGauge(res);
    renderTable(bidSizes, askSizes, state.levels);
}

function renderSummary(r, bidLevels, askLevels) {
    const badge = biasBadge(r.bias);
    const ratio = r.total_ask_size > 0 ? r.total_bid_size / r.total_ask_size : Infinity;
    document.getElementById('obi-summary').innerHTML = [
        card(t('view.order_book_imbalance.card.imbalance'),    fmtImbalance(r.imbalance), badge.cls),
        card(t('view.order_book_imbalance.card.bias'),         badge.label, badge.cls),
        card(t('view.order_book_imbalance.card.bid_total'),    fmtN(r.total_bid_size), 'pos'),
        card(t('view.order_book_imbalance.card.ask_total'),    fmtN(r.total_ask_size), 'neg'),
        card(t('view.order_book_imbalance.card.bid_ask_ratio'), Number.isFinite(ratio) ? ratio.toFixed(3) : '∞'),
        card(t('view.order_book_imbalance.card.levels_seen'),  `${Math.min(bidLevels, state.levels)} bid / ${Math.min(askLevels, state.levels)} ask`),
        card(t('view.order_book_imbalance.card.action'),       badge.hint),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderGauge(r) {
    const wrap = document.getElementById('obi-gauge');
    const imb = Math.max(-1, Math.min(1, r.imbalance || 0));
    const halfPct = (Math.abs(imb) * 50).toFixed(2);
    const sideClass = imb >= 0 ? 'is-fill-pos obi-fill-bid' : 'is-fill-neg obi-fill-ask';
    wrap.innerHTML = `
        <div class="is-bar-row">
            <div class="is-bar-label">imbalance</div>
            <div class="is-bar-track">
                <div class="is-bar-midline"></div>
                <div class="is-bar-midline obi-q-neg-strong"></div>
                <div class="is-bar-midline obi-q-neg"></div>
                <div class="is-bar-midline obi-q-pos"></div>
                <div class="is-bar-midline obi-q-pos-strong"></div>
                <div class="is-bar-fill ${sideClass}" data-bar-pct="${halfPct}"></div>
            </div>
            <div class="is-bar-value">${esc(fmtImbalance(r.imbalance))}</div>
        </div>
    `;
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.is-bar-fill').forEach(el => {
            const pct = Number(el.dataset.barPct);
            if (Number.isFinite(pct)) el.style.width = pct + '%';
        });
    });
}

function renderTable(bidSizes, askSizes, levels) {
    const rows = alignLevels(bidSizes, askSizes, levels);
    const maxSize = Math.max(...rows.map(r => Math.max(r.bid, r.ask)), 1);
    const wrap = document.getElementById('obi-table');
    if (!rows.length) { wrap.innerHTML = `<div class="muted" data-i18n="view.order_book_imbalance.empty.levels">No levels.</div>`; return; }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.order_book_imbalance.th.level">Level</th>
                <th data-i18n="view.order_book_imbalance.th.bid" style="text-align:right">Bid</th>
                <th data-i18n="view.order_book_imbalance.th.bid_bar">Bid bar</th>
                <th data-i18n="view.order_book_imbalance.th.ask_bar">Ask bar</th>
                <th data-i18n="view.order_book_imbalance.th.ask">Ask</th>
            </tr></thead>
            <tbody>
                ${rows.map(r => {
                    const bidPct = (r.bid / maxSize * 100).toFixed(2);
                    const askPct = (r.ask / maxSize * 100).toFixed(2);
                    return `<tr>
                        <td>${r.level}</td>
                        <td style="text-align:right" class="pos">${esc(fmtN(r.bid))}</td>
                        <td><div class="obi-mini-bar-track"><div class="obi-mini-bar obi-fill-bid" data-bar-pct="${bidPct}"></div></div></td>
                        <td><div class="obi-mini-bar-track"><div class="obi-mini-bar obi-fill-ask" data-bar-pct="${askPct}"></div></div></td>
                        <td class="neg">${esc(fmtN(r.ask))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.obi-mini-bar').forEach(el => {
            const pct = Number(el.dataset.barPct);
            if (Number.isFinite(pct)) el.style.width = pct + '%';
        });
    });
}

function showErr(msg) {
    const el = document.getElementById('obi-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('obi-err').style.display = 'none'; }
