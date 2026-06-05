// File Taxes wizard — TurboTax/Cash-App-Taxes-style multi-section
// guided interview.
//
// State model: one `draft` (TaxReturn) + one `result` (TaxResult)
// per (user, tax_year). Every field write triggers a debounced
// auto-save → server recomputes the result → we re-render the
// section's summary numbers and the left-rail checkmarks.

import { api } from '../api.js';
import { t } from '../i18n.js';
import { esc, fmtMoney } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const SECTIONS = [
    { id: 'personal',    titleKey: 'view.taxwiz.section.personal' },
    { id: 'income',      titleKey: 'view.taxwiz.section.income' },
    { id: 'adjustments', titleKey: 'view.taxwiz.section.adjustments' },
    { id: 'deductions',  titleKey: 'view.taxwiz.section.deductions' },
    { id: 'credits',     titleKey: 'view.taxwiz.section.credits' },
    { id: 'other_taxes', titleKey: 'view.taxwiz.section.other_taxes' },
    { id: 'review',      titleKey: 'view.taxwiz.section.review' },
    { id: 'download',    titleKey: 'view.taxwiz.section.download' },
];

const STATE = {
    year: new Date().getFullYear() - 1,   // default: last tax year
    section: 'personal',
    draft: null,
    result: null,
    saveTimer: null,
};

export async function renderTaxWizard(mount) {
    const tok = currentViewToken();

    // Resolve hash params: #file-taxes?year=2025 / ?section=income
    try {
        const hash = location.hash.replace(/^#/, '');
        const q = hash.includes('?') ? hash.slice(hash.indexOf('?') + 1) : '';
        const params = new URLSearchParams(q);
        const y = parseInt(params.get('year'), 10);
        if (!Number.isNaN(y)) STATE.year = y;
        // Param name `sec` (NOT `section`) — the literal substring
        // `&sect` would be HTML-entity-decoded as `§` by Tauri WebKit
        // on hash reads, producing `§ion=...` and breaking the router.
        const sec = params.get('sec') || params.get('section');
        if (sec && SECTIONS.some(s => s.id === sec)) STATE.section = sec;
    } catch (_) {}

    mount.innerHTML = `
        <div class="tw-shell" data-context-scope="tax-wiz-section">
            <header class="tw-head">
                <h2>${esc(t('view.taxwiz.title'))} —
                    <select id="tw-year">
                        ${yearOptions(STATE.year)}
                    </select>
                </h2>
                <span class="muted small">${esc(t('view.taxwiz.subtitle'))}</span>
                <button type="button" id="tw-autopop" class="btn btn-secondary btn-compact">
                    ${esc(t('view.taxwiz.action.autopopulate'))}
                </button>
            </header>
            <div class="tw-body">
                <nav class="tw-rail" id="tw-rail"></nav>
                <main class="tw-pane" id="tw-pane">
                    <div class="muted">${esc(t('common.loading'))}</div>
                </main>
            </div>
        </div>
    `;
    mount.querySelector('#tw-year').addEventListener('change', e => {
        STATE.year = parseInt(e.target.value, 10) || STATE.year;
        location.hash = `#file-taxes?year=${STATE.year}&sec=${STATE.section}`;
        loadAndRender(mount, tok);
    });
    mount.querySelector('#tw-autopop').addEventListener('click', () => autopop(mount, tok));

    await loadAndRender(mount, tok);
}

function yearOptions(selected) {
    const now = new Date().getFullYear();
    const years = [];
    for (let y = now; y >= now - 5; y--) years.push(y);
    return years.map(y => `<option value="${y}"${y === selected ? ' selected' : ''}>${y}</option>`).join('');
}

async function loadAndRender(mount, tok) {
    try {
        const r = await api.taxReturn(STATE.year);
        if (!viewIsCurrent(tok)) return;
        STATE.draft = r.draft;
        STATE.result = r.result;
        if (r.status && SECTIONS.some(s => s.id === r.status)) STATE.section = r.status;
    } catch (e) {
        const pane = mount.querySelector('#tw-pane');
        if (pane) pane.innerHTML = `<div class="err">${esc(t('view.taxwiz.err.load', { err: e.message }))}</div>`;
        return;
    }
    renderRail(mount);
    renderActiveSection(mount, tok);
}

function renderRail(mount) {
    const rail = mount.querySelector('#tw-rail');
    if (!rail) return;
    rail.innerHTML = SECTIONS.map(sec => {
        const active = sec.id === STATE.section ? ' tw-rail-active' : '';
        const checked = sectionCompleted(sec.id) ? '✓ ' : '';
        return `<button type="button" class="tw-rail-item${active}" data-sec="${sec.id}">${checked}${esc(t(sec.titleKey))}</button>`;
    }).join('');
    rail.querySelectorAll('button.tw-rail-item').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.section = btn.dataset.sec;
            location.hash = `#file-taxes?year=${STATE.year}&sec=${STATE.section}`;
            // Re-render rail (active class) + pane (new section).
            const m = btn.closest('.tw-shell').parentElement;
            renderRail(m);
            renderActiveSection(m, currentViewToken());
        });
    });
}

