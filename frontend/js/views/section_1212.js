// IRC § 1212 — Capital Loss Carryback / Carryforward.
// Individuals: $3,000/yr ordinary-income offset ($1,500 MFS); excess CARRIES FORWARD INDEFINITELY.
// Character preserved (ST/LT). C-corps: 3-yr CARRYBACK + 5-yr CARRYFORWARD, all ST character.
// Death of taxpayer: unused carryforward LOST (does not pass to estate / heirs).

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const ORDINARY_OFFSET_LIMIT = 3_000;
const MFS_LIMIT = 1_500;

let state = {
    filer_type: 'individual',
    filing_status: 'single',
    st_loss_current: 0,
    lt_loss_current: 0,
    st_gain_current: 0,
    lt_gain_current: 0,
    st_carryforward_prior: 0,
    lt_carryforward_prior: 0,
    marginal_rate: 0.32,
    ltcg_rate: 0.20,
};

export async function renderSection1212(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1212.h1.title">// § 1212 CAP LOSS CARRYFORWARD</span></h1>
        <p class="muted small" data-i18n="view.s1212.hint.intro">
            <strong>Individuals:</strong> $3,000/yr ordinary-income offset ($1,500 MFS); excess
            CARRIES FORWARD INDEFINITELY. Character (ST/LT) preserved. <strong>C-corps:</strong>
            3-yr CARRYBACK + 5-yr CARRYFORWARD, ALL ST character. <strong>Death</strong> of
            taxpayer: unused carryforward LOST (doesn't pass to estate). <strong>§ 475(f) trader
            MTM:</strong> ordinary loss instead — no $3k cap, no 5-yr limit (corp).
            <strong>Bankruptcy estate:</strong> uses Code § 1398 separate filing.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1212.h2.inputs">Inputs</h2>
            <form id="s1212-form" class="inline-form">
                <label><span data-i18n="view.s1212.label.filer">Filer type</span>
                    <select name="filer_type">
                        <option value="individual" ${state.filer_type === 'individual' ? 'selected' : ''}>Individual</option>
                        <option value="c_corp" ${state.filer_type === 'c_corp' ? 'selected' : ''}>C-corp</option>
                        <option value="estate_trust" ${state.filer_type === 'estate_trust' ? 'selected' : ''}>Estate / Trust</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1212.label.filing_status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / HoH</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1212.label.st_loss">ST capital loss current ($)</span>
                    <input type="number" step="0.01" name="st_loss_current" value="${state.st_loss_current}"></label>
                <label><span data-i18n="view.s1212.label.lt_loss">LT capital loss current ($)</span>
                    <input type="number" step="0.01" name="lt_loss_current" value="${state.lt_loss_current}"></label>
                <label><span data-i18n="view.s1212.label.st_gain">ST capital gain current ($)</span>
                    <input type="number" step="0.01" name="st_gain_current" value="${state.st_gain_current}"></label>
                <label><span data-i18n="view.s1212.label.lt_gain">LT capital gain current ($)</span>
                    <input type="number" step="0.01" name="lt_gain_current" value="${state.lt_gain_current}"></label>
                <label><span data-i18n="view.s1212.label.st_carry">ST carryforward from prior ($)</span>
                    <input type="number" step="0.01" name="st_carryforward_prior" value="${state.st_carryforward_prior}"></label>
                <label><span data-i18n="view.s1212.label.lt_carry">LT carryforward from prior ($)</span>
                    <input type="number" step="0.01" name="lt_carryforward_prior" value="${state.lt_carryforward_prior}"></label>
                <label><span data-i18n="view.s1212.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.s1212.label.ltcg">LTCG %</span>
                    <input type="number" step="0.01" name="ltcg_rate" value="${state.ltcg_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s1212.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1212-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1212.h2.netting">Netting order</h2>
            <ol class="muted small">
                <li data-i18n="view.s1212.net.same_class">Net ST gains × ST losses; net LT gains × LT losses</li>
                <li data-i18n="view.s1212.net.cross_class">Net ST excess loss against LT gain (or vice versa)</li>
                <li data-i18n="view.s1212.net.ordinary_offset">Excess net loss offsets ordinary income up to $3k ($1,500 MFS)</li>
                <li data-i18n="view.s1212.net.character_preserved">Character preserved going forward (ST remains ST)</li>
                <li data-i18n="view.s1212.net.absorb_st_first">$3k absorbs ST loss FIRST (ST taxed at ordinary rate anyway)</li>
                <li data-i18n="view.s1212.net.then_lt">Then absorbs LT loss</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1212.h2.special">Special rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s1212.spec.death">Death of taxpayer: carryforward LOST entirely (not estate, not heirs)</li>
                <li data-i18n="view.s1212.spec.divorce">Divorce: each spouse gets own share of joint carryforward (Rev. Rul. 71-382)</li>
                <li data-i18n="view.s1212.spec.amt">AMT capital loss carryforward separately tracked</li>
                <li data-i18n="view.s1212.spec.section_382">§ 382 ownership change limits corp capital loss carryforward</li>
                <li data-i18n="view.s1212.spec.bankruptcy">Bankruptcy estate: § 1398 + estate uses individual's CF separately</li>
                <li data-i18n="view.s1212.spec.section_1296">PFIC § 1296 MTM losses limited to prior MTM gains; not regular cap loss</li>
            </ul>
        </div>
    `;
    document.getElementById('s1212-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filer_type = fd.get('filer_type');
        state.filing_status = fd.get('filing_status');
        state.st_loss_current = Number(fd.get('st_loss_current')) || 0;
        state.lt_loss_current = Number(fd.get('lt_loss_current')) || 0;
        state.st_gain_current = Number(fd.get('st_gain_current')) || 0;
        state.lt_gain_current = Number(fd.get('lt_gain_current')) || 0;
        state.st_carryforward_prior = Number(fd.get('st_carryforward_prior')) || 0;
        state.lt_carryforward_prior = Number(fd.get('lt_carryforward_prior')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        state.ltcg_rate = Number(fd.get('ltcg_rate')) || 0.20;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1212-output');
    if (!el) return;
    // Net ST first
    const totalStLoss = state.st_loss_current + state.st_carryforward_prior;
    const totalLtLoss = state.lt_loss_current + state.lt_carryforward_prior;
    const stNet = state.st_gain_current - totalStLoss;
    const ltNet = state.lt_gain_current - totalLtLoss;
    let stCarry, ltCarry, ordinaryOffset = 0;
    if (stNet < 0 && ltNet < 0) {
        const absLimit = state.filer_type === 'individual'
            ? (state.filing_status === 'mfs' ? MFS_LIMIT : ORDINARY_OFFSET_LIMIT)
            : 0;
        // Absorb ST first
        ordinaryOffset = Math.min(absLimit, -stNet);
        stCarry = -stNet - ordinaryOffset;
        const remainingAbs = absLimit - ordinaryOffset;
        const ltAbsorbed = Math.min(remainingAbs, -ltNet);
        ordinaryOffset += ltAbsorbed;
        ltCarry = -ltNet - ltAbsorbed;
    } else if (stNet < 0 && ltNet > 0) {
        // Cross-class offset
        const offsetUsed = Math.min(-stNet, ltNet);
        const remainingStLoss = -stNet - offsetUsed;
        const limit = state.filer_type === 'individual' ? (state.filing_status === 'mfs' ? MFS_LIMIT : ORDINARY_OFFSET_LIMIT) : 0;
        ordinaryOffset = Math.min(limit, remainingStLoss);
        stCarry = remainingStLoss - ordinaryOffset;
        ltCarry = 0;
    } else if (stNet > 0 && ltNet < 0) {
        const offsetUsed = Math.min(stNet, -ltNet);
        const remainingLtLoss = -ltNet - offsetUsed;
        const limit = state.filer_type === 'individual' ? (state.filing_status === 'mfs' ? MFS_LIMIT : ORDINARY_OFFSET_LIMIT) : 0;
        ordinaryOffset = Math.min(limit, remainingLtLoss);
        stCarry = 0;
        ltCarry = remainingLtLoss - ordinaryOffset;
    } else {
        stCarry = 0;
        ltCarry = 0;
    }
    const cCorpCarryback = state.filer_type === 'c_corp' ? Math.max(0, totalStLoss + totalLtLoss - state.st_gain_current - state.lt_gain_current) : 0;
    const taxSavings = ordinaryOffset * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1212.h2.result">Carryforward analysis</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1212.card.st_net">Net ST result</div>
                    <div class="value">$${stNet.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1212.card.lt_net">Net LT result</div>
                    <div class="value">$${ltNet.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1212.card.ordinary_offset">Ordinary offset</div>
                    <div class="value">$${ordinaryOffset.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1212.card.st_carry_forward">ST carryforward (next yr)</div>
                    <div class="value">$${stCarry.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1212.card.lt_carry_forward">LT carryforward (next yr)</div>
                    <div class="value">$${ltCarry.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${cCorpCarryback > 0 ? `
                    <div class="card pos">
                        <div class="label" data-i18n="view.s1212.card.ccorp_carryback">C-corp 3-yr carryback</div>
                        <div class="value">$${cCorpCarryback.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card pos">
                    <div class="label" data-i18n="view.s1212.card.tax_saved">Year-1 tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${stCarry + ltCarry > 100_000 ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s1212.note.large_carry">
                    Large carryforward: consider § 475(f) MTM election if TTS-qualified to convert
                    to ordinary loss without $3k cap. Roth conversions in low-income years
                    consume carryforward at favorable rates.
                </p>
            ` : ''}
        </div>
    `;
}
