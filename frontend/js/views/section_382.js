// IRC § 382 — Limitation on NOL Carryforward after Ownership Change.
// Annual limit on NOL utilization = Value of loss corp × Long-term tax-exempt rate.
// "Ownership change" = > 50 percentage point increase by 5%+ shareholders over 3-year testing period.
// § 383 — applies analogous limit to credits + capital loss carryforwards.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    pre_change_nols: 0,
    capital_loss_carryforward: 0,
    fmv_loss_corporation: 0,
    long_term_tax_exempt_rate: 3.6,
    annual_s382_limitation: 0,
    rbig_5_year: 0,
    rbil_5_year: 0,
    s338_election: false,
    s338_election_basis: 0,
    s382_h_built_in_gain_recognition: 0,
    s382_h_built_in_loss_recognition: 0,
    ownership_change_date: '',
    testing_period_start: '',
    ownership_change_pct: 0,
    is_ownership_change: false,
    s382_g_owner_shifts: 0,
    s382_l_3_segregation: false,
    s382_l_5_pre_change_lookback: false,
    s382_l_6_post_change_pooling: false,
    s382_e_2_year_continuity: false,
    s382_h_continuity_business: false,
    is_qualified_purchase: false,
    s382_g_5_pct_threshold: false,
    is_5_percent_shareholder: false,
    multiple_5_pct_shareholders: 0,
    s383_credits: 0,
    s269_anti_abuse: false,
    s384_purchase_amount: 0,
    s269_a_purpose: false,
    serial_owner_change: false,
};