function sectionCompleted(secId) {
    const d = STATE.draft || {};
    switch (secId) {
        case 'personal':    return !!d.status;
        case 'income':      return (d.w2s && d.w2s.length > 0)
                                 || +d.interest_income > 0
                                 || +d.ordinary_dividends > 0
                                 || (d.schedule_c && +d.schedule_c.net_profit !== 0);
        case 'adjustments': return +d.hsa_deduction > 0
                                 || +d.ira_deduction > 0
                                 || +d.student_loan_interest > 0;
        case 'deductions':  return d.force_standard_deduction
                                 || (d.itemized && itemizedTotal(d.itemized) > 0);
        case 'credits':     return +d.qualifying_children_under_17 > 0
                                 || +d.other_dependents > 0;
        case 'other_taxes': return STATE.result && +STATE.result.se_tax.total > 0;
        case 'review':      return false; // never auto-marked
        case 'download':    return false;
        default: return false;
    }
}

function itemizedTotal(it) {
    return (+it.medical_over_7_5_pct_agi || 0)
         + Math.min(+it.state_and_local_taxes_capped_at_10k || 0, 10000)
         + (+it.mortgage_interest || 0)
         + (+it.charitable_gifts || 0)
         + (+it.casualty_losses || 0);
}

function renderActiveSection(mount, tok) {
    const pane = mount.querySelector('#tw-pane');
    if (!pane) return;
    const sec = STATE.section;
    if (sec === 'personal')    renderPersonal(pane, mount, tok);
    else if (sec === 'income') renderIncome(pane, mount, tok);
    else if (sec === 'adjustments') renderAdjustments(pane, mount, tok);
    else if (sec === 'deductions')  renderDeductions(pane, mount, tok);
    else if (sec === 'credits')     renderCredits(pane, mount, tok);
    else if (sec === 'other_taxes') renderOtherTaxes(pane, mount, tok);
    else if (sec === 'review')      renderReview(pane, mount, tok);
    else if (sec === 'download')    renderDownload(pane, mount, tok);
}

// ── auto-save (debounced) ─────────────────────────────────────────────

function scheduleSave(mount, tok, changeLabel) {
    clearTimeout(STATE.saveTimer);
    STATE.saveTimer = setTimeout(() => doSave(mount, tok, changeLabel), 400);
}

async function doSave(mount, tok, changeLabel) {
    try {
        const r = await api.saveTaxReturn(STATE.year, STATE.draft, STATE.section, changeLabel || 'autosave');
        if (!viewIsCurrent(tok)) return;
        STATE.result = r.result;
        renderRail(mount);
        // Re-render only the live-summary block of the current section
        // without rebuilding inputs (which would steal focus).
        const live = mount.querySelector('.tw-live-summary');
        if (live) live.innerHTML = liveSummaryHtml();
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.taxwiz.err.save', { err: e.message }), { level: 'error' });
    }
}

async function autopop(mount, tok) {
    try {
        const r = await api.autopopulateTaxReturn(STATE.year);
        if (!viewIsCurrent(tok)) return;
        STATE.draft = r.draft;
        STATE.result = r.result;
        renderRail(mount);
        renderActiveSection(mount, tok);
        showToast(t('view.taxwiz.toast.autopopulated'), { level: 'success' });
    } catch (e) {
        showToast(t('view.taxwiz.err.autopopulate', { err: e.message }), { level: 'error' });
    }
}

function liveSummaryHtml() {
    const r = STATE.result || {};
    return `
        <div class="tw-live-grid">
            <div><span>${esc(t('view.taxwiz.live.agi'))}</span><strong>${esc(fmtMoney(+r.agi || 0))}</strong></div>
            <div><span>${esc(t('view.taxwiz.live.taxable'))}</span><strong>${esc(fmtMoney(+r.taxable_income || 0))}</strong></div>
            <div><span>${esc(t('view.taxwiz.live.tax'))}</span><strong>${esc(fmtMoney(+r.tax_after_credits || 0))}</strong></div>
            <div><span>${esc(t('view.taxwiz.live.refund'))}</span><strong class="tw-refund">${esc(fmtMoney(+r.refund_due || 0))}</strong></div>
            <div><span>${esc(t('view.taxwiz.live.owed'))}</span><strong class="tw-owed">${esc(fmtMoney(+r.tax_owed || 0))}</strong></div>
        </div>
    `;
}

// ── Personal section ──────────────────────────────────────────────────

function renderPersonal(pane, mount, tok) {
    const d = STATE.draft;
    pane.innerHTML = `
        <h3>${esc(t('view.taxwiz.personal.h3'))}</h3>
        <p class="muted small">${esc(t('view.taxwiz.personal.subtitle'))}</p>
        <label class="tw-field">
            <span>${esc(t('view.taxwiz.personal.filing_status'))}</span>
            <select id="tw-status">
                ${['single', 'mfj', 'mfs', 'hoh'].map(s => `<option value="${s}"${d.status === s ? ' selected' : ''}>${esc(t('view.taxwiz.filing_status.' + s))}</option>`).join('')}
            </select>
        </label>
        <div class="tw-live-summary">${liveSummaryHtml()}</div>
        ${navButtons('income')}
    `;
    pane.querySelector('#tw-status').addEventListener('change', e => {
        d.status = e.target.value;
        scheduleSave(mount, tok, 'personal.filing_status');
    });
    wireNav(pane, mount, tok);
}

