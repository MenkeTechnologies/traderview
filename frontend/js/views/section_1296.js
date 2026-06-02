// IRC § 1296 — Election of Mark-to-Market for Marketable Stock in PFIC.
// Annual MTM election for PFIC stock — gain/loss recognized annually at FMV.
// Alternative to § 1291 punitive default + § 1295 QEF election.
// "Marketable" = regularly traded on qualified exchange (US + foreign).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    is_pfic: false,
    pfic_classification_test: 'income_75pct',
    passive_income_pct: 0,
    passive_assets_pct: 0,
    is_marketable_stock: false,
    is_regularly_traded: false,
    qualified_exchange: 'NYSE',
    fmv_year_end: 0,
    adjusted_basis_year_end: 0,
    s1296_mtm_election_made: false,
    s1296_election_year: 2024,
    mtm_gain_ordinary: 0,
    mtm_loss_offset: 0,
    s1296_d_inclusion: 0,
    s1296_b_loss_amount: 0,
    has_unreversed_inclusions: 0,
    s1296_c_ordinary_character: true,
    s1296_a_2_unreversed: 0,
    prior_s1291_inclusion: 0,
    s1291_inclusion_remaining: 0,
    is_s1295_qef_election: false,
    s1295_ordinary_passive: 0,
    s1295_capital_gain: 0,
    foreign_country: 'Cayman',
    cusip: '',
    days_held: 0,
    s1297_e_test_active_banking: false,
    s1298_attribution_rules: false,
    s1298_a_5_indirect_ownership: false,
    s1298_e_lookthrough: false,
    s1297_f_active_insurance: false,
    is_excepted_insurance: false,
    s1291_pedigreed_pfic: false,
    purging_distribution: 0,
};

