// Market Profile (TPO) view — Time-Price Opportunity histogram.
//
// Sierra Chart-class. Each bracket (typically 30-min) contributes one
// letter per price level it visited. Stacking those letters reveals:
//   - POC (Point of Control) — price with the most time
//   - Value Area (VAH/VAL) — 70% of TPOs around POC
//   - Single prints — price levels touched in only one bracket (excess)

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBracketBlob, validateInputs, buildBody,
    levelTier, levelLetters, tierCounts,
    makeDemoBrackets, fmtN, fmtInt,
} from '../_market_profile_inputs.js';

import { t } from '../i18n.js';
let state = { brackets: '', tickSize: 0.5 };

export async function renderMarketProfile(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.market_profile.h1.market_profile_tpo_histogram" class="view-title">// MARKET PROFILE · TPO HISTOGRAM</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.market_profile.h2.brackets_one_per_time_period_typically_30_min_rth_">Brackets (one per time period — typically 30-min RTH brackets)</h2>
            <p class="muted" data-i18n="view.market_profile.hint.format">One line per bracket: bracket_index high low. Each bracket prints one letter (A, B, C, …) at every quantized price level it traded through. Demo loads a 13-bracket A-M session shaped like a typical normal-day profile.</p>
            <textarea id="mp-brackets" rows="8" placeholder="0 102.5 101.0&#10;1 101.5 100.0&#10;..."></textarea>
            <div class="inline-form">
                <label><span data-i18n="view.market_profile.label.tick_size">Tick size</span>
                    <input id="mp-tick" type="number" step="any" min="0" value="${state.tickSize}"></label>
                <button data-i18n="view.market_profile.btn.load_demo_13_bracket_normal_day" id="mp-demo" class="secondary" type="button">Load demo (13-bracket normal day)</button>
                <button data-i18n="view.market_profile.btn.clear" id="mp-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.market_profile.btn.build_tpo" id="mp-run" class="primary" type="button">Build TPO</button>
            </div>
        </div>

        <div id="mp-errors" class="boot" style="display:none"></div>
        <div id="mp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.market_profile.h2.tpo_histogram">TPO histogram</h2>
            <div id="mp-tpo" class="mp-tpo"></div>
            <p data-i18n="view.market_profile.hint.yellow_row_poc_most_time_cyan_rows_value_area_70_r" class="muted">Yellow row = POC (most time). Cyan rows = Value Area (70%).
                Red rows = single prints (excess — often retested). Letters are the
                actual TPO brackets that touched each price level.</p>
        </div>

        <div id="mp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('mp-demo').addEventListener('click', () => {
        const b = makeDemoBrackets();
        document.getElementById('mp-brackets').value =
            b.map(x => `${x.bracket_index} ${x.high} ${x.low}`).join('\n');
    });
    document.getElementById('mp-clear').addEventListener('click', () => {
        document.getElementById('mp-brackets').value = '';
    });
    document.getElementById('mp-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.brackets = document.getElementById('mp-brackets').value;
    state.tickSize = Number(document.getElementById('mp-tick').value);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('mp-errors');
    errs.style.display = 'none';
    const { brackets, errors } = parseBracketBlob(state.brackets);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (brackets.length === 0) return;
    }
    const err = validateInputs(brackets, state.tickSize);
    if (err) { showErr(err); return; }
    let report;
    try {
        report = await api.microMarketProfile(buildBody(brackets, state.tickSize));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report);
    renderTpo(report);
}

function renderSummary(r) {
    const counts = tierCounts(r);
    const vaWidth = r.value_area_high - r.value_area_low;
    document.getElementById('mp-summary').innerHTML = [
        card(t('view.market_profile.card.poc'),           fmtN(r.poc_price), 'pos'),
        card(t('view.market_profile.card.vah'),           fmtN(r.value_area_high)),
        card(t('view.market_profile.card.val'),           fmtN(r.value_area_low)),
        card(t('view.market_profile.card.va_width'),      fmtN(vaWidth)),
        card(t('view.market_profile.card.total_tpos'),    fmtInt(r.total_tpos)),
        card(t('view.market_profile.card.price_levels'),  String((r.levels || []).length)),
        card(t('view.market_profile.card.single_prints'), String(counts.single), counts.single > 0 ? 'neg' : ''),
        card(t('view.market_profile.card.tick_size'),     fmtN(r.tick_size, 4)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderTpo(report) {
    const wrap = document.getElementById('mp-tpo');
    if (!report || !Array.isArray(report.levels) || !report.levels.length) {
        wrap.innerHTML = '<div class="muted">No levels.</div>';
        return;
    }
    // Order top-down: highest price first (standard Sierra Chart layout).
    const sorted = [...report.levels].sort((a, b) => b.price - a.price);
    const maxCount = Math.max(...sorted.map(l => l.tpo_count), 1);
    wrap.innerHTML = sorted.map(l => {
        const tier = levelTier(l, report);
        const widthPct = (l.tpo_count / maxCount * 100).toFixed(2);
        const letters = levelLetters(l);
        return `
            <div class="mp-row mp-tier-${tier}">
                <div class="mp-price">${esc(fmtN(l.price))}</div>
                <div class="mp-bar-track">
                    <div class="mp-bar-fill mp-fill-${tier}" data-bar-pct="${widthPct}"></div>
                    <div class="mp-bar-letters">${esc(letters)}</div>
                </div>
                <div class="mp-count">${l.tpo_count}</div>
            </div>
        `;
    }).join('');
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.mp-bar-fill').forEach(el => {
            const pct = Number(el.dataset.barPct);
            if (Number.isFinite(pct)) el.style.width = pct + '%';
        });
    });
}

function showErr(msg) {
    const el = document.getElementById('mp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('mp-err').style.display = 'none'; }
