// APR ↔ APY — nominal vs effective annual rate at a compounding frequency,
// plus the continuous-compounding ceiling, via /calc/apr-apy. Updates live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const pct = (n) => Number(n).toFixed(4) + '%';

export async function renderAprApy(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.apy.h1.title">// APR ↔ APY</span></h1>
        <p class="muted small" data-i18n="view.apy.hint.intro">
            A rate quoted "12% compounded monthly" (APR) isn't what you actually earn or pay
            over a year — compounding pushes the effective rate (APY) higher, more so the
            oftener it compounds, up to a ceiling at continuous compounding. Convert either
            direction at any compounding frequency. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.apy.h2.inputs">The rate</h2>
            <form id="apy-form" class="inline-form">
                <label><span data-i18n="view.apy.label.direction">Convert</span>
                    <select name="direction">
                        <option value="apr_to_apy" data-i18n="view.apy.dir.apr_to_apy">APR → APY</option>
                        <option value="apy_to_apr" data-i18n="view.apy.dir.apy_to_apr">APY → APR</option>
                    </select></label>
                <label><span data-i18n="view.apy.label.rate">Rate (%)</span>
                    <input type="number" step="0.0001" min="0" name="rate_pct" value="12" required></label>
                <label><span data-i18n="view.apy.label.periods">Compounding periods / year</span>
                    <input type="number" step="1" min="1" name="periods_per_year" value="12" required></label>
            </form>
        </div>
        <div id="apy-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#apy-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            direction: fd.get('direction'),
            rate_pct: Number(fd.get('rate_pct')) || 0,
            periods_per_year: Number(fd.get('periods_per_year')) || 1,
        };
        try {
            const r = await api.calcAprApy(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.apy.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#apy-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.apy.h2.result">The rates</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.apy.card.apr">Nominal APR</div>
                    <div class="value">${pct(r.nominal_apr_pct)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.apy.card.apy">Effective APY</div>
                    <div class="value pos">${pct(r.effective_apy_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.apy.card.spread">Compounding boost</div>
                    <div class="value">${pct(r.spread_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.apy.card.continuous">Continuous-compounding APY</div>
                    <div class="value">${pct(r.continuous_apy_pct)}</div></div>
            </div>
        </div>
    `;
    applyUiI18n(el);
}