// ── Income section ────────────────────────────────────────────────────

function renderIncome(pane, mount, tok) {
    const d = STATE.draft;
    pane.innerHTML = `
        <h3>${esc(t('view.taxwiz.income.h3'))}</h3>
        <p class="muted small">${esc(t('view.taxwiz.income.subtitle'))}</p>

        <section class="tw-card">
            <header><strong>${esc(t('view.taxwiz.income.w2.h'))}</strong>
                <input type="file" id="tw-w2-file" accept="image/*,application/pdf" hidden>
                <button type="button" id="tw-w2-upload" class="btn btn-secondary btn-compact">
                    ${esc(t('view.taxwiz.income.w2.upload'))}
                </button>
                <button type="button" id="tw-w2-add" class="btn btn-secondary btn-compact">
                    ${esc(t('view.taxwiz.income.w2.add_manual'))}
                </button>
            </header>
            <div id="tw-w2-list" class="tw-list">${renderW2List(d.w2s)}</div>
        </section>

        <section class="tw-card">
            <header><strong>${esc(t('view.taxwiz.income.other.h'))}</strong></header>
            ${moneyField('tw-interest', 'view.taxwiz.income.interest', d.interest_income)}
            ${moneyField('tw-div-ord',  'view.taxwiz.income.ordinary_div', d.ordinary_dividends)}
            ${moneyField('tw-div-q',    'view.taxwiz.income.qualified_div', d.qualified_dividends)}
            ${moneyField('tw-ltcg',     'view.taxwiz.income.ltcg', d.net_long_term_capital_gain)}
            ${moneyField('tw-other',    'view.taxwiz.income.other_income', d.other_income)}
        </section>

        <section class="tw-card">
            <header><strong>${esc(t('view.taxwiz.income.sched_c.h'))}</strong></header>
            <p class="muted small">${esc(t('view.taxwiz.income.sched_c.hint'))}</p>
            ${moneyField('tw-sc-gross',  'view.taxwiz.income.sched_c.gross', d.schedule_c.gross_receipts)}
            ${moneyField('tw-sc-exp',    'view.taxwiz.income.sched_c.expenses', d.schedule_c.total_expenses)}
            <div class="tw-readout">
                ${esc(t('view.taxwiz.income.sched_c.net'))}:
                <strong>${esc(fmtMoney(+d.schedule_c.net_profit || 0))}</strong>
            </div>
        </section>

        <section class="tw-card">
            <header><strong>${esc(t('view.taxwiz.income.sched_e.h'))}</strong></header>
            ${moneyField('tw-se-rents', 'view.taxwiz.income.sched_e.rents', d.schedule_e.gross_rents)}
            ${moneyField('tw-se-exp',   'view.taxwiz.income.sched_e.expenses', d.schedule_e.total_expenses)}
            <div class="tw-readout">
                ${esc(t('view.taxwiz.income.sched_e.net'))}:
                <strong>${esc(fmtMoney(+d.schedule_e.net_income || 0))}</strong>
            </div>
        </section>

        <div class="tw-live-summary">${liveSummaryHtml()}</div>
        ${navButtons('adjustments', 'personal')}
    `;

    // W-2 upload via OCR.
    const fileInput = pane.querySelector('#tw-w2-file');
    pane.querySelector('#tw-w2-upload').addEventListener('click', () => fileInput.click());
    fileInput.addEventListener('change', async e => {
        const f = e.target.files[0];
        if (!f) return;
        showToast(t('view.taxwiz.toast.ocr_running'), { level: 'info' });
        try {
            const r = await api.uploadTaxForm(f, STATE.year);
            if (!viewIsCurrent(tok)) return;
            if (r.kind === 'w2') {
                const w2 = {
                    employer_name: r.party_name || '',
                    box_1_wages: +r.payload.box_1 || 0,
                    box_2_federal_income_tax_withheld: +r.payload.box_2 || 0,
                    box_3_ss_wages: +r.payload.box_3 || 0,
                    box_4_ss_tax_withheld: +r.payload.box_4 || 0,
                    box_5_medicare_wages: +r.payload.box_5 || 0,
                    box_6_medicare_tax_withheld: +r.payload.box_6 || 0,
                    box_17_state_income_tax: +r.payload.box_17 || 0,
                };
                d.w2s.push(w2);
                showToast(t('view.taxwiz.toast.w2_imported', { conf: Math.round(r.confidence * 100) }), { level: 'success' });
            } else if (r.kind === 'form1099_int') {
                d.interest_income = (+d.interest_income || 0) + (+r.payload.box_1 || 0);
                showToast(t('view.taxwiz.toast.int_imported'), { level: 'success' });
            } else if (r.kind === 'form1099_nec') {
                d.schedule_c.gross_receipts = (+d.schedule_c.gross_receipts || 0) + (+r.payload.box_1 || 0);
                d.schedule_c.net_profit = (+d.schedule_c.gross_receipts) - (+d.schedule_c.total_expenses || 0);
                showToast(t('view.taxwiz.toast.nec_imported'), { level: 'success' });
            } else if (r.kind === 'form1099_div') {
                d.ordinary_dividends = (+d.ordinary_dividends || 0) + (+r.payload.box_1a || 0);
                d.qualified_dividends = (+d.qualified_dividends || 0) + (+r.payload.box_1b || 0);
                d.net_long_term_capital_gain = (+d.net_long_term_capital_gain || 0) + (+r.payload.box_2a || 0);
                showToast(t('view.taxwiz.toast.div_imported'), { level: 'success' });
            } else {
                showToast(t('view.taxwiz.toast.form_unknown', { kind: r.kind }), { level: 'warn' });
            }
            await doSave(mount, tok, 'income.tax_form_imported');
            renderIncome(pane, mount, tok);
        } catch (err) {
            showToast(t('view.taxwiz.err.ocr', { err: err.message }), { level: 'error' });
        } finally {
            fileInput.value = '';
        }
    });

    pane.querySelector('#tw-w2-add').addEventListener('click', () => {
        d.w2s.push({
            employer_name: '',
            box_1_wages: 0, box_2_federal_income_tax_withheld: 0,
            box_3_ss_wages: 0, box_4_ss_tax_withheld: 0,
            box_5_medicare_wages: 0, box_6_medicare_tax_withheld: 0,
            box_17_state_income_tax: 0,
        });
        renderIncome(pane, mount, tok);
    });

    // Delegated W-2 row edits.
    pane.querySelector('#tw-w2-list').addEventListener('input', e => {
        const tr = e.target.closest('[data-w2-idx]');
        if (!tr) return;
        const i = +tr.dataset.w2Idx;
        const field = e.target.dataset.field;
        if (!field) return;
        if (field === 'employer_name') d.w2s[i][field] = e.target.value;
        else d.w2s[i][field] = e.target.value === '' ? 0 : Number(e.target.value);
        scheduleSave(mount, tok, `income.w2_${i}_${field}`);
    });
    pane.querySelector('#tw-w2-list').addEventListener('click', e => {
        if (!e.target.classList.contains('tw-w2-rm')) return;
        const i = +e.target.dataset.i;
        d.w2s.splice(i, 1);
        renderIncome(pane, mount, tok);
        scheduleSave(mount, tok, 'income.w2_removed');
    });

    bindMoney(pane, 'tw-interest',  v => d.interest_income = v, mount, tok, 'income.interest');
    bindMoney(pane, 'tw-div-ord',   v => d.ordinary_dividends = v, mount, tok, 'income.div_ord');
    bindMoney(pane, 'tw-div-q',     v => d.qualified_dividends = v, mount, tok, 'income.div_q');
    bindMoney(pane, 'tw-ltcg',      v => d.net_long_term_capital_gain = v, mount, tok, 'income.ltcg');
    bindMoney(pane, 'tw-other',     v => d.other_income = v, mount, tok, 'income.other');
    bindMoney(pane, 'tw-sc-gross',  v => { d.schedule_c.gross_receipts = v; d.schedule_c.net_profit = (+d.schedule_c.gross_receipts || 0) - (+d.schedule_c.total_expenses || 0); }, mount, tok, 'income.sc_gross');
    bindMoney(pane, 'tw-sc-exp',    v => { d.schedule_c.total_expenses = v; d.schedule_c.net_profit = (+d.schedule_c.gross_receipts || 0) - (+d.schedule_c.total_expenses || 0); }, mount, tok, 'income.sc_exp');
    bindMoney(pane, 'tw-se-rents',  v => { d.schedule_e.gross_rents = v; d.schedule_e.net_income = (+d.schedule_e.gross_rents || 0) - (+d.schedule_e.total_expenses || 0); }, mount, tok, 'income.se_rents');
    bindMoney(pane, 'tw-se-exp',    v => { d.schedule_e.total_expenses = v; d.schedule_e.net_income = (+d.schedule_e.gross_rents || 0) - (+d.schedule_e.total_expenses || 0); }, mount, tok, 'income.se_exp');

    wireNav(pane, mount, tok);
}

