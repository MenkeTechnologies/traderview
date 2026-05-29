// Futures roll-schedule view. Reads open futures positions + today +
// roll window → emits per-position urgency table sorted most-urgent-first.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parsePositionBlob, validateInputs, buildBody, localSchedule,
    urgencyBadge, urgencyLabelKey, overallBadge,
    makeDemoPositions, todayIso,
    fmtDays, fmtContracts,
} from '../_futures_roll_inputs.js';

let state = {
    positions: makeDemoPositions('mixed'),
    today: todayIso(),
    roll_window_days: 7,
};

export async function renderFuturesRoll(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.futures_roll.h1.title" class="view-title">// FUTURES ROLL SCHEDULE</h1>

        <div class="chart-panel" data-context-scope="futures-roll">
            <h2 data-i18n="view.futures_roll.h2.positions">Open futures positions
                <small data-i18n="view.futures_roll.h2.positions_hint" class="muted">(per line: symbol contracts YYYY-MM-DD; +contracts long / −short)</small></h2>
            <textarea id="fr-pos" rows="6"
                      data-tip="view.futures_roll.tip.positions"
                      placeholder="/ES 1 2026-06-19">${esc(positionsToBlob(state.positions))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.futures_roll.label.today">Today</span>
                    <input id="fr-today" type="date" value="${esc(state.today)}"></label>
                <label><span data-i18n="view.futures_roll.label.window">Roll window (days)</span>
                    <input id="fr-window" type="number" step="1" min="0" value="${state.roll_window_days}"></label>
                <button data-i18n="view.futures_roll.btn.schedule" id="fr-run" class="primary"
                        data-tip="view.futures_roll.tip.schedule" type="button">Build schedule</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.futures_roll.btn.demo_mixed"  id="fr-demo-mix"   class="secondary" type="button">Demo: mixed urgency</button>
                <button data-i18n="view.futures_roll.btn.demo_now"    id="fr-demo-now"   class="secondary" type="button">Demo: all NOW</button>
                <button data-i18n="view.futures_roll.btn.demo_soon"   id="fr-demo-soon"  class="secondary" type="button">Demo: all SOON</button>
                <button data-i18n="view.futures_roll.btn.demo_comf"   id="fr-demo-comf"  class="secondary" type="button">Demo: all comfortable</button>
                <button data-i18n="view.futures_roll.btn.demo_emerg"  id="fr-demo-emerg" class="secondary" type="button">Demo: emergency (expired)</button>
            </div>
            <p data-i18n="view.futures_roll.hint.about" class="muted">Urgency: &lt; 0d EXPIRED · ≤ window NOW · ≤ 2× window SOON · else COMFORTABLE. Most futures traders roll 5-10 days before expiry to avoid physical delivery.</p>
        </div>

        <div id="fr-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.futures_roll.h2.rows">Roll schedule (most-urgent first)</h2>
            <div id="fr-table"></div>
        </div>

        <div id="fr-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state.positions = makeDemoPositions(k, state.today);
        document.getElementById('fr-pos').value = positionsToBlob(state.positions);
    };
    document.getElementById('fr-demo-mix').addEventListener('click',   () => loadDemo('mixed'));
    document.getElementById('fr-demo-now').addEventListener('click',   () => loadDemo('all-now'));
    document.getElementById('fr-demo-soon').addEventListener('click',  () => loadDemo('all-soon'));
    document.getElementById('fr-demo-comf').addEventListener('click',  () => loadDemo('comfortable'));
    document.getElementById('fr-demo-emerg').addEventListener('click', () => loadDemo('emergency'));
    document.getElementById('fr-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function positionsToBlob(positions) {
    return positions.map(p => `${p.symbol} ${p.contracts} ${p.expiration}`).join('\n');
}

function readInputs() {
    const p = parsePositionBlob(document.getElementById('fr-pos').value);
    if (p.errors.length) {
        showErr(`${t('view.futures_roll.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.positions        = p.positions;
    state.today            = document.getElementById('fr-today').value;
    state.roll_window_days = parseInt(document.getElementById('fr-window').value, 10);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.positions, state.today, state.roll_window_days);
    if (err) { showErr(err); return; }
    const local = localSchedule(state.positions, state.today, state.roll_window_days);
    renderSummary(local, true);
    renderTable(local);
    let resp;
    try {
        resp = await api.futuresRollSchedule(buildBody(state.positions, state.today, state.roll_window_days));
    } catch (e) {
        showErr(`${t('view.futures_roll.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localSchedule(state.positions, state.today, state.roll_window_days);
    const badge = overallBadge(report);
    const parityOk = report.now_count === local.now_count
                  && report.expired_count === local.expired_count
                  && report.rows.length === local.rows.length;
    const localTag = pending ? ` (${t('view.futures_roll.tag.local')})` : '';
    const soonCount = report.rows.filter(r => r.urgency === 'soon').length;
    const comfortableCount = report.rows.filter(r => r.urgency === 'comfortable').length;
    document.getElementById('fr-summary').innerHTML = [
        card(t('view.futures_roll.card.verdict'),     t(badge.key) + localTag, badge.cls),
        card(t('view.futures_roll.card.positions'),   String(report.rows.length)),
        card(t('view.futures_roll.card.expired'),     String(report.expired_count),
             report.expired_count > 0 ? 'neg' : 'pos'),
        card(t('view.futures_roll.card.now'),         String(report.now_count),
             report.now_count > 0 ? 'neg' : ''),
        card(t('view.futures_roll.card.soon'),        String(soonCount)),
        card(t('view.futures_roll.card.comfortable'), String(comfortableCount),
             comfortableCount === report.rows.length && report.rows.length > 0 ? 'pos' : ''),
        card(t('view.futures_roll.card.parity'),
             parityOk ? t('view.futures_roll.tag.ok') : t('view.futures_roll.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderTable(report) {
    const wrap = document.getElementById('fr-table');
    if (!report.rows || report.rows.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.futures_roll.empty">${esc(t('view.futures_roll.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.futures_roll.col.symbol">Symbol</th>
                <th data-i18n="view.futures_roll.col.contracts">Contracts</th>
                <th data-i18n="view.futures_roll.col.expiration">Expiration</th>
                <th data-i18n="view.futures_roll.col.days">Days to expiry</th>
                <th data-i18n="view.futures_roll.col.urgency">Urgency</th>
            </tr></thead>
            <tbody>
                ${report.rows.map(r => {
                    const badge = urgencyBadge(r.urgency);
                    return `<tr class="${badge.cls === 'neg' ? 'neg' : ''}">
                        <td><strong>${esc(r.symbol)}</strong></td>
                        <td class="${r.contracts >= 0 ? 'pos' : 'neg'}">${esc(fmtContracts(r.contracts))}</td>
                        <td>${esc(r.expiration)}</td>
                        <td class="${r.days_to_expiry < 0 ? 'neg' : r.days_to_expiry <= state.roll_window_days ? 'neg' : ''}">${esc(fmtDays(r.days_to_expiry))}</td>
                        <td data-i18n="${esc(urgencyLabelKey(r.urgency))}" class="${badge.cls}">${esc(t(badge.key))}</td>
                    </tr>`;
                }).join('')}
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
    const el = document.getElementById('fr-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('fr-err').style.display = 'none'; }
