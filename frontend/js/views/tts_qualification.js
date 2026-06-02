// Trader Tax Status (TTS) Qualification Checklist.
// TTS is not statutorily defined — it's a facts-and-circumstances test based
// on case law (Endicott v. Comm., Holsinger, Vyhnal, Chen, Mayer). The IRS
// looks at: (1) volume + frequency, (2) short holding periods, (3) intent to
// profit from daily swings, (4) substantial trading activity (not investing).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-tts-checklist-v1';

const FACTORS = [
    { key: 'trades_per_year',     question: 'Trades per year',
      target: 720, type: 'number', critical: true,
      hint: 'Endicott: 535 OK. IRS internal target: 720 (4× weekly avg). 1000+ = strong.' },
    { key: 'trading_days_per_year', question: 'Trading days per year',
      target: 200, type: 'number', critical: true,
      hint: 'Must be daily-frequency activity. < 75 days = NOT TTS. 200+ = strong.' },
    { key: 'avg_holding_days',    question: 'Average holding period (days)',
      target: 30, type: 'number', critical: true, invert: true,
      hint: 'Days/weeks OK; months/years = INVESTOR not TRADER. Day-traders < 5 days avg.' },
    { key: 'hours_per_day',       question: 'Hours/day actively trading',
      target: 4, type: 'number', critical: false,
      hint: 'IRS expects "substantial time." 4+ hrs/day = strong; sub-1 hour = weak.' },
    { key: 'profit_intent',       question: 'Profit-from-swings intent (vs. dividends/LT appreciation)',
      type: 'checkbox', critical: true,
      hint: 'Must be SHORT-TERM profit motive. Holding for dividends = investor.' },
    { key: 'separate_office',     question: 'Separate office or dedicated trading workspace',
      type: 'checkbox', critical: false,
      hint: 'Shows business-like operation. Home office OK.' },
    { key: 'subscriptions',       question: 'Pay for data subscriptions / trading services',
      type: 'checkbox', critical: false,
      hint: 'Webull / Finnhub / Bloomberg / Trade Ideas — shows business-like activity.' },
    { key: 'continuous_activity', question: 'Trading is continuous (not seasonal)',
      type: 'checkbox', critical: true,
      hint: 'Must be regular, frequent, continuous — not "ramped up in April".' },
    { key: 'primary_income',      question: 'Trading is primary or significant income source',
      type: 'checkbox', critical: false,
      hint: 'Helpful but not required. Doctor-who-trades-2hrs-evenings can still qualify.' },
    { key: 'business_entity',     question: 'Operate via LLC, S-corp, or sole prop (Schedule C)',
      type: 'checkbox', critical: false,
      hint: 'Strong evidence of business operation. Trading via personal account is OK but weaker.' },
    { key: 'no_other_full_job',   question: 'No other full-time non-trading W-2 job',
      type: 'checkbox', critical: false,
      hint: 'Holsinger lost TTS partly because of his full-time medical practice.' },
];

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || 'null') || {}; }
    catch { return {}; }
}
function save(answers) { try { localStorage.setItem(LS_KEY, JSON.stringify(answers)); } catch { /* ignore */ } }

let state = { answers: load() };

export async function renderTtsQualification(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.tts.h1.title">// TTS QUALIFICATION CHECKLIST</span></h1>
        <p class="muted small" data-i18n="view.tts.hint.intro">
            Trader Tax Status (TTS) is a facts-and-circumstances test from case law
            (Endicott, Holsinger, Vyhnal, Mayer). No bright-line rule — IRS weighs volume,
            frequency, holding period, intent, business-like operation. Pass = Schedule C +
            ability to make § 475(f) MTM election + QBI eligibility.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.tts.h2.factors">Factors</h2>
            <form id="tts-form">
                ${FACTORS.map((f, i) => renderFactorInput(f, i)).join('')}
                <button class="primary" type="submit" data-i18n="view.tts.btn.compute" style="margin-top:10px">Compute</button>
            </form>
        </div>
        <div id="tts-result"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.tts.h2.cases">Key cases</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.tts.th.case">Case</th>
                    <th data-i18n="view.tts.th.result">Result</th>
                    <th data-i18n="view.tts.th.takeaway">Takeaway</th>
                </tr></thead>
                <tbody>
                    <tr><td>Endicott v. Comm. (2013)</td><td class="pos">TTS granted</td>
                        <td class="muted">535 trades / 250 days / 4 hrs daily</td></tr>
                    <tr><td>Holsinger v. Comm. (2008)</td><td class="neg">TTS denied</td>
                        <td class="muted">Full-time MD; trading was hobby-frequency</td></tr>
                    <tr><td>Vyhnal v. Comm. (2009)</td><td class="neg">TTS denied</td>
                        <td class="muted">Long holding periods; investor intent</td></tr>
                    <tr><td>Chen v. Comm. (2004)</td><td class="neg">TTS denied</td>
                        <td class="muted">Sporadic activity; not continuous</td></tr>
                    <tr><td>Mayer v. Comm. (1994)</td><td class="pos">TTS granted</td>
                        <td class="muted">1,100+ trades / day-frequency / business-like</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('tts-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.answers = {};
        for (const f of FACTORS) {
            if (f.type === 'checkbox') state.answers[f.key] = !!fd.get(f.key);
            else state.answers[f.key] = Number(fd.get(f.key)) || 0;
        }
        save(state.answers);
        showToast(t('view.tts.toast.saved'), { level: 'success' });
        renderResult();
    });
    renderResult();
}