function renderW2List(w2s) {
    if (!w2s || w2s.length === 0) {
        return `<p class="muted small">${esc(t('view.taxwiz.income.w2.empty'))}</p>`;
    }
    return w2s.map((w, i) => `
        <div class="tw-w2-row" data-w2-idx="${i}">
            <input type="text" data-field="employer_name" placeholder="${esc(t('view.taxwiz.income.w2.employer'))}" value="${esc(w.employer_name || '')}">
            <label>Box 1 <input type="number" data-field="box_1_wages" value="${w.box_1_wages || 0}"></label>
            <label>Box 2 <input type="number" data-field="box_2_federal_income_tax_withheld" value="${w.box_2_federal_income_tax_withheld || 0}"></label>
            <label>Box 3 <input type="number" data-field="box_3_ss_wages" value="${w.box_3_ss_wages || 0}"></label>
            <label>Box 5 <input type="number" data-field="box_5_medicare_wages" value="${w.box_5_medicare_wages || 0}"></label>
            <button type="button" class="btn btn-secondary btn-compact tw-w2-rm" data-i="${i}">×</button>
        </div>
    `).join('');
}

// ── Adjustments / Deductions / Credits / Other / Review / Download ───

function renderAdjustments(pane, mount, tok) {
    const d = STATE.draft;
    pane.innerHTML = `
        <h3>${esc(t('view.taxwiz.adjustments.h3'))}</h3>
        <p class="muted small">${esc(t('view.taxwiz.adjustments.subtitle'))}</p>
        ${moneyField('tw-hsa', 'view.taxwiz.adjustments.hsa', d.hsa_deduction)}
        ${moneyField('tw-ira', 'view.taxwiz.adjustments.ira', d.ira_deduction)}
        ${moneyField('tw-stud', 'view.taxwiz.adjustments.student_loan', d.student_loan_interest)}
        ${moneyField('tw-other-adj', 'view.taxwiz.adjustments.other', d.other_adjustments)}
        <div class="tw-live-summary">${liveSummaryHtml()}</div>
        ${navButtons('deductions', 'income')}
    `;
    bindMoney(pane, 'tw-hsa',       v => d.hsa_deduction = v, mount, tok, 'adj.hsa');
    bindMoney(pane, 'tw-ira',       v => d.ira_deduction = v, mount, tok, 'adj.ira');
    bindMoney(pane, 'tw-stud',      v => d.student_loan_interest = v, mount, tok, 'adj.student');
    bindMoney(pane, 'tw-other-adj', v => d.other_adjustments = v, mount, tok, 'adj.other');
    wireNav(pane, mount, tok);
}

