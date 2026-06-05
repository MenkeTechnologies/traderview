// IRC § 165(d) — Gambling Losses.
// Deductible ONLY to extent of GAMBLING WINNINGS (no excess deduction).
// Pre-TCJA: itemized only (Schedule A). Post-TCJA 2017: still itemized for non-professional.
// Professional gambler: trade/business — Schedule C; gambling expenses + losses (limited to winnings since 2018).
// TCJA § 165(d) clarification: expenses + losses combined limited to winnings (Mayo v. Comm'r limited).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    total_winnings: 0,
    total_losses: 0,
    travel_expenses: 0,
    is_professional: false,
    is_itemized: true,
    standard_deduction_2025: 14_600,
    other_itemized: 0,
    marginal_rate: 24,
    state_tax_rate: 5,
    w2g_received: false,
    is_amateur: true,
    has_records: true,
    casino_win_loss_statement: false,
};

export async function renderSection165D(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s165d.h1.title">// § 165(d) GAMBLING LOSSES</span></h1>
        <p class="muted small" data-i18n="view.s165d.hint.intro">
            Gambling LOSSES deductible ONLY to extent of <strong>GAMBLING WINNINGS</strong>. Excess losses
            NOT deductible / no carry-forward. <strong>Amateurs:</strong> winnings → Schedule 1 income; losses
            → Schedule A itemized (only if itemizing). <strong>Professionals:</strong> Schedule C trade /
            business — gambling expenses + losses limited to winnings under TCJA 2017. <strong>Mayo v. Comm'r</strong>
            (2018) — pre-TCJA expenses + losses cap to winnings questioned; TCJA codified strict limit.
            <strong>Form W-2G</strong>: payer reports winnings ≥ $600/$1,200/$5,000 thresholds.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s165d.h2.inputs">Inputs</h2>
            <form id="s165d-form" class="inline-form">
                <label><span data-i18n="view.s165d.label.winnings">Total winnings ($)</span>
                    <input type="number" step="0.01" name="total_winnings" value="${state.total_winnings}"></label>
                <label><span data-i18n="view.s165d.label.losses">Total losses ($)</span>
                    <input type="number" step="0.01" name="total_losses" value="${state.total_losses}"></label>
                <label><span data-i18n="view.s165d.label.travel">Travel + lodging expenses ($)</span>
                    <input type="number" step="0.01" name="travel_expenses" value="${state.travel_expenses}"></label>
                <label><span data-i18n="view.s165d.label.professional">Professional gambler (Schedule C)?</span>
                    <input type="checkbox" name="is_professional" ${state.is_professional ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165d.label.itemized">Itemizing (Schedule A)?</span>
                    <input type="checkbox" name="is_itemized" ${state.is_itemized ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165d.label.std_ded">Standard deduction 2025 ($)</span>
                    <input type="number" step="0.01" name="standard_deduction_2025" value="${state.standard_deduction_2025}"></label>
                <label><span data-i18n="view.s165d.label.other_item">Other itemized deductions ($)</span>
                    <input type="number" step="0.01" name="other_itemized" value="${state.other_itemized}"></label>
                <label><span data-i18n="view.s165d.label.marginal">Marginal federal rate %</span>
                    <input type="number" step="0.1" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.s165d.label.state_rate">State tax rate %</span>
                    <input type="number" step="0.1" name="state_tax_rate" value="${state.state_tax_rate}"></label>
                <label><span data-i18n="view.s165d.label.w2g">W-2G received?</span>
                    <input type="checkbox" name="w2g_received" ${state.w2g_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165d.label.amateur">Amateur (vs professional)?</span>
                    <input type="checkbox" name="is_amateur" ${state.is_amateur ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165d.label.records">Adequate records (Rev. Proc. 77-29)?</span>
                    <input type="checkbox" name="has_records" ${state.has_records ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165d.label.casino_stmt">Casino win/loss statement?</span>
                    <input type="checkbox" name="casino_win_loss_statement" ${state.casino_win_loss_statement ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s165d.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s165d-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s165d.h2.amateur_pro">Amateur vs Professional gambler</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s165d.th.feature">Feature</th>
                    <th data-i18n="view.s165d.th.amateur">Amateur</th>
                    <th data-i18n="view.s165d.th.professional">Professional</th>
                </tr></thead>
                <tbody>
                    <tr><td>Winnings reported</td><td>Schedule 1 Line 8</td><td>Schedule C Line 1</td></tr>
                    <tr><td>Losses deducted</td><td>Schedule A (itemize only)</td><td>Schedule C</td></tr>
                    <tr><td>Travel + meals</td><td>NOT deductible</td><td>Schedule C (50% meals)</td></tr>
                    <tr><td>Loss limit</td><td>Up to winnings</td><td>Winnings + travel limited to winnings</td></tr>
                    <tr><td>Excess losses</td><td>LOST (no carryforward)</td><td>LOST (per TCJA 2017)</td></tr>
                    <tr><td>SE tax</td><td>NO</td><td>YES — 15.3% on net</td></tr>
                    <tr><td>NOL creation</td><td>NO</td><td>NO (TCJA limits)</td></tr>
                    <tr><td>QBI § 199A</td><td>N/A</td><td>NO — specified service trade/business excluded</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s165d.h2.session">Session accounting (Notice 2015-21 proposed)</h2>
            <ul class="muted small">
                <li data-i18n="view.s165d.sess.purpose">Net within "session" — discrete period (single day at casino)</li>
                <li data-i18n="view.s165d.sess.formula">Track buy-in + cash out per session</li>
                <li data-i18n="view.s165d.sess.report">Report only NET WIN per session (vs gross-up of every spin)</li>
                <li data-i18n="view.s165d.sess.electronic">Electronic gaming devices: per-machine, per-session</li>
                <li data-i18n="view.s165d.sess.poker">Poker / table games: per-table or per-tournament session</li>
                <li data-i18n="view.s165d.sess.sports">Sports betting: per-bet basis (no session netting)</li>
                <li data-i18n="view.s165d.sess.crypto">Crypto gambling: each bet separate (depending on platform)</li>
                <li data-i18n="view.s165d.sess.documentation">Documentation: time, date, location, amount, type, persons present (Rev. Proc. 77-29)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s165d.h2.w2g">Form W-2G withholding thresholds</h2>
            <ul class="muted small">
                <li data-i18n="view.s165d.w2g.bingo">Bingo: ≥ $1,200 (no withholding required)</li>
                <li data-i18n="view.s165d.w2g.keno">Keno: ≥ $1,500 reduced by wager (no withholding)</li>
                <li data-i18n="view.s165d.w2g.slot">Slots: ≥ $1,200 (no withholding)</li>
                <li data-i18n="view.s165d.w2g.poker_tourn">Poker tournament: ≥ $5,000 reduced by entry fee (withholding 24% if &gt; $5K)</li>
                <li data-i18n="view.s165d.w2g.race">Racing: ≥ $600 + 300:1 (withholding 24%)</li>
                <li data-i18n="view.s165d.w2g.lottery">Lottery / sweepstakes: ≥ $5,000 (withholding 24%)</li>
                <li data-i18n="view.s165d.w2g.sports">Sports betting: ≥ $600 + 300:1 + Form W-2G (state-by-state variability)</li>
                <li data-i18n="view.s165d.w2g.cap">Backup withholding 24% if no TIN provided (vs default 24%)</li>
                <li data-i18n="view.s165d.w2g.copy_irs">Casino files copy with IRS — match to your return</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s165d.h2.records">Recordkeeping (Rev. Proc. 77-29)</h2>
            <ul class="muted small">
                <li data-i18n="view.s165d.rec.diary">Contemporaneous diary / log: date, location, type, amount, persons present</li>
                <li data-i18n="view.s165d.rec.receipts">Receipts: hotel, travel, parking, food (professional only)</li>
                <li data-i18n="view.s165d.rec.casino_card">Casino loyalty card statements (helpful but not authoritative)</li>
                <li data-i18n="view.s165d.rec.bank">Bank withdrawals + ATM receipts</li>
                <li data-i18n="view.s165d.rec.w2g_copy">W-2G copies</li>
                <li data-i18n="view.s165d.rec.cohan">Cohan rule: courts may estimate if records inadequate (but disfavored)</li>
                <li data-i18n="view.s165d.rec.session_log">Session-based log preferred over per-bet detail</li>
                <li data-i18n="view.s165d.rec.audit_risk">Audit risk: high if winnings + losses don't square; IRS cross-checks W-2Gs</li>
            </ul>
        </div>
    `;
    document.getElementById('s165d-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.total_winnings = Number(fd.get('total_winnings')) || 0;
        state.total_losses = Number(fd.get('total_losses')) || 0;
        state.travel_expenses = Number(fd.get('travel_expenses')) || 0;
        state.is_professional = !!fd.get('is_professional');
        state.is_itemized = !!fd.get('is_itemized');
        state.standard_deduction_2025 = Number(fd.get('standard_deduction_2025')) || 0;
        state.other_itemized = Number(fd.get('other_itemized')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0;
        state.state_tax_rate = Number(fd.get('state_tax_rate')) || 0;
        state.w2g_received = !!fd.get('w2g_received');
        state.is_amateur = !!fd.get('is_amateur');
        state.has_records = !!fd.get('has_records');
        state.casino_win_loss_statement = !!fd.get('casino_win_loss_statement');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s165d-output');
    if (!el) return;
    const allowedLosses = Math.min(state.total_losses, state.total_winnings);
    const lostLosses = Math.max(0, state.total_losses - state.total_winnings);
    const itemizedTotal = state.other_itemized + (state.is_itemized ? allowedLosses : 0);
    const itemizesBeats = itemizedTotal > state.standard_deduction_2025;
    const deductionUsed = state.is_professional ? (allowedLosses + Math.min(state.travel_expenses, state.total_winnings - allowedLosses)) :
        (itemizesBeats ? allowedLosses : 0);
    const netIncome = state.total_winnings - deductionUsed;
    const federalTax = netIncome * (state.marginal_rate / 100);
    const stateTax = netIncome * (state.state_tax_rate / 100);
    const seTax = state.is_professional ? netIncome * 0.153 : 0;
    const totalTax = federalTax + stateTax + seTax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s165d.h2.result">§ 165(d) computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s165d.card.winnings">Winnings reportable</div>
                    <div class="value">$${state.total_winnings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s165d.card.allowed">Losses allowed (≤ winnings)</div>
                    <div class="value">$${allowedLosses.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s165d.card.lost">Losses lost (excess)</div>
                    <div class="value">$${lostLosses.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${itemizesBeats ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s165d.card.itemize">Itemizing beats SD?</div>
                    <div class="value">${itemizesBeats ? esc(t('view.s165d.status.yes')) : esc(t('view.s165d.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s165d.card.deduction">Deduction used</div>
                    <div class="value">$${deductionUsed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s165d.card.net">Net income reported</div>
                    <div class="value">$${netIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s165d.card.fed_tax">Fed tax</div>
                    <div class="value">$${federalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s165d.card.total">Total tax cost</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!state.is_professional && !itemizesBeats ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s165d.no_itemize_note">
                    Standard deduction beats itemizing — gambling losses DISALLOWED (cannot reduce winnings).
                    Full $${state.total_winnings.toLocaleString()} taxable. Common trap: amateur gambler with
                    high standard deduction loses ALL benefit of losses. Consider professional status if
                    activity is consistent + profit-motive demonstrable (5 Groetzinger factors).
                </p>
            ` : ''}
        </div>
    `;
}
