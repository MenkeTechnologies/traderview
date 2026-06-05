// IRC § 1259 — Constructive Sale of Appreciated Financial Position.
// Anti-short-against-the-box rule. If you have a long appreciated position and enter
// into a transaction that "substantially eliminates" risk + opportunity for gain
// (short-against-the-box, equity swap pair, forward at fixed price), treated as SOLD.
// Cap gain recognized at FMV on date of constructive sale. Defeats deferral.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-1259-v1';

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); } catch { return []; } }
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    positions: load(),
};

export async function renderSection1259(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1259.h1.title">// § 1259 CONSTRUCTIVE SALE</span></h1>
        <p class="muted small" data-i18n="view.s1259.hint.intro">
            Anti-<strong>short-against-the-box</strong> rule. If you hold an appreciated long
            position and enter into a hedging transaction that <strong>substantially eliminates
            risk + gain opportunity</strong> — treated as SOLD at FMV on that date. Cap gain
            recognized immediately. Defeats deferral strategies. Common triggers: short-against-the-box,
            equity swap pair, forward sale at fixed price, collar with overlap.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1259.h2.add">Log position + hedge</h2>
            <form id="s1259-form" class="inline-form">
                <label><span data-i18n="view.s1259.label.symbol">Symbol</span>
                    <input type="text" name="symbol" required></label>
                <label><span data-i18n="view.s1259.label.basis">Long basis ($)</span>
                    <input type="number" step="0.01" name="long_basis" required></label>
                <label><span data-i18n="view.s1259.label.fmv">FMV at hedge date ($)</span>
                    <input type="number" step="0.01" name="fmv_at_hedge" required></label>
                <label><span data-i18n="view.s1259.label.hedge_type">Hedge type</span>
                    <select name="hedge_type">
                        <option value="short_against_box">Short-against-the-box</option>
                        <option value="prepaid_forward">Prepaid forward at fixed price</option>
                        <option value="equity_swap">Equity swap (pay fixed return)</option>
                        <option value="collar_overlap">Collar with overlapping strikes</option>
                        <option value="atm_collar">ATM collar (safe — not 1259)</option>
                        <option value="oom_collar">OOM collar (likely safe)</option>
                        <option value="protective_put">Protective put (safe)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1259.label.hedge_date">Hedge date</span>
                    <input type="date" name="hedge_date" required></label>
                <label><span data-i18n="view.s1259.label.unwound">Unwound within 30 days post year-end?</span>
                    <input type="checkbox" name="unwound_in_30_days"></label>
                <label><span data-i18n="view.s1259.label.long_60">Long position held 60+ days post-unwind?</span>
                    <input type="checkbox" name="long_held_60_post"></label>
                <label><span data-i18n="view.s1259.label.marginal">Marginal ordinary %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="0.32"></label>
                <label><span data-i18n="view.s1259.label.ltcg">LTCG %</span>
                    <input type="number" step="0.01" name="ltcg_rate" value="0.20"></label>
                <button class="primary" type="submit" data-i18n="view.s1259.btn.add">Add</button>
            </form>
        </div>
        <div id="s1259-summary"></div>
        <div id="s1259-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1259.h2.safe_harbors">§ 1259(c) Safe Harbors</h2>
            <ul class="muted small">
                <li data-i18n="view.s1259.safe.close_by_jan30">Position closed within 30 days after year-end</li>
                <li data-i18n="view.s1259.safe.long_held_60">Long position held 60+ days after close of identifying transaction without diminished risk</li>
                <li data-i18n="view.s1259.safe.both_required">BOTH conditions required for safe-harbor protection</li>
                <li data-i18n="view.s1259.safe.atm_collar">ATM collars + protective puts generally don't trigger (gain opportunity remains)</li>
                <li data-i18n="view.s1259.safe.delta_hedge">Variable forward / VPF: usually safe IF retain delta &gt; 0.20</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1259.h2.workarounds">Workarounds</h2>
            <ul class="muted small">
                <li data-i18n="view.s1259.work.vpf">Variable Prepaid Forward (VPF): give up upside in narrow band, keep some risk</li>
                <li data-i18n="view.s1259.work.collar">Wide-strike collar maintains real risk + gain opportunity</li>
                <li data-i18n="view.s1259.work.protective_put">Pure protective put: full upside retained, downside hedged</li>
                <li data-i18n="view.s1259.work.exchange_fund">Exchange fund: 7-year hold for diversification without constructive sale</li>
                <li data-i18n="view.s1259.work.margin_loan">Margin loan against unhedged position (no constructive sale)</li>
                <li data-i18n="view.s1259.work.qoz">§ 1400Z Opportunity Zone reinvestment (defers, doesn't avoid)</li>
            </ul>
        </div>
    `;
    document.getElementById('s1259-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.positions.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            symbol: String(fd.get('symbol') || '').toUpperCase(),
            long_basis: Number(fd.get('long_basis')) || 0,
            fmv_at_hedge: Number(fd.get('fmv_at_hedge')) || 0,
            hedge_type: fd.get('hedge_type'),
            hedge_date: fd.get('hedge_date'),
            unwound_in_30_days: !!fd.get('unwound_in_30_days'),
            long_held_60_post: !!fd.get('long_held_60_post'),
            marginal_rate: Number(fd.get('marginal_rate')) || 0.32,
            ltcg_rate: Number(fd.get('ltcg_rate')) || 0.20,
        });
        save(state.positions);
        e.target.reset();
        showToast(t('view.s1259.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function isConstructiveSale(p) {
    const triggers = ['short_against_box', 'prepaid_forward', 'equity_swap', 'collar_overlap'];
    const isTrigger = triggers.includes(p.hedge_type);
    const safeHarbor = p.unwound_in_30_days && p.long_held_60_post;
    return isTrigger && !safeHarbor;
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('s1259-summary');
    if (!el) return;
    const triggered = state.positions.filter(isConstructiveSale);
    const totalGain = triggered.reduce((s, p) => s + Math.max(0, p.fmv_at_hedge - p.long_basis), 0);
    const totalTax = triggered.reduce((s, p) => s + Math.max(0, p.fmv_at_hedge - p.long_basis) * p.ltcg_rate, 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1259.h2.summary">Summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1259.card.count">Positions logged</div>
                    <div class="value">${state.positions.length}</div>
                </div>
                <div class="card ${triggered.length > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1259.card.triggered">Triggered § 1259</div>
                    <div class="value">${triggered.length}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1259.card.gain">Forced gain recognized</div>
                    <div class="value">$${totalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1259.card.tax">Tax cost</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('s1259-table');
    if (!el) return;
    if (!state.positions.length) {
        el.innerHTML = `<h2 data-i18n="view.s1259.h2.positions">Positions</h2>
            <p class="muted" data-i18n="view.s1259.empty">No positions logged.</p>`;
        return;
    }
    const sorted = [...state.positions].sort((a, b) => (b.hedge_date || '').localeCompare(a.hedge_date || ''));
    el.innerHTML = `
        <h2 data-i18n="view.s1259.h2.positions">Positions</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s1259.th.symbol">Symbol</th>
                <th data-i18n="view.s1259.th.basis">Basis</th>
                <th data-i18n="view.s1259.th.fmv">FMV at hedge</th>
                <th data-i18n="view.s1259.th.gain">Embedded gain</th>
                <th data-i18n="view.s1259.th.hedge">Hedge type</th>
                <th data-i18n="view.s1259.th.triggered">§ 1259 triggered?</th>
                <th data-i18n="view.s1259.th.tax">Forced tax</th>
                <th data-i18n="view.s1259.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(p => {
                const gain = Math.max(0, p.fmv_at_hedge - p.long_basis);
                const triggered = isConstructiveSale(p);
                const tax = triggered ? gain * p.ltcg_rate : 0;
                return `<tr>
                    <td>${esc(p.symbol)}</td>
                    <td>$${p.long_basis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${p.fmv_at_hedge.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="pos">$${gain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">${esc(p.hedge_type)}</td>
                    <td class="${triggered ? 'neg' : 'pos'}">${triggered ? esc(t('view.s1259.status.yes')) : esc(t('view.s1259.status.no'))}</td>
                    <td class="${tax > 0 ? 'neg' : ''}">$${tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(p.id)}" data-i18n="view.s1259.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.positions = state.positions.filter(p => p.id !== btn.dataset.del);
            save(state.positions);
            render();
        });
    });
}
