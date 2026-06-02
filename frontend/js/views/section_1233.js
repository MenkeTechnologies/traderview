// IRC § 1233 — Short Sale Holding Period Rules.
// Gain on closing short sale: holding period determined by HOLD on STOCK at TIME of short, not at close.
// Loss on short: long-term if held > 1 yr; otherwise short-term.
// If holding identical stock long before short → "covered short" — holding period MAY suspend.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-1233-v1';

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); } catch { return []; } }
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    shorts: load(),
};

export async function renderSection1233(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1233.h1.title">// § 1233 SHORT SALE HOLDING PERIOD</span></h1>
        <p class="muted small" data-i18n="view.s1233.hint.intro">
            Short sale gain: holding period determined by stock <strong>HELD at time of short</strong>,
            not closing. Loss on short: long-term if &gt; 1 yr; otherwise short-term.
            <strong>"Short-against-the-box" or "covered short":</strong> holding identical
            stock long before short → § 1233(b) suspends OR resets holding period of long.
            <strong>§ 1259 constructive sale</strong> takes precedence for substantially-eliminated risk.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1233.h2.add">Log short position</h2>
            <form id="s1233-form" class="inline-form">
                <label><span data-i18n="view.s1233.label.symbol">Symbol</span>
                    <input type="text" name="symbol" required></label>
                <label><span data-i18n="view.s1233.label.short_date">Date of short</span>
                    <input type="date" name="short_date" required></label>
                <label><span data-i18n="view.s1233.label.close_date">Date of close</span>
                    <input type="date" name="close_date"></label>
                <label><span data-i18n="view.s1233.label.short_proceeds">Short proceeds ($)</span>
                    <input type="number" step="0.01" name="short_proceeds" required></label>
                <label><span data-i18n="view.s1233.label.cover_cost">Cover cost ($)</span>
                    <input type="number" step="0.01" name="cover_cost"></label>
                <label><span data-i18n="view.s1233.label.long_held_at_short">Held same stock long at short date?</span>
                    <input type="checkbox" name="long_held_at_short"></label>
                <label><span data-i18n="view.s1233.label.long_acquired">Long stock acquired date</span>
                    <input type="date" name="long_acquired_date"></label>
                <label><span data-i18n="view.s1233.label.long_more_year">Long stock held &gt; 1 yr at short?</span>
                    <input type="checkbox" name="long_more_than_year_at_short"></label>
                <button class="primary" type="submit" data-i18n="view.s1233.btn.add">Add</button>
            </form>
        </div>
        <div id="s1233-summary"></div>
        <div id="s1233-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1233.h2.rules">§ 1233 rules summary</h2>
            <ul class="muted small">
                <li data-i18n="view.s1233.rule.gain_simple">Pure short with no long held: gain ALWAYS short-term (you owned the borrowed share &lt; 1 yr)</li>
                <li data-i18n="view.s1233.rule.loss_holdings">Loss on close: looks at how long you owned the SOLD stock (the close buy)</li>
                <li data-i18n="view.s1233.rule.long_held_under_yr">Long held &lt; 1 yr + short = long holding period RESTARTS at short close</li>
                <li data-i18n="view.s1233.rule.long_held_over_yr">Long held &gt; 1 yr + short = LT character protected for long; short still ST</li>
                <li data-i18n="view.s1233.rule.box_disposes">Closing the short "against the box" = sells the LONG (taxable disposition)</li>
                <li data-i18n="view.s1233.rule.identical">"Substantially identical" = same stock, same class, similar economic exposure</li>
                <li data-i18n="view.s1233.rule.wash_sale">Wash sale § 1091 applies + complicates basis</li>
                <li data-i18n="view.s1233.rule.475f_overrides">§ 475(f) trader MTM overrides § 1233 (everything ordinary)</li>
            </ul>
        </div>
    `;
    document.getElementById('s1233-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.shorts.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            symbol: String(fd.get('symbol') || '').toUpperCase(),
            short_date: fd.get('short_date'),
            close_date: fd.get('close_date') || null,
            short_proceeds: Number(fd.get('short_proceeds')) || 0,
            cover_cost: Number(fd.get('cover_cost')) || 0,
            long_held_at_short: !!fd.get('long_held_at_short'),
            long_acquired_date: fd.get('long_acquired_date') || null,
            long_more_than_year_at_short: !!fd.get('long_more_than_year_at_short'),
        });
        save(state.shorts);
        e.target.reset();
        showToast(t('view.s1233.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function classify(s) {
    const pnl = s.short_proceeds - s.cover_cost;
    if (s.close_date === null) return { closed: false, pnl: 0, character: '—' };
    const isGain = pnl > 0;
    let character;
    if (s.long_held_at_short && s.long_more_than_year_at_short) {
        character = isGain ? 'st' : 'lt';
    } else if (s.long_held_at_short && !s.long_more_than_year_at_short) {
        character = 'st';
    } else {
        character = 'st';
    }
    return { closed: true, pnl, character };
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('s1233-summary');
    if (!el) return;
    const closed = state.shorts.filter(s => s.close_date);
    const st = closed.filter(s => classify(s).character === 'st');
    const lt = closed.filter(s => classify(s).character === 'lt');
    const totalGain = closed.reduce((s, sh) => s + Math.max(0, classify(sh).pnl), 0);
    const totalLoss = closed.reduce((s, sh) => s + Math.min(0, classify(sh).pnl), 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1233.h2.summary">Summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1233.card.count">Shorts logged</div>
                    <div class="value">${state.shorts.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1233.card.closed">Closed positions</div>
                    <div class="value">${closed.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1233.card.st_count">Short-term character</div>
                    <div class="value">${st.length}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1233.card.lt_count">Long-term character</div>
                    <div class="value">${lt.length}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1233.card.gain">Total gain</div>
                    <div class="value">$${totalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1233.card.loss">Total loss</div>
                    <div class="value">$${totalLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('s1233-table');
    if (!el) return;
    if (!state.shorts.length) {
        el.innerHTML = `<h2 data-i18n="view.s1233.h2.shorts">Short positions</h2>
            <p class="muted" data-i18n="view.s1233.empty">No shorts logged.</p>`;
        return;
    }
    const sorted = [...state.shorts].sort((a, b) => (b.short_date || '').localeCompare(a.short_date || ''));
    el.innerHTML = `
        <h2 data-i18n="view.s1233.h2.shorts">Short positions</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s1233.th.symbol">Symbol</th>
                <th data-i18n="view.s1233.th.short_date">Short date</th>
                <th data-i18n="view.s1233.th.close">Close date</th>
                <th data-i18n="view.s1233.th.pnl">P&amp;L</th>
                <th data-i18n="view.s1233.th.character">Character</th>
                <th data-i18n="view.s1233.th.long_held">Long held?</th>
                <th data-i18n="view.s1233.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(s => {
                const c = classify(s);
                const charLabel = c.character === 'lt' ? t('view.s1233.char.lt') : (c.character === 'st' ? t('view.s1233.char.st') : '—');
                const charCls = c.character === 'lt' ? 'pos' : 'muted';
                return `<tr>
                    <td>${esc(s.symbol)}</td>
                    <td class="muted">${esc(s.short_date)}</td>
                    <td class="muted">${esc(s.close_date || '—')}</td>
                    <td class="${c.pnl >= 0 ? 'pos' : 'neg'}">$${c.pnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${charCls}">${esc(charLabel)}</td>
                    <td class="${s.long_held_at_short ? 'neg' : 'pos'}">${s.long_held_at_short ? esc(t('view.s1233.status.yes')) : esc(t('view.s1233.status.no'))}</td>
                    <td><button class="link neg" data-del="${esc(s.id)}" data-i18n="view.s1233.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.shorts = state.shorts.filter(s => s.id !== btn.dataset.del);
            save(state.shorts);
            render();
        });
    });
}
