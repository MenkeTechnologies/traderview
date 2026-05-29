// News Event Handler view — pre-event auto-resize policy advisor.
//
// Trader pastes their current open positions + the calendar of upcoming
// high-impact news events (FOMC / NFP / CPI / earnings). Engine emits
// a trim recommendation per position: Low impact = no action, Medium =
// 25% trim, High = 50% trim, Critical = full close.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parsePositions, parseEvents, validateInputs, buildBody,
    impactBadge, summarize, makeDemoData,
    fmtN, fmtInt, fmtPct, trimFractionFor,
} from '../_news_event_inputs.js';

import { t } from '../i18n.js';
let state = { positionsText: '', eventsText: '' };

export async function renderNewsEvent(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.news_event.h1.news_event_handler" class="view-title">// NEWS EVENT HANDLER</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.news_event.h2.open_positions">Open positions</h2>
            <p class="muted" data-i18n="view.news_event.hint.position_format">One line per position: symbol qty.</p>
            <textarea id="ne-pos" rows="5" placeholder="AAPL 100&#10;TSLA 50&#10;MSFT 200"></textarea>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.news_event.h2.upcoming_news_events">Upcoming news events</h2>
            <p class="muted" data-i18n="view.news_event.hint.event_format">One event per line: event_name &lt;low|medium|high|critical&gt; [comma,sep,symbols]. Omit symbols for market-wide events (e.g., FOMC, NFP). Event-name can contain spaces; the parser finds the impact token automatically.</p>
            <textarea id="ne-ev" rows="5" placeholder="FOMC critical&#10;CPI high TSLA&#10;Retail sales medium MSFT&#10;Fed minutes low ILQD"></textarea>
            <div class="inline-form">
                <button data-i18n="view.news_event.btn.load_demo_5_positions_4_events_spanning_all_tiers" id="ne-demo" class="secondary" type="button">Load demo (5 positions, 4 events spanning all tiers)</button>
                <button data-i18n="view.news_event.btn.clear" id="ne-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.news_event.btn.evaluate" id="ne-run" class="primary" type="button">Evaluate</button>
            </div>
        </div>

        <div id="ne-errors" class="boot" style="display:none"></div>
        <div id="ne-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.news_event.h2.trim_recommendations">Trim recommendations</h2>
            <div id="ne-actions"></div>
            <p data-i18n="view.news_event.hint.trim_percent_by_impact_low_0_no_action_medium_25_h" class="muted">Trim percent by impact: Low=0% (no action) · Medium=25% · High=50% · Critical=100% (full close).
                If multiple events affect a position, the highest-impact one wins.</p>
        </div>

        <div id="ne-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('ne-demo').addEventListener('click', () => {
        const { positions, events } = makeDemoData();
        document.getElementById('ne-pos').value =
            positions.map(p => `${p.symbol} ${p.current_qty}`).join('\n');
        document.getElementById('ne-ev').value = events.map(e => {
            const sym = e.affected_symbols.length ? ' ' + e.affected_symbols.join(',') : '';
            return `${e.event_name} ${e.impact}${sym}`;
        }).join('\n');
    });
    document.getElementById('ne-clear').addEventListener('click', () => {
        document.getElementById('ne-pos').value = '';
        document.getElementById('ne-ev').value = '';
    });
    document.getElementById('ne-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.positionsText = document.getElementById('ne-pos').value;
    state.eventsText = document.getElementById('ne-ev').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('ne-errors');
    errs.style.display = 'none';
    const { positions, errors: pe } = parsePositions(state.positionsText);
    const { events,    errors: ee } = parseEvents(state.eventsText);
    const allErrs = [
        ...pe.map(e => ({ ...e, src: 'positions' })),
        ...ee.map(e => ({ ...e, src: 'events' })),
    ];
    if (allErrs.length) {
        const head = allErrs.slice(0, 8).map(e =>
            `[${e.src}] line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = allErrs.length > 8 ? `<br>… and ${allErrs.length - 8} more.` : '';
        errs.innerHTML = `<strong>${allErrs.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
    }
    const err = validateInputs(positions, events);
    if (err) { showErr(err); return; }
    let report;
    try {
        report = await api.regimeNewsEvent(buildBody(positions, events));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report, positions, events);
    renderActions(report);
}

function renderSummary(report, positions, events) {
    const s = summarize(report, positions);
    const totalQty = positions.reduce((a, p) => a + (p.current_qty || 0), 0);
    const trimPct = totalQty > 0 ? s.totalTrim / totalQty : 0;
    document.getElementById('ne-summary').innerHTML = [
        card(t('view.news_event.card.positions'),  String(s.positionCount)),
        card(t('view.news_event.card.events'),     String(events.length)),
        card(t('view.news_event.card.actions'),    String(s.actionCount), s.actionCount ? 'neg' : 'pos'),
        card(t('view.news_event.card.unchanged'),  String(s.unchanged), s.unchanged ? 'pos' : ''),
        card(t('view.news_event.card.total_qty'),  fmtInt(totalQty)),
        card(t('view.news_event.card.total_trim'), fmtInt(s.totalTrim), s.totalTrim ? 'neg' : ''),
        card(t('view.news_event.card.trim_of_book'), fmtPct(trimPct), trimPct > 0.5 ? 'neg' : ''),
        card(t('view.news_event.card.critical_actions'), String(s.critical), s.critical ? 'neg' : ''),
    ].join('');
    void trimFractionFor; // re-exported for spec / future hint UI
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderActions(report) {
    const wrap = document.getElementById('ne-actions');
    const actions = (report && report.actions) || [];
    if (!actions.length) {
        wrap.innerHTML = '<div class="muted">No trim actions — no high-impact events affecting open positions.</div>';
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.news_event.th.symbol">Symbol</th><th data-i18n="view.news_event.th.current">Current</th><th data-i18n="view.news_event.th.recommended">Recommended</th>
                <th data-i18n="view.news_event.th.trim">Trim</th><th data-i18n="view.news_event.th.trim_2">Trim %</th><th data-i18n="view.news_event.th.reason">Reason</th>
            </tr></thead>
            <tbody>
                ${actions.map(a => {
                    const trimPct = a.current_qty > 0 ? a.trim_amount / a.current_qty : 0;
                    // Pull the impact word out of the reason text so we can color-badge it.
                    const reasonLower = String(a.reason || '').toLowerCase();
                    let impact = 'medium';
                    if (reasonLower.includes('critical')) impact = 'critical';
                    else if (reasonLower.includes('high')) impact = 'high';
                    else if (reasonLower.includes('low')) impact = 'low';
                    const badge = impactBadge(impact);
                    return `<tr>
                        <td><strong>${esc(a.symbol)}</strong></td>
                        <td>${esc(fmtN(a.current_qty))}</td>
                        <td>${esc(fmtN(a.recommended_qty))}</td>
                        <td class="neg">${esc(fmtN(a.trim_amount))}</td>
                        <td class="${badge.cls}">${esc(fmtPct(trimPct))} · ${esc(badge.label)}</td>
                        <td class="muted">${esc(a.reason || '')}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('ne-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ne-err').style.display = 'none'; }
