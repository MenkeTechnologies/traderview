// IRC § 1231 — Property Used in a Trade or Business + Involuntary Conversions.
// Net § 1231 gain = LTCG character. Net § 1231 loss = ORDINARY loss character.
// Asymmetric "best of both worlds" treatment for depreciable real + personal business property.
// § 1231(c) 5-year look-back: prior net § 1231 losses recapture current gain as ordinary.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    current_year_s1231_gains: 0,
    current_year_s1231_losses: 0,
    net_s1231_gain: 0,
    net_s1231_loss: 0,
    prior_5_year_net_losses: [0, 0, 0, 0, 0],
    cumulative_5yr_losses: 0,
    look_back_recapture: 0,
    after_recapture_capital: 0,
    holding_period_months: 12,
    depreciable_business_property: 0,
    real_estate_used_business: 0,
    timber_iron_coal: 0,
    livestock_held_12mo: 0,
    livestock_cattle_24mo: 0,
    unharvested_crops: 0,
    involuntary_conversion_gain: 0,
    casualty_loss: 0,
    is_inventory: false,
    is_capital_asset: false,
    is_s1245_property: false,
    is_s1250_property: false,
    s1245_recapture_first: 0,
    s1250_recapture_first: 0,
    s1231_post_recapture: 0,
    is_partnership_distribution: false,
    s751_hot_asset_adjustment: 0,
    s732_e_basis_adjust: 0,
};