function renderDeductions(pane, mount, tok) {
    const d = STATE.draft;
    const it = d.itemized;
    const stdAmount = { single: 15000, mfj: 30000, mfs: 15000, hoh: 22500 }[d.status] || 15000;
    const itTotal = itemizedTotal(it);
    pane.innerHTML = `
        <h3>${esc(t('view.taxwiz.deductions.h3'))}</h3>
        <p class="muted small">${esc(t('view.taxwiz.deductions.subtitle'))}</p>

        <div class="tw-cmp">
            <div class="tw-cmp-card ${itTotal <= stdAmount || d.force_standard_deduction ? 'tw-cmp-win' : ''}">
                <strong>${esc(t('view.taxwiz.deductions.standard'))}</strong>
                <span>${esc(fmtMoney(stdAmount))}</span>
            </div>
            <div class="tw-cmp-card ${itTotal > stdAmount && !d.force_standard_deduction ? 'tw-cmp-win' : ''}">
                <strong>${esc(t('view.taxwiz.deductions.itemized'))}</strong>
                <span>${esc(fmtMoney(itTotal))}</span>
            </div>
        </div>

        <label class="tw-field"><input type="checkbox" id="tw-force-std" ${d.force_standard_deduction ? 'checked' : ''}>
            <span>${esc(t('view.taxwiz.deductions.force_std'))}</span></label>

        <section class="tw-card">
            <header><strong>${esc(t('view.taxwiz.deductions.itemized_h'))}</strong></header>
            ${moneyField('tw-med',  'view.taxwiz.deductions.medical', it.medical_over_7_5_pct_agi)}
            ${moneyField('tw-salt', 'view.taxwiz.deductions.salt', it.state_and_local_taxes_capped_at_10k)}
            ${moneyField('tw-mort', 'view.taxwiz.deductions.mortgage', it.mortgage_interest)}
            ${moneyField('tw-char', 'view.taxwiz.deductions.charitable', it.charitable_gifts)}
            ${moneyField('tw-cas',  'view.taxwiz.deductions.casualty', it.casualty_losses)}
        </section>
        <div class="tw-live-summary">${liveSummaryHtml()}</div>
        ${navButtons('credits', 'adjustments')}
    `;
    pane.querySelector('#tw-force-std').addEventListener('change', e => {
        d.force_standard_deduction = e.target.checked;
        scheduleSave(mount, tok, 'deductions.force_std');
        // Re-render comparison cards immediately.
        setTimeout(() => renderDeductions(pane, mount, tok), 450);
    });
    bindMoney(pane, 'tw-med',  v => it.medical_over_7_5_pct_agi = v, mount, tok, 'deductions.medical');
    bindMoney(pane, 'tw-salt', v => it.state_and_local_taxes_capped_at_10k = v, mount, tok, 'deductions.salt');
    bindMoney(pane, 'tw-mort', v => it.mortgage_interest = v, mount, tok, 'deductions.mortgage');
    bindMoney(pane, 'tw-char', v => it.charitable_gifts = v, mount, tok, 'deductions.charitable');
    bindMoney(pane, 'tw-cas',  v => it.casualty_losses = v, mount, tok, 'deductions.casualty');
    wireNav(pane, mount, tok);
}