function renderFactorInput(f, i) {
    const cur = state.answers[f.key];
    const critBadge = f.critical
        ? `<span class="tile-badge neg" style="margin-left:6px" data-i18n="view.tts.badge.critical">CRITICAL</span>`
        : '';
    if (f.type === 'checkbox') {
        return `<div style="padding:8px 0;border-bottom:1px solid var(--border)">
            <label style="display:flex;align-items:center;gap:8px;cursor:pointer">
                <input type="checkbox" name="${esc(f.key)}" ${cur ? 'checked' : ''}>
                <span><strong>${i + 1}. ${esc(f.question)}</strong>${critBadge}</span>
            </label>
            <p class="muted small" style="margin:4px 0 0 24px">${esc(f.hint)}</p>
        </div>`;
    }
    return `<div style="padding:8px 0;border-bottom:1px solid var(--border)">
        <label style="display:block">
            <span><strong>${i + 1}. ${esc(f.question)}</strong>${critBadge}
                ${f.target ? `<span class="muted small">(target: ${f.invert ? '≤' : '≥'} ${f.target})</span>` : ''}
            </span>
            <input type="number" step="any" name="${esc(f.key)}" value="${cur || ''}" style="margin-top:4px">
        </label>
        <p class="muted small" style="margin:4px 0 0 0">${esc(f.hint)}</p>
    </div>`;
}

function renderResult() {
    const el = document.getElementById('tts-result');
    if (!el) return;
    let score = 0;
    let maxScore = 0;
    let criticalPassed = 0;
    let criticalTotal = 0;
    const breakdown = [];
    for (const f of FACTORS) {
        const v = state.answers[f.key];
        if (f.type === 'checkbox') {
            maxScore += f.critical ? 2 : 1;
            if (v) {
                score += f.critical ? 2 : 1;
            }
            if (f.critical) {
                criticalTotal++;
                if (v) criticalPassed++;
            }
            breakdown.push({ ...f, value: v, passed: !!v });
        } else {
            maxScore += f.critical ? 2 : 1;
            const passed = f.invert ? (v > 0 && v <= f.target) : (v >= f.target);
            if (passed) score += f.critical ? 2 : 1;
            if (f.critical) {
                criticalTotal++;
                if (passed) criticalPassed++;
            }
            breakdown.push({ ...f, value: v, passed });
        }
    }
    const pct = maxScore > 0 ? (score / maxScore) : 0;
    const verdict = criticalPassed === criticalTotal && pct >= 0.75
        ? { cls: 'pos', label: t('view.tts.verdict.strong') }
        : criticalPassed >= criticalTotal - 1 && pct >= 0.5
        ? { cls: '', label: t('view.tts.verdict.marginal') }
        : { cls: 'neg', label: t('view.tts.verdict.fail') };
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.tts.h2.score">Qualification score</h2>
            <div class="cards">
                <div class="card ${verdict.cls}">
                    <div class="label" data-i18n="view.tts.card.verdict">Verdict</div>
                    <div class="value">${esc(verdict.label)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.tts.card.score">Score</div>
                    <div class="value">${score} / ${maxScore} (${(pct * 100).toFixed(0)}%)</div>
                </div>
                <div class="card ${criticalPassed === criticalTotal ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.tts.card.critical_passed">Critical factors passed</div>
                    <div class="value">${criticalPassed} / ${criticalTotal}</div>
                </div>
            </div>
            <h3 style="margin-top:14px" data-i18n="view.tts.h3.breakdown">Factor breakdown</h3>
            <table class="trades">
                <tbody>${breakdown.map((b, i) => `
                    <tr>
                        <td>${i + 1}. ${esc(b.question)}</td>
                        <td class="${b.passed ? 'pos' : 'neg'}">${b.passed ? '✓' : '×'}</td>
                        <td class="muted">${b.critical ? esc(t('view.tts.factor.critical')) : esc(t('view.tts.factor.standard'))}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
    `;
}