export async function renderSection1231(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1231.h1.title">// § 1231 BUSINESS PROPERTY (HOTCHPOT)</span></h1>
        <p class="muted small" data-i18n="view.s1231.hint.intro">
            <strong>§ 1231 "hotchpot"</strong> — net gains/losses from sale or involuntary conversion
            of depreciable business property + business real estate get ASYMMETRIC treatment.
            <strong>Net § 1231 gain</strong> → LONG-TERM CAPITAL GAIN. <strong>Net § 1231 loss</strong>
            → ORDINARY LOSS. <strong>§ 1231(c) 5-year look-back:</strong> if cumulative 5-year net
            § 1231 losses, recapture current year gain as ORDINARY up to cumulative loss.
            <strong>§ 1245 / § 1250 recapture applies FIRST</strong> — § 1231 treatment only for
            remaining gain. <strong>§ 1231 property:</strong> depreciable business + real estate (&gt; 1
            year), timber/iron/coal, livestock (12 mo for most / 24 mo cattle + horses), unharvested
            crops on land sold concurrently. <strong>NOT § 1231:</strong> inventory, capital assets,
            government publications, copyright (artist or first owner).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1231.h2.inputs">Inputs</h2>
            <form id="s1231-form" class="inline-form">
                <label><span data-i18n="view.s1231.label.gains">Current year § 1231 gains ($)</span>
                    <input type="number" step="1000" name="current_year_s1231_gains" value="${state.current_year_s1231_gains}"></label>
                <label><span data-i18n="view.s1231.label.losses">Current year § 1231 losses ($)</span>
                    <input type="number" step="1000" name="current_year_s1231_losses" value="${state.current_year_s1231_losses}"></label>
                <label><span data-i18n="view.s1231.label.net_gain">Net § 1231 gain ($)</span>
                    <input type="number" step="1000" name="net_s1231_gain" value="${state.net_s1231_gain}"></label>
                <label><span data-i18n="view.s1231.label.net_loss">Net § 1231 loss ($)</span>
                    <input type="number" step="1000" name="net_s1231_loss" value="${state.net_s1231_loss}"></label>
                <label><span data-i18n="view.s1231.label.cum_5yr">Cumulative 5-yr losses ($)</span>
                    <input type="number" step="1000" name="cumulative_5yr_losses" value="${state.cumulative_5yr_losses}"></label>
                <label><span data-i18n="view.s1231.label.recapture">Look-back recapture ($)</span>
                    <input type="number" step="1000" name="look_back_recapture" value="${state.look_back_recapture}"></label>
                <label><span data-i18n="view.s1231.label.after_rec">After-recapture capital ($)</span>
                    <input type="number" step="1000" name="after_recapture_capital" value="${state.after_recapture_capital}"></label>
                <label><span data-i18n="view.s1231.label.holding">Holding period (months)</span>
                    <input type="number" step="1" name="holding_period_months" value="${state.holding_period_months}"></label>
                <label><span data-i18n="view.s1231.label.dep_biz">Depreciable business ($)</span>
                    <input type="number" step="1000" name="depreciable_business_property" value="${state.depreciable_business_property}"></label>
                <label><span data-i18n="view.s1231.label.real_estate">Real estate (biz use) ($)</span>
                    <input type="number" step="10000" name="real_estate_used_business" value="${state.real_estate_used_business}"></label>
                <label><span data-i18n="view.s1231.label.timber">Timber / iron / coal ($)</span>
                    <input type="number" step="1000" name="timber_iron_coal" value="${state.timber_iron_coal}"></label>
                <label><span data-i18n="view.s1231.label.livestock_12">Livestock 12-mo ($)</span>
                    <input type="number" step="1000" name="livestock_held_12mo" value="${state.livestock_held_12mo}"></label>
                <label><span data-i18n="view.s1231.label.livestock_24">Cattle/horses 24-mo ($)</span>
                    <input type="number" step="1000" name="livestock_cattle_24mo" value="${state.livestock_cattle_24mo}"></label>
                <label><span data-i18n="view.s1231.label.crops">Unharvested crops ($)</span>
                    <input type="number" step="1000" name="unharvested_crops" value="${state.unharvested_crops}"></label>
                <label><span data-i18n="view.s1231.label.inv_conv">Involuntary conv gain ($)</span>
                    <input type="number" step="1000" name="involuntary_conversion_gain" value="${state.involuntary_conversion_gain}"></label>
                <label><span data-i18n="view.s1231.label.casualty">Casualty loss ($)</span>
                    <input type="number" step="1000" name="casualty_loss" value="${state.casualty_loss}"></label>
                <label><span data-i18n="view.s1231.label.inventory">Inventory?</span>
                    <input type="checkbox" name="is_inventory" ${state.is_inventory ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1231.label.capital">Capital asset?</span>
                    <input type="checkbox" name="is_capital_asset" ${state.is_capital_asset ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1231.label.s1245">§ 1245 property?</span>
                    <input type="checkbox" name="is_s1245_property" ${state.is_s1245_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1231.label.s1250">§ 1250 property?</span>
                    <input type="checkbox" name="is_s1250_property" ${state.is_s1250_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1231.label.s1245_rec">§ 1245 recapture first ($)</span>
                    <input type="number" step="1000" name="s1245_recapture_first" value="${state.s1245_recapture_first}"></label>
                <label><span data-i18n="view.s1231.label.s1250_rec">§ 1250 recapture first ($)</span>
                    <input type="number" step="1000" name="s1250_recapture_first" value="${state.s1250_recapture_first}"></label>
                <label><span data-i18n="view.s1231.label.post_rec">§ 1231 post-recapture ($)</span>
                    <input type="number" step="1000" name="s1231_post_recapture" value="${state.s1231_post_recapture}"></label>
                <label><span data-i18n="view.s1231.label.partnership">Partnership distribution?</span>
                    <input type="checkbox" name="is_partnership_distribution" ${state.is_partnership_distribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1231.label.s751">§ 751 hot adj ($)</span>
                    <input type="number" step="1000" name="s751_hot_asset_adjustment" value="${state.s751_hot_asset_adjustment}"></label>
                <label><span data-i18n="view.s1231.label.s732_e">§ 732(e) basis adj ($)</span>
                    <input type="number" step="1000" name="s732_e_basis_adjust" value="${state.s732_e_basis_adjust}"></label>
                <button class="primary" type="submit" data-i18n="view.s1231.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1231-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1231.h2.property">§ 1231 property definition</h2>
            <ol class="muted small">
                <li data-i18n="view.s1231.prop.depreciable">Depreciable property used in trade or business — held &gt; 1 year</li>
                <li data-i18n="view.s1231.prop.real_estate">Real property used in trade or business — held &gt; 1 year</li>
                <li data-i18n="view.s1231.prop.timber">Timber (§ 631(a)/(b)), iron ore, coal (§ 631(c))</li>
                <li data-i18n="view.s1231.prop.livestock_12">Livestock (12+ months) — NOT poultry</li>
                <li data-i18n="view.s1231.prop.livestock_24">Cattle + horses: 24+ months</li>
                <li data-i18n="view.s1231.prop.crops">Unharvested crops on land sold simultaneously</li>
                <li data-i18n="view.s1231.prop.involuntary">Involuntary conversion gains on § 1231 property</li>
                <li data-i18n="view.s1231.prop.NOT_inventory">NOT inventory (§ 1221(a)(1))</li>
                <li data-i18n="view.s1231.prop.NOT_capital">NOT capital assets (separate § 1221 treatment)</li>
                <li data-i18n="view.s1231.prop.NOT_government">NOT government publications, copyright by author</li>
                <li data-i18n="view.s1231.prop.NOT_supplies">NOT supplies in business use</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1231.h2.hotchpot">"Hotchpot" netting (§ 1231(a))</h2>
            <ol class="muted small">
                <li data-i18n="view.s1231.hp.step1">Step 1: Net all § 1231 gains and losses for the year</li>
                <li data-i18n="view.s1231.hp.step2">Step 2: If net is GAIN → LTCG (subject to look-back recapture)</li>
                <li data-i18n="view.s1231.hp.step3">Step 3: If net is LOSS → ordinary loss (Schedule D, line 1)</li>
                <li data-i18n="view.s1231.hp.firepot_first">"Firepot" sub-netting: net casualty losses + involuntary conversions</li>
                <li data-i18n="view.s1231.hp.firepot_loss">If firepot is NET LOSS → goes to ordinary (Schedule A or Schedule C)</li>
                <li data-i18n="view.s1231.hp.firepot_gain">If firepot is NET GAIN → enters main hotchpot for LTCG/loss netting</li>
                <li data-i18n="view.s1231.hp.s1245_first">§ 1245 / § 1250 recapture applied FIRST — only excess enters hotchpot</li>
                <li data-i18n="view.s1231.hp.character">Character determined ENTITY-LEVEL at partnership / S-corp</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1231.h2.lookback">5-year look-back recapture (§ 1231(c))</h2>
            <ul class="muted small">
                <li data-i18n="view.s1231.lb.purpose">Anti-abuse: prevents alternating loss + gain years to game character</li>
                <li data-i18n="view.s1231.lb.5yr">Looks back 5 most recent tax years</li>
                <li data-i18n="view.s1231.lb.cumulative">Cumulative net § 1231 losses in those 5 years</li>
                <li data-i18n="view.s1231.lb.recapture">Current year § 1231 GAIN recaptured as ORDINARY up to cumulative losses</li>
                <li data-i18n="view.s1231.lb.excess_LTCG">Excess over cumulative losses retains LTCG treatment</li>
                <li data-i18n="view.s1231.lb.example">Example: prior 5 yrs $50K cumulative loss + current $80K gain → $50K ordinary + $30K LTCG</li>
                <li data-i18n="view.s1231.lb.no_carryover">Recaptured losses do NOT carryforward — used up in current year</li>
                <li data-i18n="view.s1231.lb.tracked">Taxpayer must track 5-year history per § 1.1231-1(g)</li>
                <li data-i18n="view.s1231.lb.individual_partnership">Tested at INDIVIDUAL partner level (not partnership)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1231.h2.coordination">Coordination with § 1245 / § 1250</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s1231.tbl.step">Step</th><th data-i18n="view.s1231.tbl.action">Action</th><th data-i18n="view.s1231.tbl.citation">Citation</th></tr></thead>
                <tbody>
                    <tr><td>1</td><td data-i18n="view.s1231.tbl.s1245">Apply § 1245 (personal property) / § 1250 (real property) recapture</td><td>§ 1245(a)(1), § 1250(a)(1)</td></tr>
                    <tr><td>2</td><td data-i18n="view.s1231.tbl.ordinary">Recapture amount = ORDINARY income</td><td>§ 1245(a)(1)</td></tr>
                    <tr><td>3</td><td data-i18n="view.s1231.tbl.remaining">REMAINING gain enters § 1231 hotchpot</td><td>§ 1231(a)(4)(C)</td></tr>
                    <tr><td>4</td><td data-i18n="view.s1231.tbl.net_step">Net all § 1231 gains/losses</td><td>§ 1231(a)(1)</td></tr>
                    <tr><td>5</td><td data-i18n="view.s1231.tbl.apply_lookback">Apply 5-year look-back recapture</td><td>§ 1231(c)</td></tr>
                    <tr><td>6</td><td data-i18n="view.s1231.tbl.remaining_LTCG">Remaining net gain = LTCG (or net loss = ordinary)</td><td>§ 1231(a)(2)</td></tr>
                    <tr><td>7</td><td data-i18n="view.s1231.tbl.unrec_s1250">Unrecaptured § 1250 gain split out for 25% rate</td><td>§ 1(h)(6)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1231.h2.special">Special rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s1231.spec.casualty">Casualty / theft losses on personal-use property: separate treatment</li>
                <li data-i18n="view.s1231.spec.partnership">Partnership: § 1231 character flows through to partners</li>
                <li data-i18n="view.s1231.spec.s_corp">S-corp: same — flow-through</li>
                <li data-i18n="view.s1231.spec.s_corp_BIG">S-corp with C-corp history: § 1374 built-in gain may apply</li>
                <li data-i18n="view.s1231.spec.s1239">§ 1239 transfer to controlled entity: ordinary recapture</li>
                <li data-i18n="view.s1231.spec.s1245_complete">§ 1245 recapture is COMPLETE — no § 1231 if full recapture</li>
                <li data-i18n="view.s1231.spec.s1252">§ 1252 farm soil/water conservation expenditures</li>
                <li data-i18n="view.s1231.spec.s1253">§ 1253 franchise transfers</li>
                <li data-i18n="view.s1231.spec.s1254">§ 1254 IDC + oil/gas property</li>
                <li data-i18n="view.s1231.spec.qoz">§ 1400Z-2 QOZ — defers § 1231 gain into QOF</li>
                <li data-i18n="view.s1231.spec.f4797">Form 4797 reports + Section A (S/T) / B (L/T)</li>
            </ul>
        </div>
    `;
    document.getElementById('s1231-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.current_year_s1231_gains = Number(fd.get('current_year_s1231_gains')) || 0;
        state.current_year_s1231_losses = Number(fd.get('current_year_s1231_losses')) || 0;
        state.net_s1231_gain = Number(fd.get('net_s1231_gain')) || 0;
        state.net_s1231_loss = Number(fd.get('net_s1231_loss')) || 0;
        state.cumulative_5yr_losses = Number(fd.get('cumulative_5yr_losses')) || 0;
        state.look_back_recapture = Number(fd.get('look_back_recapture')) || 0;
        state.after_recapture_capital = Number(fd.get('after_recapture_capital')) || 0;
        state.holding_period_months = Number(fd.get('holding_period_months')) || 0;
        state.depreciable_business_property = Number(fd.get('depreciable_business_property')) || 0;
        state.real_estate_used_business = Number(fd.get('real_estate_used_business')) || 0;
        state.timber_iron_coal = Number(fd.get('timber_iron_coal')) || 0;
        state.livestock_held_12mo = Number(fd.get('livestock_held_12mo')) || 0;
        state.livestock_cattle_24mo = Number(fd.get('livestock_cattle_24mo')) || 0;
        state.unharvested_crops = Number(fd.get('unharvested_crops')) || 0;
        state.involuntary_conversion_gain = Number(fd.get('involuntary_conversion_gain')) || 0;
        state.casualty_loss = Number(fd.get('casualty_loss')) || 0;
        state.is_inventory = !!fd.get('is_inventory');
        state.is_capital_asset = !!fd.get('is_capital_asset');
        state.is_s1245_property = !!fd.get('is_s1245_property');
        state.is_s1250_property = !!fd.get('is_s1250_property');
        state.s1245_recapture_first = Number(fd.get('s1245_recapture_first')) || 0;
        state.s1250_recapture_first = Number(fd.get('s1250_recapture_first')) || 0;
        state.s1231_post_recapture = Number(fd.get('s1231_post_recapture')) || 0;
        state.is_partnership_distribution = !!fd.get('is_partnership_distribution');
        state.s751_hot_asset_adjustment = Number(fd.get('s751_hot_asset_adjustment')) || 0;
        state.s732_e_basis_adjust = Number(fd.get('s732_e_basis_adjust')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1231-output');
    if (!el) return;
    const net = state.current_year_s1231_gains - state.current_year_s1231_losses;
    const recapture = net > 0 ? Math.min(net, state.cumulative_5yr_losses) : 0;
    const ltcg_portion = Math.max(0, net - recapture);
    const ordinary_loss = net < 0 ? Math.abs(net) : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1231.h2.result">§ 1231 hotchpot result</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s1231.card.net">Net § 1231</div><div class="value">$${net.toLocaleString()}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s1231.card.recapture">Look-back recapture (ord)</div><div class="value">$${recapture.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s1231.card.ltcg">LTCG portion</div><div class="value">$${ltcg_portion.toLocaleString()}</div></div>
                <div class="card ${ordinary_loss > 0 ? 'pos' : ''}"><div class="label" data-i18n="view.s1231.card.ord_loss">Ordinary loss</div><div class="value">$${ordinary_loss.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
