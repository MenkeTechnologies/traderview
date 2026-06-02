// Trader Tax Status (TTS) qualification scorer.
// Gates the § 475(f) MTM election. IRS uses a multi-factor test from case law
// (Holsinger, Endicott, Vines). No bright line — typical criteria: ≥720 trades/yr,
// ≥4 hrs/day, daily activity, short holding period, equipment, intent to profit
// from short-term moves. Score against documented thresholds.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    annual_trades: 0,
    trading_days: 0,
    avg_hours_per_day: 0,
    avg_holding_days: 30,
    has_dedicated_setup: false,
    business_intent: false,
    has_business_entity: false,
    only_trader: false,
};

export async function renderTtsScorer(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.tts.h1.title">// TRADER TAX STATUS SCORER</span></h1>
        <p class="muted small" data-i18n="view.tts.hint.intro">
            TTS isn't a checkbox — it's a <strong>facts-and-circumstances</strong> test from
            case law (Holsinger, Endicott, Vines, Mayer). Without TTS, you can't make a
            § 475(f) MTM election. Score yourself against documented benchmarks: ≥720 total
            trades/yr, daily activity (≥4 hrs/day on trading days), short avg holds (under 31 days),
            business-like setup, profit-from-short-term-moves intent.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.tts.h2.inputs">Inputs</h2>
            <form id="tts-form" class="inline-form">
                <label><span data-i18n="view.tts.label.annual_trades">Total trades per year (round trip = 2)</span>
                    <input type="number" step="1" name="annual_trades" value="${state.annual_trades}"></label>
                <label><span data-i18n="view.tts.label.trading_days">Days traded in year</span>
                    <input type="number" step="1" name="trading_days" value="${state.trading_days}"></label>
                <label><span data-i18n="view.tts.label.avg_hours">Avg hours/day on trading days</span>
                    <input type="number" step="0.5" name="avg_hours_per_day" value="${state.avg_hours_per_day}"></label>
                <label><span data-i18n="view.tts.label.avg_holding">Avg holding period (days)</span>
                    <input type="number" step="1" name="avg_holding_days" value="${state.avg_holding_days}"></label>
                <label><span data-i18n="view.tts.label.dedicated_setup">Dedicated home office + multi-monitor setup?</span>
                    <input type="checkbox" name="has_dedicated_setup" ${state.has_dedicated_setup ? 'checked' : ''}></label>
                <label><span data-i18n="view.tts.label.business_intent">Documented business intent (plan, journal)?</span>
                    <input type="checkbox" name="business_intent" ${state.business_intent ? 'checked' : ''}></label>
                <label><span data-i18n="view.tts.label.entity">LLC / S-corp / sole prop entity?</span>
                    <input type="checkbox" name="has_business_entity" ${state.has_business_entity ? 'checked' : ''}></label>
                <label><span data-i18n="view.tts.label.only_trader">Trading is your sole / primary occupation?</span>
                    <input type="checkbox" name="only_trader" ${state.only_trader ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.tts.btn.score">Score</button>
            </form>
        </div>
        <div id="tts-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.tts.h2.case_law">Leading case law</h2>
            <ul class="muted small">
                <li data-i18n="view.tts.case.holsinger">Holsinger v. Comm'r (2008) — DENIED TTS: 372 trades, $3M volume insufficient</li>
                <li data-i18n="view.tts.case.endicott">Endicott v. Comm'r (2013) — DENIED: only 1.7-yr avg hold, options writer not trader</li>
                <li data-i18n="view.tts.case.vines">Vines v. Comm'r (2006) — GRANTED relief for late MTM § 9100 election</li>
                <li data-i18n="view.tts.case.mayer">Mayer v. Comm'r (1994) — DENIED: investments, not trading; 600 trades / 5.3-yr hold</li>
                <li data-i18n="view.tts.case.chen">Chen v. Comm'r (2004) — DENIED: 323 trades, 94 trading days, lacks frequency</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.tts.h2.benefits">TTS benefits</h2>
            <ul class="muted small">
                <li data-i18n="view.tts.benefit.expenses">Schedule C business expense deduction (no 2% floor, no AMT)</li>
                <li data-i18n="view.tts.benefit.health">Health insurance § 162(l) above-the-line deduction</li>
                <li data-i18n="view.tts.benefit.retirement">Solo 401(k), SEP IRA contributions on trading income</li>
                <li data-i18n="view.tts.benefit.475f">Eligible to make § 475(f) MTM election (no wash sales, ordinary treatment)</li>
                <li data-i18n="view.tts.benefit.home_office">Home office § 280A deduction</li>
                <li data-i18n="view.tts.benefit.equipment">Equipment § 179 / bonus depreciation</li>
            </ul>
        </div>
    `;
    document.getElementById('tts-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.annual_trades = Number(fd.get('annual_trades')) || 0;
        state.trading_days = Number(fd.get('trading_days')) || 0;
        state.avg_hours_per_day = Number(fd.get('avg_hours_per_day')) || 0;
        state.avg_holding_days = Number(fd.get('avg_holding_days')) || 0;
        state.has_dedicated_setup = !!fd.get('has_dedicated_setup');
        state.business_intent = !!fd.get('business_intent');
        state.has_business_entity = !!fd.get('has_business_entity');
        state.only_trader = !!fd.get('only_trader');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('tts-output');
    if (!el) return;
    const tradesScore = state.annual_trades >= 720 ? 25 : (state.annual_trades >= 360 ? 12 : 0);
    const daysScore = state.trading_days >= 200 ? 20 : (state.trading_days >= 150 ? 10 : 0);
    const hoursScore = state.avg_hours_per_day >= 4 ? 15 : (state.avg_hours_per_day >= 2 ? 7 : 0);
    const holdScore = state.avg_holding_days < 7 ? 15 : (state.avg_holding_days < 31 ? 10 : 0);
    const setupScore = state.has_dedicated_setup ? 5 : 0;
    const intentScore = state.business_intent ? 5 : 0;
    const entityScore = state.has_business_entity ? 5 : 0;
    const onlyScore = state.only_trader ? 10 : 0;
    const total = tradesScore + daysScore + hoursScore + holdScore + setupScore + intentScore + entityScore + onlyScore;
    let verdict, cls;
    if (total >= 80) { verdict = t('view.tts.verdict.strong'); cls = 'pos'; }
    else if (total >= 60) { verdict = t('view.tts.verdict.likely'); cls = 'pos'; }
    else if (total >= 40) { verdict = t('view.tts.verdict.marginal'); cls = 'neg'; }
    else { verdict = t('view.tts.verdict.unlikely'); cls = 'neg'; }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.tts.h2.result">Score</h2>
            <div class="cards">
                <div class="card ${cls}">
                    <div class="label" data-i18n="view.tts.card.verdict">Verdict</div>
                    <div class="value">${esc(verdict)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.tts.card.total">Total</div>
                    <div class="value">${total} / 100</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.tts.card.trades_score">Trades (25)</div>
                    <div class="value">${tradesScore}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.tts.card.days_score">Days (20)</div>
                    <div class="value">${daysScore}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.tts.card.hours_score">Hours (15)</div>
                    <div class="value">${hoursScore}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.tts.card.hold_score">Holds (15)</div>
                    <div class="value">${holdScore}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.tts.card.setup_score">Setup (5)</div>
                    <div class="value">${setupScore}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.tts.card.intent_score">Intent (5)</div>
                    <div class="value">${intentScore}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.tts.card.entity_score">Entity (5)</div>
                    <div class="value">${entityScore}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.tts.card.only_score">Sole occupation (10)</div>
                    <div class="value">${onlyScore}</div>
                </div>
            </div>
        </div>
    `;
}