export async function renderSection382(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s382.h1.title">// § 382 NOL LIMITATION (OWNERSHIP CHANGE)</span></h1>
        <p class="muted small" data-i18n="view.s382.hint.intro">
            <strong>§ 382 limits NOL utilization</strong> after "ownership change" to ANNUAL limit
            = FMV of loss corporation × long-term tax-exempt rate (~3.6% as of 2024).
            <strong>Ownership change:</strong> &gt; 50 percentage point increase by 5%+ shareholders
            over <strong>3-year testing period</strong>. <strong>§ 382(h) RBIG/RBIL:</strong>
            recognized built-in gain (RBIG) — 5-year window adds to annual limit; recognized built-in
            loss (RBIL) — 5-year — does NOT escape § 382 limit. <strong>§ 382(c)(1) continuity of
            business enterprise (COBE):</strong> if business discontinued within 2 years post-change,
            NOLs ELIMINATED entirely. <strong>§ 383</strong> applies analogous limit to credit
            carryforwards + capital losses. <strong>§ 269</strong> anti-abuse + <strong>§ 384</strong>
            related rules for asset acquisitions. <strong>§ 338 elections</strong> can reset baseline.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s382.h2.inputs">Inputs</h2>
            <form id="s382-form" class="inline-form">
                <label><span data-i18n="view.s382.label.nols">Pre-change NOLs ($)</span>
                    <input type="number" step="100000" name="pre_change_nols" value="${state.pre_change_nols}"></label>
                <label><span data-i18n="view.s382.label.cap_loss">Cap loss carryforward ($)</span>
                    <input type="number" step="100000" name="capital_loss_carryforward" value="${state.capital_loss_carryforward}"></label>
                <label><span data-i18n="view.s382.label.fmv">FMV of loss corp ($)</span>
                    <input type="number" step="100000" name="fmv_loss_corporation" value="${state.fmv_loss_corporation}"></label>
                <label><span data-i18n="view.s382.label.lttx">LT tax-exempt rate %</span>
                    <input type="number" step="0.01" name="long_term_tax_exempt_rate" value="${state.long_term_tax_exempt_rate}"></label>
                <label><span data-i18n="view.s382.label.annual">§ 382 annual limit ($)</span>
                    <input type="number" step="10000" name="annual_s382_limitation" value="${state.annual_s382_limitation}"></label>
                <label><span data-i18n="view.s382.label.rbig">RBIG 5-yr ($)</span>
                    <input type="number" step="100000" name="rbig_5_year" value="${state.rbig_5_year}"></label>
                <label><span data-i18n="view.s382.label.rbil">RBIL 5-yr ($)</span>
                    <input type="number" step="100000" name="rbil_5_year" value="${state.rbil_5_year}"></label>
                <label><span data-i18n="view.s382.label.s338">§ 338 election?</span>
                    <input type="checkbox" name="s338_election" ${state.s338_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s382.label.s338_basis">§ 338 election basis ($)</span>
                    <input type="number" step="100000" name="s338_election_basis" value="${state.s338_election_basis}"></label>
                <label><span data-i18n="view.s382.label.bg_rec">§ 382(h) BIG recognized ($)</span>
                    <input type="number" step="100000" name="s382_h_built_in_gain_recognition" value="${state.s382_h_built_in_gain_recognition}"></label>
                <label><span data-i18n="view.s382.label.bl_rec">§ 382(h) BIL recognized ($)</span>
                    <input type="number" step="100000" name="s382_h_built_in_loss_recognition" value="${state.s382_h_built_in_loss_recognition}"></label>
                <label><span data-i18n="view.s382.label.change_date">Ownership change date</span>
                    <input type="date" name="ownership_change_date" value="${state.ownership_change_date}"></label>
                <label><span data-i18n="view.s382.label.test_start">Testing period start</span>
                    <input type="date" name="testing_period_start" value="${state.testing_period_start}"></label>
                <label><span data-i18n="view.s382.label.pct">Ownership change %</span>
                    <input type="number" step="0.1" name="ownership_change_pct" value="${state.ownership_change_pct}"></label>
                <label><span data-i18n="view.s382.label.is_change">Ownership change?</span>
                    <input type="checkbox" name="is_ownership_change" ${state.is_ownership_change ? 'checked' : ''}></label>
                <label><span data-i18n="view.s382.label.shifts">§ 382(g) owner shifts</span>
                    <input type="number" step="1" name="s382_g_owner_shifts" value="${state.s382_g_owner_shifts}"></label>
                <label><span data-i18n="view.s382.label.segregation">§ 382(l)(3) segregation?</span>
                    <input type="checkbox" name="s382_l_3_segregation" ${state.s382_l_3_segregation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s382.label.lookback">§ 382(l)(5) pre-change lookback?</span>
                    <input type="checkbox" name="s382_l_5_pre_change_lookback" ${state.s382_l_5_pre_change_lookback ? 'checked' : ''}></label>
                <label><span data-i18n="view.s382.label.pooling">§ 382(l)(6) post-change pooling?</span>
                    <input type="checkbox" name="s382_l_6_post_change_pooling" ${state.s382_l_6_post_change_pooling ? 'checked' : ''}></label>
                <label><span data-i18n="view.s382.label.cobe">§ 382(c)(1) COBE 2-yr?</span>
                    <input type="checkbox" name="s382_e_2_year_continuity" ${state.s382_e_2_year_continuity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s382.label.continuity">Continuity of business?</span>
                    <input type="checkbox" name="s382_h_continuity_business" ${state.s382_h_continuity_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s382.label.qualified">Qualified purchase?</span>
                    <input type="checkbox" name="is_qualified_purchase" ${state.is_qualified_purchase ? 'checked' : ''}></label>
                <label><span data-i18n="view.s382.label.s382g5">5% threshold met?</span>
                    <input type="checkbox" name="s382_g_5_pct_threshold" ${state.s382_g_5_pct_threshold ? 'checked' : ''}></label>
                <label><span data-i18n="view.s382.label.is_5pct">Is 5%+ shareholder?</span>
                    <input type="checkbox" name="is_5_percent_shareholder" ${state.is_5_percent_shareholder ? 'checked' : ''}></label>
                <label><span data-i18n="view.s382.label.mult5">Multiple 5%+ count</span>
                    <input type="number" step="1" name="multiple_5_pct_shareholders" value="${state.multiple_5_pct_shareholders}"></label>
                <label><span data-i18n="view.s382.label.s383">§ 383 credits ($)</span>
                    <input type="number" step="10000" name="s383_credits" value="${state.s383_credits}"></label>
                <label><span data-i18n="view.s382.label.s269">§ 269 anti-abuse?</span>
                    <input type="checkbox" name="s269_anti_abuse" ${state.s269_anti_abuse ? 'checked' : ''}></label>
                <label><span data-i18n="view.s382.label.s384">§ 384 purchase ($)</span>
                    <input type="number" step="100000" name="s384_purchase_amount" value="${state.s384_purchase_amount}"></label>
                <label><span data-i18n="view.s382.label.s269a">§ 269(a) purpose?</span>
                    <input type="checkbox" name="s269_a_purpose" ${state.s269_a_purpose ? 'checked' : ''}></label>
                <label><span data-i18n="view.s382.label.serial">Serial owner change?</span>
                    <input type="checkbox" name="serial_owner_change" ${state.serial_owner_change ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s382.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s382-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s382.h2.ownership_change">Ownership change test (§ 382(g))</h2>
            <ol class="muted small">
                <li data-i18n="view.s382.oc.threshold">"Ownership change" = &gt; 50 PERCENTAGE POINT increase in stock owned by 5%+ shareholders</li>
                <li data-i18n="view.s382.oc.testing">Testing period: 3 years ending on potential change date</li>
                <li data-i18n="view.s382.oc.shareholder">"5% shareholder" includes both direct (named) + indirect (via aggregation)</li>
                <li data-i18n="view.s382.oc.public_group">"Public group" = aggregation of all &lt; 5% shareholders (treated as single 5% shareholder)</li>
                <li data-i18n="view.s382.oc.equity_structure">Test based on lower of: voting power OR value</li>
                <li data-i18n="view.s382.oc.entity_attribution">§ 382(l)(3)(A) entity attribution: family + entities aggregated</li>
                <li data-i18n="view.s382.oc.options">Stock options + convertible debt: treated as ownership if "in the money"</li>
                <li data-i18n="view.s382.oc.equity_split">Equity restructuring + recapitalization may trigger ownership change</li>
                <li data-i18n="view.s382.oc.s382_g_4">§ 382(g)(4)(D) — small issuance exception (limited dilution)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s382.h2.limitation">Annual limitation formula</h2>
            <ul class="muted small">
                <li data-i18n="view.s382.lim.formula">Annual limit = FMV of loss corp (immediately before change) × LT tax-exempt rate</li>
                <li data-i18n="view.s382.lim.rate">LT tax-exempt rate: highest of 3 prior months (published monthly by IRS)</li>
                <li data-i18n="view.s382.lim.rate_current">2024 LT tax-exempt rate: ~3.5%-3.7% (varies monthly)</li>
                <li data-i18n="view.s382.lim.example">$100M FMV × 3.6% = $3.6M annual NOL utilization</li>
                <li data-i18n="view.s382.lim.carryforward">Unused limit: carries forward to next year (cumulative)</li>
                <li data-i18n="view.s382.lim.s382_b_2">§ 382(b)(2): in change year, prorate based on days in year</li>
                <li data-i18n="view.s382.lim.s382_e_anti_stuffing">§ 382(e) anti-stuffing: redemptions/contributions in 2-year period before change reduce FMV</li>
                <li data-i18n="view.s382.lim.zero_fmv">If FMV at change is ~$0 (bankruptcy etc.): annual limit = $0, NOLs functionally lost</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s382.h2.rbig">§ 382(h) RBIG/RBIL</h2>
            <ul class="muted small">
                <li data-i18n="view.s382.rb.purpose">5-year window: identifies built-in gains/losses at time of change</li>
                <li data-i18n="view.s382.rb.rbig">RBIG: recognized built-in gain — ADDS to annual limit when recognized</li>
                <li data-i18n="view.s382.rb.rbil">RBIL: recognized built-in loss — subject to § 382 limit (does NOT escape)</li>
                <li data-i18n="view.s382.rb.threshold">NUBIG/NUBIL threshold: lesser of $10M OR 15% of value of asset at change</li>
                <li data-i18n="view.s382.rb.below_threshold">If below threshold: no RBIG/RBIL adjustments</li>
                <li data-i18n="view.s382.rb.5_year">5-year recognition period from change date</li>
                <li data-i18n="view.s382.rb.s338_approach">§ 338 approach: hypothetical sale at change FMV identifies all gains/losses</li>
                <li data-i18n="view.s382.rb.s1374_approach">§ 1374 approach: track items individually recognized in 5-year window</li>
                <li data-i18n="view.s382.rb.notice_2003_65">Notice 2003-65: optional safe harbor</li>
                <li data-i18n="view.s382.rb.s382_h_8">§ 382(h)(8) — deferred CODI included if subject to attribute reduction</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s382.h2.cobe">§ 382(c)(1) Continuity of Business Enterprise</h2>
            <ul class="muted small">
                <li data-i18n="view.s382.cobe.purpose">Anti-trafficking: prevents acquiring loss corp + immediately shutting down</li>
                <li data-i18n="view.s382.cobe.period">2-year period post-change</li>
                <li data-i18n="view.s382.cobe.test">Must EITHER continue historic business OR use significant historic assets</li>
                <li data-i18n="view.s382.cobe.failure">If FAIL: pre-change NOLs ELIMINATED entirely (not just limited)</li>
                <li data-i18n="view.s382.cobe.s368">Borrowed from § 368 reorganization continuity doctrine</li>
                <li data-i18n="view.s382.cobe.subsidiary">Conducted in subsidiary OK (per § 1.382-3(d))</li>
                <li data-i18n="view.s382.cobe.partial_continuation">Partial continuation: facts + circumstances</li>
                <li data-i18n="view.s382.cobe.industries">Different industries: factor in similarity</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s382.h2.relief_provisions">Relief provisions</h2>
            <ul class="muted small">
                <li data-i18n="view.s382.relief.s382_l_5">§ 382(l)(5) bankruptcy: title 11 reorg — exempt if old shareholders retain 50%+</li>
                <li data-i18n="view.s382.relief.s382_l_6">§ 382(l)(6) bankruptcy alternative: increased FMV based on post-reorg value</li>
                <li data-i18n="view.s382.relief.s269_b">§ 269(b)(2) — relief from § 269 if no tax avoidance purpose</li>
                <li data-i18n="view.s382.relief.s338_election">§ 338 / § 336(e) election: step-up basis avoids inheriting limitation in some cases</li>
                <li data-i18n="view.s382.relief.f_reorganization">F-reorganization: NO ownership change (single corp continuation)</li>
                <li data-i18n="view.s382.relief.b_reorganization">B-reorganization (stock-for-stock): may trigger ownership change</li>
                <li data-i18n="view.s382.relief.qstk">QSST + ESBT for S-corp ownership testing</li>
                <li data-i18n="view.s382.relief.s382_g_4">§ 382(g)(4) small issuance + cash issuance exceptions</li>
                <li data-i18n="view.s382.relief.poison_pill">Poison pill / shareholder rights plan — defends against unintentional change</li>
            </ul>
        </div>
    `;
    document.getElementById('s382-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.pre_change_nols = Number(fd.get('pre_change_nols')) || 0;
        state.capital_loss_carryforward = Number(fd.get('capital_loss_carryforward')) || 0;
        state.fmv_loss_corporation = Number(fd.get('fmv_loss_corporation')) || 0;
        state.long_term_tax_exempt_rate = Number(fd.get('long_term_tax_exempt_rate')) || 0;
        state.annual_s382_limitation = Number(fd.get('annual_s382_limitation')) || 0;
        state.rbig_5_year = Number(fd.get('rbig_5_year')) || 0;
        state.rbil_5_year = Number(fd.get('rbil_5_year')) || 0;
        state.s338_election = !!fd.get('s338_election');
        state.s338_election_basis = Number(fd.get('s338_election_basis')) || 0;
        state.s382_h_built_in_gain_recognition = Number(fd.get('s382_h_built_in_gain_recognition')) || 0;
        state.s382_h_built_in_loss_recognition = Number(fd.get('s382_h_built_in_loss_recognition')) || 0;
        state.ownership_change_date = fd.get('ownership_change_date') || '';
        state.testing_period_start = fd.get('testing_period_start') || '';
        state.ownership_change_pct = Number(fd.get('ownership_change_pct')) || 0;
        state.is_ownership_change = !!fd.get('is_ownership_change');
        state.s382_g_owner_shifts = Number(fd.get('s382_g_owner_shifts')) || 0;
        state.s382_l_3_segregation = !!fd.get('s382_l_3_segregation');
        state.s382_l_5_pre_change_lookback = !!fd.get('s382_l_5_pre_change_lookback');
        state.s382_l_6_post_change_pooling = !!fd.get('s382_l_6_post_change_pooling');
        state.s382_e_2_year_continuity = !!fd.get('s382_e_2_year_continuity');
        state.s382_h_continuity_business = !!fd.get('s382_h_continuity_business');
        state.is_qualified_purchase = !!fd.get('is_qualified_purchase');
        state.s382_g_5_pct_threshold = !!fd.get('s382_g_5_pct_threshold');
        state.is_5_percent_shareholder = !!fd.get('is_5_percent_shareholder');
        state.multiple_5_pct_shareholders = Number(fd.get('multiple_5_pct_shareholders')) || 0;
        state.s383_credits = Number(fd.get('s383_credits')) || 0;
        state.s269_anti_abuse = !!fd.get('s269_anti_abuse');
        state.s384_purchase_amount = Number(fd.get('s384_purchase_amount')) || 0;
        state.s269_a_purpose = !!fd.get('s269_a_purpose');
        state.serial_owner_change = !!fd.get('serial_owner_change');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s382-output');
    if (!el) return;
    const annual_limit = state.fmv_loss_corporation * (state.long_term_tax_exempt_rate / 100);
    const limit_with_rbig = annual_limit + state.s382_h_built_in_gain_recognition;
    const years_to_use = state.pre_change_nols > 0 && annual_limit > 0 ? state.pre_change_nols / annual_limit : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s382.h2.result">§ 382 NOL utilization</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s382.card.nols">Pre-change NOLs</div><div class="value">$${state.pre_change_nols.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s382.card.annual">Annual § 382 limit</div><div class="value">$${annual_limit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s382.card.rbig">With RBIG add</div><div class="value">$${limit_with_rbig.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card warn"><div class="label" data-i18n="view.s382.card.years">Years to use</div><div class="value">${years_to_use.toFixed(1)}</div></div>
                <div class="card ${state.is_ownership_change ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s382.card.change">Ownership change?</div><div class="value">${state.is_ownership_change ? 'YES (limit active)' : 'NO'}</div></div>
            </div>
        </div>
    `;
}