function renderCredits(pane, mount, tok) {
    const d = STATE.draft;
    pane.innerHTML = `
        <h3>${esc(t('view.taxwiz.credits.h3'))}</h3>
        <p class="muted small">${esc(t('view.taxwiz.credits.subtitle'))}</p>
        <label class="tw-field"><span>${esc(t('view.taxwiz.credits.ctc_count'))}</span>
            <input type="number" min="0" id="tw-ctc" value="${d.qualifying_children_under_17 || 0}"></label>
        <label class="tw-field"><span>${esc(t('view.taxwiz.credits.odc_count'))}</span>
            <input type="number" min="0" id="tw-odc" value="${d.other_dependents || 0}"></label>
        ${moneyField('tw-eitc',  'view.taxwiz.credits.eitc_claim', d.eitc_claim, t('view.taxwiz.credits.eitc_hint'))}
        ${moneyField('tw-paid',  'view.taxwiz.credits.estimated_paid', d.estimated_tax_payments)}
        <div class="tw-live-summary">${liveSummaryHtml()}</div>
        ${navButtons('other_taxes', 'deductions')}
    `;
    pane.querySelector('#tw-ctc').addEventListener('input', e => {
        d.qualifying_children_under_17 = +e.target.value || 0;
        scheduleSave(mount, tok, 'credits.ctc_count');
    });
    pane.querySelector('#tw-odc').addEventListener('input', e => {
        d.other_dependents = +e.target.value || 0;
        scheduleSave(mount, tok, 'credits.odc_count');
    });
    bindMoney(pane, 'tw-eitc', v => d.eitc_claim = v, mount, tok, 'credits.eitc');
    bindMoney(pane, 'tw-paid', v => d.estimated_tax_payments = v, mount, tok, 'credits.estimated_paid');
    wireNav(pane, mount, tok);
}

function renderOtherTaxes(pane, mount, tok) {
    const r = STATE.result || {};
    pane.innerHTML = `
        <h3>${esc(t('view.taxwiz.other.h3'))}</h3>
        <p class="muted small">${esc(t('view.taxwiz.other.subtitle'))}</p>
        <div class="tw-readout-grid">
            <div><span>${esc(t('view.taxwiz.other.se_base'))}</span><strong>${esc(fmtMoney(+(r.se_tax || {}).se_base || 0))}</strong></div>
            <div><span>${esc(t('view.taxwiz.other.ss'))}</span><strong>${esc(fmtMoney(+(r.se_tax || {}).ss_tax || 0))}</strong></div>
            <div><span>${esc(t('view.taxwiz.other.medicare'))}</span><strong>${esc(fmtMoney(+(r.se_tax || {}).medicare_tax || 0))}</strong></div>
            <div><span>${esc(t('view.taxwiz.other.add_medicare'))}</span><strong>${esc(fmtMoney(+(r.se_tax || {}).additional_medicare_tax || 0))}</strong></div>
            <div><span>${esc(t('view.taxwiz.other.se_total'))}</span><strong>${esc(fmtMoney(+(r.se_tax || {}).total || 0))}</strong></div>
            <div><span>${esc(t('view.taxwiz.other.se_above_line'))}</span><strong>${esc(fmtMoney(+(r.se_tax || {}).above_line_deduction || 0))}</strong></div>
        </div>
        <div class="tw-live-summary">${liveSummaryHtml()}</div>
        ${navButtons('review', 'credits')}
    `;
    wireNav(pane, mount, tok);
}

