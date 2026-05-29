// Wash-sale (§1091) detector view. For each LOSING closing trade,
// finds every replacement BUY of the same symbol within ±30 days and
// reports the disallowed-loss estimate.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseClosingBlob, parseOpeningBlob, validateInputs, buildBody,
    localDetectHits, localTotalDisallowed, dec,
    washBadge, totalRealizedLoss,
    makeDemoClosings, makeDemoOpenings,
    fmtUSD, fmtDays, fmtPct, fmtNum, shortUuid,
} from '../_wash_sale_inputs.js';

let state = {
    closings: makeDemoClosings('classic-trap'),
    openings: makeDemoOpenings('classic-trap'),
};

export async function renderWashSale(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.wash_sale.h1.title" class="view-title">// WASH SALE DETECTOR</h1>

        <div class="chart-panel" data-context-scope="wash-sale">
            <h2 data-i18n="view.wash_sale.h2.closings">Closed trades
                <small data-i18n="view.wash_sale.h2.closings_hint" class="muted">(per line: SYMBOL YYYY-MM-DD net_pnl qty; negative pnl = loss)</small></h2>
            <textarea id="ws-close" rows="5"
                      data-tip="view.wash_sale.tip.closings"
                      placeholder="AAPL 2026-06-01 -500 100">${esc(closingsToBlob(state.closings))}</textarea>

            <h2 data-i18n="view.wash_sale.h2.openings">Opening executions
                <small data-i18n="view.wash_sale.h2.openings_hint" class="muted">(per line: SYMBOL YYYY-MM-DD qty)</small></h2>
            <textarea id="ws-open" rows="4"
                      data-tip="view.wash_sale.tip.openings"
                      placeholder="AAPL 2026-06-15 100">${esc(openingsToBlob(state.openings))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.wash_sale.btn.detect" id="ws-run" class="primary"
                        data-tip="view.wash_sale.tip.detect" type="button">Detect</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.wash_sale.btn.demo_classic"  id="ws-demo-classic"  class="secondary" type="button">Demo: classic trap (+14d buyback)</button>
                <button data-i18n="view.wash_sale.btn.demo_winning"  id="ws-demo-win"      class="secondary" type="button">Demo: winning trade (no flag)</button>
                <button data-i18n="view.wash_sale.btn.demo_outside"  id="ws-demo-outside"  class="secondary" type="button">Demo: outside window (+34d)</button>
                <button data-i18n="view.wash_sale.btn.demo_partial"  id="ws-demo-partial"  class="secondary" type="button">Demo: partial replacement (30 / 100)</button>
                <button data-i18n="view.wash_sale.btn.demo_multi"    id="ws-demo-multi"    class="secondary" type="button">Demo: multiple buybacks</button>
                <button data-i18n="view.wash_sale.btn.demo_mixed"    id="ws-demo-mixed"    class="secondary" type="button">Demo: mixed portfolio</button>
            </div>
            <p data-i18n="view.wash_sale.hint.about" class="muted">±30-day bidirectional window. Boundary 30 days exactly = INSIDE. Disallowed loss = loss × min(replacement_qty, close_qty) / close_qty. Symbol match only — substantially-identical (options / ETFs / share classes) is out of scope.</p>
        </div>

        <div id="ws-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.wash_sale.h2.hits">Wash-sale hits</h2>
            <div id="ws-table"></div>
        </div>

        <div id="ws-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state.closings = makeDemoClosings(k);
        state.openings = makeDemoOpenings(k);
        document.getElementById('ws-close').value = closingsToBlob(state.closings);
        document.getElementById('ws-open').value  = openingsToBlob(state.openings);
    };
    document.getElementById('ws-demo-classic').addEventListener('click', () => loadDemo('classic-trap'));
    document.getElementById('ws-demo-win').addEventListener('click',     () => loadDemo('winning-trade-no-flag'));
    document.getElementById('ws-demo-outside').addEventListener('click', () => loadDemo('outside-window'));
    document.getElementById('ws-demo-partial').addEventListener('click', () => loadDemo('partial-replacement'));
    document.getElementById('ws-demo-multi').addEventListener('click',   () => loadDemo('multi-hit'));
    document.getElementById('ws-demo-mixed').addEventListener('click',   () => loadDemo('mixed'));
    document.getElementById('ws-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function closingsToBlob(closings) {
    return closings.map(c => `${c.symbol} ${c.closed_at} ${c.net_pnl} ${c.qty}`).join('\n');
}

function openingsToBlob(openings) {
    return openings.map(o => `${o.symbol} ${o.executed_at} ${o.qty}`).join('\n');
}

function readInputs() {
    const pc = parseClosingBlob(document.getElementById('ws-close').value);
    const po = parseOpeningBlob(document.getElementById('ws-open').value);
    const errs = [
        ...pc.errors.map(e => `close[${e.line_no}] ${e.message}`),
        ...po.errors.map(e => `open[${e.line_no}] ${e.message}`),
    ];
    if (errs.length) { showErr(errs.slice(0, 4).join('; ')); return; }
    hideErr();
    state.closings = pc.rows;
    state.openings = po.rows;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.closings, state.openings);
    if (err) { showErr(err); return; }
    const localHits = localDetectHits(state.closings, state.openings);
    const localTotal = localTotalDisallowed(localHits);
    renderSummary({ hits: localHits, total_disallowed: localTotal }, true);
    renderTable(localHits);
    let resp;
    try {
        resp = await api.calcWashSale(buildBody(state.closings, state.openings));
    } catch (e) {
        showErr(`${t('view.wash_sale.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    const normalized = {
        hits: (resp.hits || []).map(h => ({
            ...h,
            loss_amount: dec(h.loss_amount),
            disallowed_loss_estimate: dec(h.disallowed_loss_estimate),
        })),
        total_disallowed: dec(resp.total_disallowed),
    };
    renderSummary(normalized, false);
    renderTable(normalized.hits);
}

function renderSummary(report, pending) {
    const totalLoss = totalRealizedLoss(state.closings);
    const badge = washBadge(report.total_disallowed, totalLoss);
    const local = localDetectHits(state.closings, state.openings);
    const localTotal = localTotalDisallowed(local);
    const parityOk = Math.abs(report.total_disallowed - localTotal) < 1e-6
                  && report.hits.length === local.length;
    const localTag = pending ? ` (${t('view.wash_sale.tag.local')})` : '';
    const disallowedPct = totalLoss > 0 ? report.total_disallowed / totalLoss : 0;
    document.getElementById('ws-summary').innerHTML = [
        card(t('view.wash_sale.card.verdict'),       t(badge.key) + localTag, badge.cls),
        card(t('view.wash_sale.card.hits'),          String(report.hits.length),
             report.hits.length > 0 ? 'neg' : 'pos'),
        card(t('view.wash_sale.card.total_loss'),    fmtUSD(totalLoss), totalLoss > 0 ? 'neg' : ''),
        card(t('view.wash_sale.card.disallowed'),    fmtUSD(report.total_disallowed), 'neg'),
        card(t('view.wash_sale.card.disallowed_pct'), fmtPct(disallowedPct),
             disallowedPct >= 0.25 ? 'neg' : ''),
        card(t('view.wash_sale.card.usable_loss'),   fmtUSD(Math.max(0, totalLoss - report.total_disallowed)),
             totalLoss - report.total_disallowed > 0 ? 'pos' : ''),
        card(t('view.wash_sale.card.parity'),
             parityOk ? t('view.wash_sale.tag.ok') : t('view.wash_sale.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderTable(hits) {
    const wrap = document.getElementById('ws-table');
    if (!hits || hits.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.wash_sale.empty">${esc(t('view.wash_sale.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.wash_sale.col.symbol">Symbol</th>
                <th data-i18n="view.wash_sale.col.close_id">Loss trade</th>
                <th data-i18n="view.wash_sale.col.open_id">Replacement buy</th>
                <th data-i18n="view.wash_sale.col.days">Days offset</th>
                <th data-i18n="view.wash_sale.col.loss">Loss</th>
                <th data-i18n="view.wash_sale.col.disallowed">Disallowed</th>
                <th data-i18n="view.wash_sale.col.ratio">Disallowed %</th>
            </tr></thead>
            <tbody>
                ${hits.map(h => {
                    const ratio = dec(h.loss_amount) > 0 ? dec(h.disallowed_loss_estimate) / dec(h.loss_amount) : 0;
                    return `<tr>
                        <td><strong>${esc(h.symbol)}</strong></td>
                        <td class="muted"><code>${esc(shortUuid(h.losing_trade_id))}</code></td>
                        <td class="muted"><code>${esc(shortUuid(h.replacement_execution_id))}</code></td>
                        <td>${esc(fmtDays(h.days_offset))}</td>
                        <td class="neg">${esc(fmtUSD(dec(h.loss_amount)))}</td>
                        <td class="neg">${esc(fmtUSD(dec(h.disallowed_loss_estimate)))}</td>
                        <td class="${ratio >= 0.5 ? 'neg' : ''}">${esc(fmtPct(ratio))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
    void fmtNum;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('ws-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ws-err').style.display = 'none'; }
