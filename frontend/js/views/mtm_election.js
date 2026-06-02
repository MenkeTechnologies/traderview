// § 475(f) Mark-to-Market Election Tracker.
// THE election for active traders. Converts trading P&L from capital
// to ORDINARY (no $3k loss cap, no wash sales), enables QBI § 199A,
// closes positions at year-end for tax purposes (MTM).
//
// CRITICAL deadline: April 15 of the year you want it to apply. Missed
// it for 2024? Must wait until April 15 2026 for 2026 to be the first year.
// (You file election WITH timely 2024 return for 2025+ to apply.)

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-mtm-state-v1';

function load() {
    try {
        const raw = localStorage.getItem(LS_KEY);
        return raw ? JSON.parse(raw) : null;
    } catch { return null; }
}
function save(s) { try { localStorage.setItem(LS_KEY, JSON.stringify(s)); } catch { /* ignore */ } }

let state = load() || {
    election_filed: false,
    election_year: null,
    election_filed_date: null,
    tts_qualified: false,
    has_qbi: false,
    election_revoke: false,
};

export async function renderMtmElection(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    const today = new Date();
    const currentYear = today.getFullYear();
    const nextApril15 = `${currentYear}-04-15`;
    const daysUntilDeadline = Math.floor((new Date(nextApril15).getTime() - today.getTime()) / 86_400_000);
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mtm.h1.title">// § 475(f) MTM ELECTION</span></h1>
        <p class="muted small" data-i18n="view.mtm.hint.intro">
            <strong>THE election</strong> for active traders. Converts trading P&amp;L
            from capital to ordinary (no $3k loss cap, no wash sales), enables QBI
            § 199A. Year-end positions are marked-to-market for tax purposes.
            <strong>Hard deadline: April 15</strong> of the year you want it to
            first apply, attached to your prior year return.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.mtm.h2.deadline">Deadline status</h2>
            <div class="cards">
                <div class="card ${daysUntilDeadline > 30 ? 'pos' : daysUntilDeadline > 0 ? '' : 'neg'}">
                    <div class="label" data-i18n="view.mtm.card.days_to_deadline">Days to April 15 ${currentYear}</div>
                    <div class="value">${daysUntilDeadline}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.mtm.card.applies_year">If filed now, applies for</div>
                    <div class="value">${daysUntilDeadline > 0 ? currentYear : currentYear + 1}</div>
                </div>
            </div>
            <p class="muted small" data-i18n="view.mtm.deadline.note" style="margin-top:10px">
                Filed BY April 15 ${currentYear}: § 475(f) MTM applies for ${currentYear} forward.
                Missed it? Next opportunity: April 15 ${currentYear + 1} for ${currentYear + 1} forward.
                Can request relief via Rev. Proc. 2015-13 for late election but rarely granted.
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.mtm.h2.your_status">Your election status</h2>
            <form id="mtm-form" class="inline-form">
                <label><span data-i18n="view.mtm.label.tts_qualified">Trader Tax Status (TTS) qualified?</span>
                    <input type="checkbox" name="tts_qualified" ${state.tts_qualified ? 'checked' : ''}></label>
                <label><span data-i18n="view.mtm.label.has_qbi">Have QBI-generating business?</span>
                    <input type="checkbox" name="has_qbi" ${state.has_qbi ? 'checked' : ''}></label>
                <label><span data-i18n="view.mtm.label.election_filed">§ 475(f) election filed?</span>
                    <input type="checkbox" name="election_filed" ${state.election_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.mtm.label.election_year">First election year</span>
                    <input type="number" step="1" name="election_year" value="${state.election_year || ''}"></label>
                <label><span data-i18n="view.mtm.label.election_filed_date">Date election attached to return</span>
                    <input type="date" name="election_filed_date" value="${state.election_filed_date || ''}"></label>
                <label><span data-i18n="view.mtm.label.election_revoke">Plan to revoke?</span>
                    <input type="checkbox" name="election_revoke" ${state.election_revoke ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.mtm.btn.save">Save status</button>
            </form>
        </div>
        <div id="mtm-recommendation"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.mtm.h2.benefits">MTM benefits</h2>
            <ul class="muted small">
                <li data-i18n="view.mtm.benefit.ordinary">Trading P&L is ORDINARY income/loss — no $3k/yr capital-loss cap</li>
                <li data-i18n="view.mtm.benefit.no_wash">EXEMPT from wash-sale rules (§ 1091)</li>
                <li data-i18n="view.mtm.benefit.qbi">Trading P&L is QBI-eligible for § 199A 20% deduction</li>
                <li data-i18n="view.mtm.benefit.nol">Losses generate NOLs — carry forward indefinitely (vs. capital-loss carryover)</li>
                <li data-i18n="view.mtm.benefit.full_deduct">Trading expenses fully deductible (Schedule C, no 2% AGI floor)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.mtm.h2.drawbacks">MTM drawbacks</h2>
            <ul class="muted small">
                <li data-i18n="view.mtm.drawback.mtm_realization">Forced year-end mark-to-market — phantom gains on open positions</li>
                <li data-i18n="view.mtm.drawback.ordinary_rates">Gains taxed at ordinary rates (no LT 15-20% cap-gains advantage)</li>
                <li data-i18n="view.mtm.drawback.investment_segregation">Must SEGREGATE investments from trading account — invested holdings keep cap-gains treatment</li>
                <li data-i18n="view.mtm.drawback.no_revoke">Hard to revoke — Form 3115 + IRS consent + 5-year wait if voluntary</li>
                <li data-i18n="view.mtm.drawback.se_tax">If TTS+MTM via S-corp or sole-prop: trading income may be SE-taxable</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.mtm.h2.checklist">Filing checklist</h2>
            <ol class="muted small">
                <li data-i18n="view.mtm.checklist.tts">Confirm TTS qualified (trades 720+ per year, regular daily activity)</li>
                <li data-i18n="view.mtm.checklist.attach">Attach signed statement to PRIOR year 1040 by April 15</li>
                <li data-i18n="view.mtm.checklist.text">Statement language: "Pursuant to IRC § 475(f)(1), the taxpayer hereby elects to use the mark-to-market method of accounting for securities and traders in securities, effective for the taxable year beginning [date]..."</li>
                <li data-i18n="view.mtm.checklist.form_3115">File Form 3115 with the FIRST election-year return (accounting method change)</li>
                <li data-i18n="view.mtm.checklist.form_4797">Report MTM gains/losses on Form 4797 Part II (ordinary)</li>
                <li data-i18n="view.mtm.checklist.schedule_c">Trading expenses on Schedule C, trading income on Form 4797</li>
            </ol>
        </div>
    `;
    document.getElementById('mtm-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state = {
            tts_qualified: !!fd.get('tts_qualified'),
            has_qbi: !!fd.get('has_qbi'),
            election_filed: !!fd.get('election_filed'),
            election_year: Number(fd.get('election_year')) || null,
            election_filed_date: fd.get('election_filed_date') || null,
            election_revoke: !!fd.get('election_revoke'),
        };
        save(state);
        showToast(t('view.mtm.toast.saved'), { level: 'success' });
        renderRecommendation();
    });
    renderRecommendation();
}

function renderRecommendation() {
    const el = document.getElementById('mtm-recommendation');
    if (!el) return;
    let cls = '';
    let title = '';
    let body = '';
    if (!state.tts_qualified) {
        cls = 'neg';
        title = t('view.mtm.rec.tts_first');
        body = t('view.mtm.rec.tts_first_body');
    } else if (state.election_filed) {
        cls = 'pos';
        title = t('view.mtm.rec.filed_continue');
        body = t('view.mtm.rec.filed_continue_body');
    } else {
        cls = '';
        title = t('view.mtm.rec.consider');
        body = t('view.mtm.rec.consider_body');
    }
    el.innerHTML = `
        <div class="chart-panel ${cls}">
            <h2><strong>${esc(title)}</strong></h2>
            <p>${esc(body)}</p>
        </div>
    `;
}