function renderReview(pane, mount, tok) {
    const d = STATE.draft;
    const r = STATE.result || {};
    const review = (label, value) => `<div><span>${esc(label)}</span><strong>${esc(value)}</strong></div>`;
    pane.innerHTML = `
        <h3>${esc(t('view.taxwiz.review.h3'))}</h3>
        <p class="muted small">${esc(t('view.taxwiz.review.subtitle'))}</p>
        <div class="tw-review">
            ${review(t('view.taxwiz.live.agi'), fmtMoney(+r.agi || 0))}
            ${review(t('view.taxwiz.review.deduction'), `${r.deduction_label || '—'} ${fmtMoney(+r.deduction_used || 0)}`)}
            ${review(t('view.taxwiz.review.qbi'), fmtMoney(+r.qbi_deduction || 0))}
            ${review(t('view.taxwiz.live.taxable'), fmtMoney(+r.taxable_income || 0))}
            ${review(t('view.taxwiz.review.ordinary_tax'), fmtMoney(+r.ordinary_tax || 0))}
            ${review(t('view.taxwiz.review.se_tax'), fmtMoney(+(r.se_tax || {}).total || 0))}
            ${review(t('view.taxwiz.review.ctc'), fmtMoney(+(r.ctc || {}).total || 0))}
            ${review(t('view.taxwiz.review.after_credits'), fmtMoney(+r.tax_after_credits || 0))}
            ${review(t('view.taxwiz.review.payments'), fmtMoney(+r.total_payments || 0))}
        </div>
        <div class="tw-review-final">
            <div class="tw-refund-block">
                <span>${esc(t('view.taxwiz.live.refund'))}</span>
                <strong class="tw-refund">${esc(fmtMoney(+r.refund_due || 0))}</strong>
            </div>
            <div class="tw-owed-block">
                <span>${esc(t('view.taxwiz.live.owed'))}</span>
                <strong class="tw-owed">${esc(fmtMoney(+r.tax_owed || 0))}</strong>
            </div>
        </div>
        ${r.qbi_needs_manual_review ? `<div class="tw-warn">${esc(t('view.taxwiz.review.qbi_warn'))}</div>` : ''}

        <section class="tw-card">
            <header><strong>${esc(t('view.taxwiz.safe_harbor.h'))}</strong>
                <select id="tw-sh-quarter">
                    <option value="1">${esc(t('view.taxwiz.safe_harbor.q1'))}</option>
                    <option value="2">${esc(t('view.taxwiz.safe_harbor.q2'))}</option>
                    <option value="3">${esc(t('view.taxwiz.safe_harbor.q3'))}</option>
                    <option value="4">${esc(t('view.taxwiz.safe_harbor.q4'))}</option>
                </select>
                <button type="button" id="tw-sh-run" class="btn btn-secondary btn-compact">
                    ${esc(t('view.taxwiz.safe_harbor.run'))}
                </button>
            </header>
            <p class="muted small">${esc(t('view.taxwiz.safe_harbor.hint'))}</p>
            <div id="tw-sh-out"></div>
        </section>

        <section class="tw-card">
            <header><strong>${esc(t('view.taxwiz.what_if.h'))}</strong>
                <select id="tw-wi-path">
                    <option value="ira_deduction">${esc(t('view.taxwiz.what_if.ira'))}</option>
                    <option value="hsa_deduction">${esc(t('view.taxwiz.what_if.hsa'))}</option>
                    <option value="qualifying_children_under_17">${esc(t('view.taxwiz.what_if.kids'))}</option>
                    <option value="estimated_tax_payments">${esc(t('view.taxwiz.what_if.estimated'))}</option>
                    <option value="schedule_c_net_profit">${esc(t('view.taxwiz.what_if.sched_c'))}</option>
                    <option value="w2_box_1_wages_total">${esc(t('view.taxwiz.what_if.wages'))}</option>
                </select>
                <input type="number" id="tw-wi-value" step="0.01" placeholder="${esc(t('view.taxwiz.what_if.placeholder'))}" />
                <button type="button" id="tw-wi-run" class="btn btn-secondary btn-compact">
                    ${esc(t('view.taxwiz.what_if.run'))}
                </button>
            </header>
            <p class="muted small">${esc(t('view.taxwiz.what_if.hint'))}</p>
            <div id="tw-wi-out"></div>
        </section>

        ${navButtons('download', 'other_taxes')}
    `;
    wireNav(pane, mount, tok);
    wireSafeHarbor(pane, tok);
    wireWhatIf(pane, tok);
}

async function wireSafeHarbor(pane, tok) {
    pane.querySelector('#tw-sh-run').addEventListener('click', async () => {
        const q = pane.querySelector('#tw-sh-quarter').value;
        const out = pane.querySelector('#tw-sh-out');
        out.innerHTML = `<div class="muted small">${esc(t('common.loading'))}</div>`;
        try {
            const r = await api.taxSafeHarbor(STATE.year, { quarter: q });
            if (!viewIsCurrent(tok)) return;
            const cls = +r.additional_due_this_quarter > 0 ? 'tw-owed' : 'tw-refund';
            out.innerHTML = `
                <div class="tw-sh-grid">
                    <div><span>${esc(t('view.taxwiz.safe_harbor.binding'))}</span>
                        <strong>${esc(r.binding_harbor === 'prior_year'
                            ? t('view.taxwiz.safe_harbor.binding_prior')
                            : t('view.taxwiz.safe_harbor.binding_current'))}</strong></div>
                    <div><span>${esc(t('view.taxwiz.safe_harbor.annual_floor'))}</span>
                        <strong>${esc(fmtMoney(+r.annual_floor))}</strong></div>
                    <div><span>${esc(t('view.taxwiz.safe_harbor.cumulative_target'))}</span>
                        <strong>${esc(fmtMoney(+r.cumulative_target))}</strong></div>
                    <div><span>${esc(t('view.taxwiz.safe_harbor.paid_to_date'))}</span>
                        <strong>${esc(fmtMoney(+r.paid_to_date))}</strong></div>
                    <div><span>${esc(t('view.taxwiz.safe_harbor.due'))}</span>
                        <strong class="${cls}">${esc(fmtMoney(+r.additional_due_this_quarter))}</strong></div>
                    <div><span>${esc(t('view.taxwiz.safe_harbor.surplus'))}</span>
                        <strong class="tw-refund">${esc(fmtMoney(+r.surplus))}</strong></div>
                </div>
                ${r.prior_year_high_income ? `<p class="muted small">${esc(t('view.taxwiz.safe_harbor.high_income_note'))}</p>` : ''}
            `;
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            out.innerHTML = `<div class="err">${esc(t('view.taxwiz.safe_harbor.err', { err: e.message }))}</div>`;
        }
    });
}