export async function renderSection1296(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1296.h1.title">// § 1296 PFIC MARK-TO-MARKET ELECTION</span></h1>
        <p class="muted small" data-i18n="view.s1296.hint.intro">
            <strong>§ 1296 MTM election</strong> for "marketable" PFIC stock — annual gain INCLUDED
            as ORDINARY income; LOSS deductible as ORDINARY but limited to "unreversed inclusions"
            (cumulative prior MTM gains net of prior MTM losses). <strong>"Marketable" (§ 1296(e)):</strong>
            regularly traded on qualified exchange (US national securities exchange registered under
            1934 Act + foreign qualifying exchanges). <strong>Alternative regimes:</strong> § 1291
            punitive default (excess distribution + 4-year throwback + interest charge),
            <strong>§ 1295 QEF</strong> election (current inclusion of pro-rata share of E&P + ordinary
            for passive + LTCG for capital gain). <strong>§ 1297 PFIC test:</strong> 75% income OR
            50% assets passive (asset test by FMV or basis). <strong>§ 1298(a)(5)</strong> attribution
            via 50%+ owned chain. <strong>Form 8621</strong> mandatory annual reporting.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1296.h2.inputs">Inputs</h2>
            <form id="s1296-form" class="inline-form">
                <label><span data-i18n="view.s1296.label.is_pfic">Is PFIC?</span>
                    <input type="checkbox" name="is_pfic" ${state.is_pfic ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.classification">PFIC classification</span>
                    <select name="pfic_classification_test">
                        <option value="income_75pct" ${state.pfic_classification_test === 'income_75pct' ? 'selected' : ''}>Income test (75%+ passive)</option>
                        <option value="asset_50pct" ${state.pfic_classification_test === 'asset_50pct' ? 'selected' : ''}>Asset test (50%+ passive)</option>
                        <option value="both" ${state.pfic_classification_test === 'both' ? 'selected' : ''}>Both</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1296.label.passive_inc">Passive income %</span>
                    <input type="number" step="0.1" name="passive_income_pct" value="${state.passive_income_pct}"></label>
                <label><span data-i18n="view.s1296.label.passive_asst">Passive assets %</span>
                    <input type="number" step="0.1" name="passive_assets_pct" value="${state.passive_assets_pct}"></label>
                <label><span data-i18n="view.s1296.label.marketable">Marketable?</span>
                    <input type="checkbox" name="is_marketable_stock" ${state.is_marketable_stock ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.traded">Regularly traded?</span>
                    <input type="checkbox" name="is_regularly_traded" ${state.is_regularly_traded ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.exchange">Qualified exchange</span>
                    <select name="qualified_exchange">
                        <option value="NYSE" ${state.qualified_exchange === 'NYSE' ? 'selected' : ''}>NYSE</option>
                        <option value="NASDAQ" ${state.qualified_exchange === 'NASDAQ' ? 'selected' : ''}>NASDAQ</option>
                        <option value="AMEX" ${state.qualified_exchange === 'AMEX' ? 'selected' : ''}>NYSE American</option>
                        <option value="CBOE" ${state.qualified_exchange === 'CBOE' ? 'selected' : ''}>CBOE</option>
                        <option value="LSE" ${state.qualified_exchange === 'LSE' ? 'selected' : ''}>London (LSE)</option>
                        <option value="TSE" ${state.qualified_exchange === 'TSE' ? 'selected' : ''}>Tokyo (TSE)</option>
                        <option value="Euronext" ${state.qualified_exchange === 'Euronext' ? 'selected' : ''}>Euronext</option>
                        <option value="other" ${state.qualified_exchange === 'other' ? 'selected' : ''}>Other qualifying</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1296.label.fmv">FMV year-end ($)</span>
                    <input type="number" step="100" name="fmv_year_end" value="${state.fmv_year_end}"></label>
                <label><span data-i18n="view.s1296.label.basis">Adj basis year-end ($)</span>
                    <input type="number" step="100" name="adjusted_basis_year_end" value="${state.adjusted_basis_year_end}"></label>
                <label><span data-i18n="view.s1296.label.elected">§ 1296 MTM election made?</span>
                    <input type="checkbox" name="s1296_mtm_election_made" ${state.s1296_mtm_election_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.year">Election year</span>
                    <input type="number" step="1" name="s1296_election_year" value="${state.s1296_election_year}"></label>
                <label><span data-i18n="view.s1296.label.gain">MTM gain ordinary ($)</span>
                    <input type="number" step="100" name="mtm_gain_ordinary" value="${state.mtm_gain_ordinary}"></label>
                <label><span data-i18n="view.s1296.label.loss">MTM loss offset ($)</span>
                    <input type="number" step="100" name="mtm_loss_offset" value="${state.mtm_loss_offset}"></label>
                <label><span data-i18n="view.s1296.label.s1296d">§ 1296(d) inclusion ($)</span>
                    <input type="number" step="100" name="s1296_d_inclusion" value="${state.s1296_d_inclusion}"></label>
                <label><span data-i18n="view.s1296.label.s1296b">§ 1296(b) loss ($)</span>
                    <input type="number" step="100" name="s1296_b_loss_amount" value="${state.s1296_b_loss_amount}"></label>
                <label><span data-i18n="view.s1296.label.unreversed">Unreversed inclusions ($)</span>
                    <input type="number" step="100" name="has_unreversed_inclusions" value="${state.has_unreversed_inclusions}"></label>
                <label><span data-i18n="view.s1296.label.ordinary">§ 1296(c) ordinary?</span>
                    <input type="checkbox" name="s1296_c_ordinary_character" ${state.s1296_c_ordinary_character ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.cumulative">Cumulative unreversed ($)</span>
                    <input type="number" step="100" name="s1296_a_2_unreversed" value="${state.s1296_a_2_unreversed}"></label>
                <label><span data-i18n="view.s1296.label.prior_s1291">Prior § 1291 ($)</span>
                    <input type="number" step="100" name="prior_s1291_inclusion" value="${state.prior_s1291_inclusion}"></label>
                <label><span data-i18n="view.s1296.label.s1291_remain">§ 1291 remaining ($)</span>
                    <input type="number" step="100" name="s1291_inclusion_remaining" value="${state.s1291_inclusion_remaining}"></label>
                <label><span data-i18n="view.s1296.label.qef">§ 1295 QEF election?</span>
                    <input type="checkbox" name="is_s1295_qef_election" ${state.is_s1295_qef_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.s1295_pass">QEF passive ($)</span>
                    <input type="number" step="100" name="s1295_ordinary_passive" value="${state.s1295_ordinary_passive}"></label>
                <label><span data-i18n="view.s1296.label.s1295_cap">QEF capital ($)</span>
                    <input type="number" step="100" name="s1295_capital_gain" value="${state.s1295_capital_gain}"></label>
                <label><span data-i18n="view.s1296.label.country">Foreign country</span>
                    <input type="text" name="foreign_country" value="${esc(state.foreign_country)}"></label>
                <label><span data-i18n="view.s1296.label.cusip">CUSIP</span>
                    <input type="text" name="cusip" value="${esc(state.cusip)}"></label>
                <label><span data-i18n="view.s1296.label.days">Days held</span>
                    <input type="number" step="1" name="days_held" value="${state.days_held}"></label>
                <label><span data-i18n="view.s1296.label.bank">§ 1297(e) active banking?</span>
                    <input type="checkbox" name="s1297_e_test_active_banking" ${state.s1297_e_test_active_banking ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.attribution">§ 1298 attribution?</span>
                    <input type="checkbox" name="s1298_attribution_rules" ${state.s1298_attribution_rules ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.indirect">§ 1298(a)(5) indirect?</span>
                    <input type="checkbox" name="s1298_a_5_indirect_ownership" ${state.s1298_a_5_indirect_ownership ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.lookthrough">§ 1298(e) lookthrough?</span>
                    <input type="checkbox" name="s1298_e_lookthrough" ${state.s1298_e_lookthrough ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.insurance">§ 1297(f) active insurance?</span>
                    <input type="checkbox" name="s1297_f_active_insurance" ${state.s1297_f_active_insurance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.excepted_ins">Excepted insurance?</span>
                    <input type="checkbox" name="is_excepted_insurance" ${state.is_excepted_insurance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.pedigreed">Pedigreed PFIC (§ 1291)?</span>
                    <input type="checkbox" name="s1291_pedigreed_pfic" ${state.s1291_pedigreed_pfic ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1296.label.purging">Purging distribution ($)</span>
                    <input type="number" step="1000" name="purging_distribution" value="${state.purging_distribution}"></label>
                <button class="primary" type="submit" data-i18n="view.s1296.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1296-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1296.h2.pfic_test">§ 1297 PFIC classification (annual)</h2>
            <ol class="muted small">
                <li data-i18n="view.s1296.pfic.income">Income test: 75%+ of gross income is PASSIVE</li>
                <li data-i18n="view.s1296.pfic.asset">Asset test: 50%+ of assets (FMV or basis) produce passive income</li>
                <li data-i18n="view.s1296.pfic.either_test">EITHER test triggers PFIC classification</li>
                <li data-i18n="view.s1296.pfic.passive_examples">Passive: interest, dividends, royalties, rents, annuities</li>
                <li data-i18n="view.s1296.pfic.s1297_b_2">§ 1297(b)(2) — exclude active rents + dividends from operating subsidiary</li>
                <li data-i18n="view.s1296.pfic.s1297_e">§ 1297(e) — banking + securities dealer exception</li>
                <li data-i18n="view.s1296.pfic.s1297_f">§ 1297(f) — active insurance exception (25% test)</li>
                <li data-i18n="view.s1296.pfic.s1298_b_1">§ 1298(b)(1) — once PFIC, ALWAYS PFIC (until purged)</li>
                <li data-i18n="view.s1296.pfic.start_up">"Start-up" exception (1st year of operations only)</li>
                <li data-i18n="view.s1296.pfic.qbi_4year">"QBC" exception — 4-year look-back disregard</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1296.h2.three_regimes">3 PFIC regimes</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s1296.tbl.regime">Regime</th><th data-i18n="view.s1296.tbl.treatment">Treatment</th><th data-i18n="view.s1296.tbl.character">Character</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s1296.tbl.s1291">§ 1291 default (excess distribution)</td><td data-i18n="view.s1296.tbl.s1291_treat">Throwback over holding period + interest charge + highest rate</td><td>Ordinary</td></tr>
                    <tr><td data-i18n="view.s1296.tbl.s1295">§ 1295 QEF election</td><td data-i18n="view.s1296.tbl.s1295_treat">Annual pro-rata E&P inclusion</td><td>Ordinary (passive) + LTCG (capital)</td></tr>
                    <tr><td data-i18n="view.s1296.tbl.s1296">§ 1296 MTM election</td><td data-i18n="view.s1296.tbl.s1296_treat">Annual FMV mark; gain ordinary + loss limited to unreversed</td><td>Ordinary</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1296.h2.mtm_mechanics">§ 1296 MTM mechanics</h2>
            <ol class="muted small">
                <li data-i18n="view.s1296.mtm.election">Election made by attaching statement to timely-filed return for first year</li>
                <li data-i18n="view.s1296.mtm.binding">BINDING for that year + all subsequent years (until stock sold or election revoked with IRS consent)</li>
                <li data-i18n="view.s1296.mtm.fmv">Year-end FMV - adjusted basis = MTM gain or loss</li>
                <li data-i18n="view.s1296.mtm.gain_ordinary">GAIN: ORDINARY income (no LTCG even if held &gt; 1 year)</li>
                <li data-i18n="view.s1296.mtm.loss_limited">LOSS: ORDINARY deduction limited to "unreversed inclusions" (cumulative prior gains minus prior losses)</li>
                <li data-i18n="view.s1296.mtm.basis_adjusted">Basis ADJUSTED by gain/loss recognized — no double recognition</li>
                <li data-i18n="view.s1296.mtm.gain_first_year">First-year election: catch-up — recognize built-in gain through purging</li>
                <li data-i18n="view.s1296.mtm.coordination_s1291">Coordinates with § 1291: prior years treated as § 1291 if not pedigreed QEF/MTM</li>
                <li data-i18n="view.s1296.mtm.form_8621">Form 8621 reports election + annual MTM gain/loss</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1296.h2.marketable">"Marketable" qualified exchanges</h2>
            <ul class="muted small">
                <li data-i18n="view.s1296.mkt.us">US national securities exchanges (NYSE, NASDAQ, NYSE American)</li>
                <li data-i18n="view.s1296.mkt.foreign">Foreign qualifying exchanges (LSE, Tokyo, Euronext, Toronto, ASX, Hong Kong, etc.)</li>
                <li data-i18n="view.s1296.mkt.regs">Reg § 1.1296-2 defines "qualified exchange" list</li>
                <li data-i18n="view.s1296.mkt.regularly_traded">"Regularly traded" = 15+ days each quarter OR 1/6 of trading days</li>
                <li data-i18n="view.s1296.mkt.de_minimis">De minimis trading not qualifying — facts &amp; circumstances</li>
                <li data-i18n="view.s1296.mkt.OTC_not">OTC market generally not qualifying (limited exceptions)</li>
                <li data-i18n="view.s1296.mkt.pink_sheets">Pink Sheets / OTCBB not qualified</li>
                <li data-i18n="view.s1296.mkt.adr_ok">ADR (American Depositary Receipt): often qualifying via US exchange</li>
                <li data-i18n="view.s1296.mkt.options">Options + futures NOT "stock" for § 1296 — separate</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1296.h2.election_choice">Choice between regimes</h2>
            <ul class="muted small">
                <li data-i18n="view.s1296.choice.s1291_default">Doing NOTHING → § 1291 default (most punitive)</li>
                <li data-i18n="view.s1296.choice.s1295_qef">§ 1295 QEF: best if PFIC produces capital gain — preserves LTCG rate</li>
                <li data-i18n="view.s1296.choice.s1295_problem">§ 1295 requires PFIC to provide annual statement — many will NOT</li>
                <li data-i18n="view.s1296.choice.s1296_simple">§ 1296 MTM: simpler — no PFIC cooperation needed</li>
                <li data-i18n="view.s1296.choice.s1296_loss">§ 1296 disadvantage: loss limited to unreversed inclusions</li>
                <li data-i18n="view.s1296.choice.s1296_ordinary">§ 1296 disadvantage: gain ordinary even on held &gt; 1 yr</li>
                <li data-i18n="view.s1296.choice.s1296_only_marketable">§ 1296 limited to marketable stock — many small/private PFICs don't qualify</li>
                <li data-i18n="view.s1296.choice.purging">Purging distribution: can convert § 1291 PFIC to § 1295/§ 1296 (catch-up tax)</li>
                <li data-i18n="view.s1296.choice.holding_period">Length of holding: § 1291 interest charge accrues over holding period — bad for long holds</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1296.h2.form_8621">Form 8621 reporting</h2>
            <ul class="muted small">
                <li data-i18n="view.s1296.f.required">Required ANNUALLY by US PFIC shareholder</li>
                <li data-i18n="view.s1296.f.thresholds">2014+: Form 8621 required regardless of value (post-HIRE Act + Reg § 1.1298-1T)</li>
                <li data-i18n="view.s1296.f.exception_5k">$5,000 de minimis exception (Reg § 1.1298-1(b)(3)) — limited circumstances</li>
                <li data-i18n="view.s1296.f.s6038d_dual">May also trigger § 6038D Form 8938 reporting (FATCA)</li>
                <li data-i18n="view.s1296.f.disclosure">Election (§ 1295 / § 1296) reported on Form 8621</li>
                <li data-i18n="view.s1296.f.failure">Failure: SOL never starts (§ 6501(c)(8))</li>
                <li data-i18n="view.s1296.f.s6664_d_4">§ 6664(d)(4) — civil fraud SOL never starts on PFIC items</li>
                <li data-i18n="view.s1296.f.s6038d_combine">Form 8938 + Form 8621 cross-references — different reporting regime</li>
            </ul>
        </div>
    `;
    document.getElementById('s1296-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_pfic = !!fd.get('is_pfic');
        state.pfic_classification_test = fd.get('pfic_classification_test');
        state.passive_income_pct = Number(fd.get('passive_income_pct')) || 0;
        state.passive_assets_pct = Number(fd.get('passive_assets_pct')) || 0;
        state.is_marketable_stock = !!fd.get('is_marketable_stock');
        state.is_regularly_traded = !!fd.get('is_regularly_traded');
        state.qualified_exchange = fd.get('qualified_exchange');
        state.fmv_year_end = Number(fd.get('fmv_year_end')) || 0;
        state.adjusted_basis_year_end = Number(fd.get('adjusted_basis_year_end')) || 0;
        state.s1296_mtm_election_made = !!fd.get('s1296_mtm_election_made');
        state.s1296_election_year = Number(fd.get('s1296_election_year')) || 0;
        state.mtm_gain_ordinary = Number(fd.get('mtm_gain_ordinary')) || 0;
        state.mtm_loss_offset = Number(fd.get('mtm_loss_offset')) || 0;
        state.s1296_d_inclusion = Number(fd.get('s1296_d_inclusion')) || 0;
        state.s1296_b_loss_amount = Number(fd.get('s1296_b_loss_amount')) || 0;
        state.has_unreversed_inclusions = Number(fd.get('has_unreversed_inclusions')) || 0;
        state.s1296_c_ordinary_character = !!fd.get('s1296_c_ordinary_character');
        state.s1296_a_2_unreversed = Number(fd.get('s1296_a_2_unreversed')) || 0;
        state.prior_s1291_inclusion = Number(fd.get('prior_s1291_inclusion')) || 0;
        state.s1291_inclusion_remaining = Number(fd.get('s1291_inclusion_remaining')) || 0;
        state.is_s1295_qef_election = !!fd.get('is_s1295_qef_election');
        state.s1295_ordinary_passive = Number(fd.get('s1295_ordinary_passive')) || 0;
        state.s1295_capital_gain = Number(fd.get('s1295_capital_gain')) || 0;
        state.foreign_country = fd.get('foreign_country') || '';
        state.cusip = fd.get('cusip') || '';
        state.days_held = Number(fd.get('days_held')) || 0;
        state.s1297_e_test_active_banking = !!fd.get('s1297_e_test_active_banking');
        state.s1298_attribution_rules = !!fd.get('s1298_attribution_rules');
        state.s1298_a_5_indirect_ownership = !!fd.get('s1298_a_5_indirect_ownership');
        state.s1298_e_lookthrough = !!fd.get('s1298_e_lookthrough');
        state.s1297_f_active_insurance = !!fd.get('s1297_f_active_insurance');
        state.is_excepted_insurance = !!fd.get('is_excepted_insurance');
        state.s1291_pedigreed_pfic = !!fd.get('s1291_pedigreed_pfic');
        state.purging_distribution = Number(fd.get('purging_distribution')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1296-output');
    if (!el) return;
    const mtm_diff = state.fmv_year_end - state.adjusted_basis_year_end;
    const ord_gain = mtm_diff > 0 ? mtm_diff : 0;
    const allowed_loss = mtm_diff < 0 ? Math.min(Math.abs(mtm_diff), state.has_unreversed_inclusions) : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1296.h2.result">§ 1296 MTM result</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s1296.card.fmv">FMV year-end</div><div class="value">$${state.fmv_year_end.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s1296.card.basis">Adj basis</div><div class="value">$${state.adjusted_basis_year_end.toLocaleString()}</div></div>
                <div class="card ${ord_gain > 0 ? 'neg' : ''}"><div class="label" data-i18n="view.s1296.card.gain">Ordinary gain</div><div class="value">$${ord_gain.toLocaleString()}</div></div>
                <div class="card ${allowed_loss > 0 ? 'pos' : ''}"><div class="label" data-i18n="view.s1296.card.loss">Allowed loss (limited)</div><div class="value">$${allowed_loss.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s1296.card.unreversed">Unreversed inclusions</div><div class="value">$${state.has_unreversed_inclusions.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
