// Forex Desk — live USD majors, the 24/5 session clock, and the
// position calculators (pip value + risk-based sizing) over the FX
// backend. Carry / CIP forwards live in the Strategy Tools view
// (/calc/fx-carry); this view does not duplicate them.

import { api } from '../api.js';
import { esc } from '../util.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const SESSION_LABEL = {
    sydney: 'Sydney',
    tokyo: 'Tokyo',
    london: 'London',
    newyork: 'New York',
};

const num = (n, dp = 2) =>
    n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: dp });

export async function renderForex(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.forex.h1.title">// FOREX DESK</span></h1>
        <p class="muted small" data-i18n="view.forex.hint.intro">
            FX is a first-class asset class here: EURUSD-form pairs quote and chart
            through the same seam as equities and crypto, and fill in the paper engine
            with a spread cost model. Below: which centers are open now, live majors,
            and the pip / position-size calculators.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.forex.h2.sessions">Trading sessions (UTC)</h2>
            <div id="fx-sessions" class="muted small" data-i18n="view.forex.loading">Loading…</div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.forex.h2.majors">Major pairs</h2>
            <div id="fx-majors" data-i18n="view.forex.loading">Loading…</div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.forex.h2.position_size">Position size (risk-based)</h2>
            <form id="fx-size-form" class="inline-form">
                <label><span data-i18n="view.forex.label.pair">Pair</span>
                    <input type="text" name="pair" value="EURUSD" required></label>
                <label><span data-i18n="view.forex.label.equity">Account equity ($)</span>
                    <input type="number" step="0.01" name="equity" value="10000" min="0" required></label>
                <label><span data-i18n="view.forex.label.risk_pct">Risk %</span>
                    <input type="number" step="0.1" name="risk_pct" value="1" min="0" required></label>
                <label><span data-i18n="view.forex.label.stop_pips">Stop (pips)</span>
                    <input type="number" step="0.1" name="stop_pips" value="20" min="0" required></label>
                <button class="primary" type="submit" data-i18n="view.forex.btn.size">Size it</button>
            </form>
            <div id="fx-size-result"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.forex.h2.pip_value">Pip value</h2>
            <form id="fx-pip-form" class="inline-form">
                <label><span data-i18n="view.forex.label.pair">Pair</span>
                    <input type="text" name="pair" value="EURUSD" required></label>
                <label><span data-i18n="view.forex.label.units">Units</span>
                    <input type="number" step="1" name="units" value="100000" min="0" required></label>
                <button class="primary" type="submit" data-i18n="view.forex.btn.pip">Compute</button>
            </form>
            <div id="fx-pip-result" class="muted small"></div>
        </div>
    `;
    applyUiI18n(mount);

    mount.querySelector('#fx-size-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        try {
            const r = await api.forexPositionSize({
                pair: fd.get('pair'),
                equity: Number(fd.get('equity')),
                risk_pct: Number(fd.get('risk_pct')),
                stop_pips: Number(fd.get('stop_pips')),
            });
            if (!viewIsCurrent(tok)) return;
            renderSizeResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.forex.toast.error'), { level: 'error' });
        }
    });

    mount.querySelector('#fx-pip-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        try {
            const v = await api.forexPipValue({
                pair: fd.get('pair'),
                units: Number(fd.get('units')),
            });
            if (!viewIsCurrent(tok)) return;
            const el = mount.querySelector('#fx-pip-result');
            el.textContent = `${t('view.forex.pip_result')}: ${num(v, 4)}`;
        } catch (err) {
            showToast(err.message || t('view.forex.toast.error'), { level: 'error' });
        }
    });

    await Promise.all([loadSessions(mount, tok), loadMajors(mount, tok)]);
}

function renderSizeResult(mount, r) {
    const el = mount.querySelector('#fx-size-result');
    el.innerHTML = `
        <div class="cards">
            <div class="card pos">
                <div class="label" data-i18n="view.forex.card.units">Units</div>
                <div class="value">${num(r.units, 0)}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.forex.card.std_lots">Standard lots</div>
                <div class="value">${num(r.standard_lots, 3)}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.forex.card.mini_lots">Mini lots</div>
                <div class="value">${num(r.mini_lots, 2)}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.forex.card.risk_amount">Risk amount</div>
                <div class="value">$${num(r.risk_amount, 2)}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.forex.card.pip_value">Pip value</div>
                <div class="value">${num(r.pip_value, 2)}</div>
            </div>
        </div>
    `;
    applyUiI18n(el);
}

async function loadSessions(mount, tok) {
    const el = mount.querySelector('#fx-sessions');
    try {
        const s = await api.forexSessions();
        if (!viewIsCurrent(tok)) return;
        if (!s.market_open) {
            el.innerHTML = `<span class="neg" data-i18n="view.forex.sessions.closed">Market closed (weekend)</span>`;
            applyUiI18n(el);
            return;
        }
        const active = (s.active || []).map((k) => esc(SESSION_LABEL[k] || k)).join(', ') || '—';
        const overlap = s.london_ny_overlap
            ? ` · <span class="pos" data-i18n="view.forex.sessions.overlap">London/NY overlap — deepest liquidity</span>`
            : '';
        el.innerHTML = `<span class="pos" data-i18n="view.forex.sessions.open">Market open</span> · ${active}${overlap}`;
        applyUiI18n(el);
    } catch {
        if (viewIsCurrent(tok)) el.textContent = t('view.forex.toast.error');
    }
}

async function loadMajors(mount, tok) {
    const el = mount.querySelector('#fx-majors');
    try {
        const rows = await api.forexPairs();
        if (!viewIsCurrent(tok)) return;
        if (!rows || !rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.forex.majors.empty">No quotes available.</p>`;
            applyUiI18n(el);
            return;
        }
        el.innerHTML = `
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.forex.th.pair">Pair</th>
                    <th data-i18n="view.forex.th.price">Price</th>
                    <th data-i18n="view.forex.th.change">Change %</th>
                </tr></thead>
                <tbody>${rows.map((q) => {
                    const chg = q.change_pct;
                    const cls = chg == null ? '' : chg >= 0 ? 'pos' : 'neg';
                    return `<tr>
                        <td>${esc(q.symbol)}</td>
                        <td>${num(q.price, 5)}</td>
                        <td class="${cls}">${chg == null ? '—' : num(chg, 2) + '%'}</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
        `;
        applyUiI18n(el);
    } catch {
        if (viewIsCurrent(tok)) el.textContent = t('view.forex.toast.error');
    }
}