async function wireWhatIf(pane, tok) {
    pane.querySelector('#tw-wi-run').addEventListener('click', async () => {
        const path = pane.querySelector('#tw-wi-path').value;
        const value = Number(pane.querySelector('#tw-wi-value').value);
        if (!Number.isFinite(value)) return;
        const out = pane.querySelector('#tw-wi-out');
        out.innerHTML = `<div class="muted small">${esc(t('common.loading'))}</div>`;
        try {
            const r = await api.taxWhatIf(STATE.year, { path, value: String(value) });
            if (!viewIsCurrent(tok)) return;
            const netCls = +r.net_dollar_change_in_pocket >= 0 ? 'tw-refund' : 'tw-owed';
            const sign = +r.net_dollar_change_in_pocket >= 0 ? '+' : '';
            out.innerHTML = `
                <div class="tw-wi-grid">
                    <div><span>${esc(t('view.taxwiz.what_if.delta_agi'))}</span>
                        <strong>${esc(fmtMoney(+r.delta_agi))}</strong></div>
                    <div><span>${esc(t('view.taxwiz.what_if.delta_taxable'))}</span>
                        <strong>${esc(fmtMoney(+r.delta_taxable_income))}</strong></div>
                    <div><span>${esc(t('view.taxwiz.what_if.delta_tax'))}</span>
                        <strong>${esc(fmtMoney(+r.delta_total_tax))}</strong></div>
                    <div><span>${esc(t('view.taxwiz.what_if.delta_refund'))}</span>
                        <strong class="tw-refund">${esc(fmtMoney(+r.delta_refund_due))}</strong></div>
                    <div><span>${esc(t('view.taxwiz.what_if.delta_owed'))}</span>
                        <strong class="tw-owed">${esc(fmtMoney(+r.delta_tax_owed))}</strong></div>
                </div>
                <div class="tw-wi-bottom">
                    <span>${esc(t('view.taxwiz.what_if.net'))}</span>
                    <strong class="${netCls}">${sign}${esc(fmtMoney(+r.net_dollar_change_in_pocket))}</strong>
                </div>
            `;
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            out.innerHTML = `<div class="err">${esc(t('view.taxwiz.what_if.err', { err: e.message }))}</div>`;
        }
    });
}

function renderDownload(pane, mount, tok) {
    pane.innerHTML = `
        <h3>${esc(t('view.taxwiz.download.h3'))}</h3>
        <p>${esc(t('view.taxwiz.download.subtitle'))}</p>
        <p class="muted small">${esc(t('view.taxwiz.download.disclaimer'))}</p>
        <a href="${esc(api.taxReturnPdfUrl(STATE.year))}" target="_blank" rel="noopener" class="btn btn-primary">
            ${esc(t('view.taxwiz.download.pdf_btn'))}
        </a>
        <p class="muted small tw-download-next">${esc(t('view.taxwiz.download.next_step'))}</p>
        <a href="https://www.irs.gov/e-file-providers/free-file-fillable-forms" target="_blank" rel="noopener" class="btn btn-secondary btn-compact">
            ${esc(t('view.taxwiz.download.irs_link'))}
        </a>
        ${navButtons('', 'review')}
    `;
    wireNav(pane, mount, tok);
}

// ── primitive widgets ─────────────────────────────────────────────────

function moneyField(id, labelKey, value, hint) {
    const v = value == null ? '' : (+value === 0 ? 0 : value);
    return `<label class="tw-field">
        <span>${esc(t(labelKey))}</span>
        <input type="number" step="0.01" id="${id}" value="${v}">
        ${hint ? `<small class="muted">${esc(hint)}</small>` : ''}
    </label>`;
}

function bindMoney(pane, id, setter, mount, tok, label) {
    const el = pane.querySelector('#' + id);
    if (!el) return;
    el.addEventListener('input', e => {
        const v = e.target.value === '' ? 0 : Number(e.target.value);
        setter(Number.isFinite(v) ? v : 0);
        scheduleSave(mount, tok, label);
    });
}

function navButtons(nextSec, prevSec) {
    return `<nav class="tw-nav">
        ${prevSec ? `<button type="button" class="btn btn-secondary tw-back" data-go="${prevSec}">← ${esc(t('common.back'))}</button>` : ''}
        ${nextSec ? `<button type="button" class="btn btn-primary tw-next" data-go="${nextSec}">${esc(t('common.next'))} →</button>` : ''}
    </nav>`;
}

function wireNav(pane, mount, tok) {
    pane.querySelectorAll('button[data-go]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.section = btn.dataset.go;
            location.hash = `#file-taxes?year=${STATE.year}&sec=${STATE.section}`;
            renderRail(mount);
            renderActiveSection(mount, tok);
        });
    });
}
