// Forced post-trade reflection: inbox of |R|>=2 trades + 5-question form.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const MOOD_OPTS = [
    [-2, '😡 awful'], [-1, '🙁 down'], [0, '😐 flat'], [1, '🙂 good'], [2, '😄 great'],
];

export async function renderTradeReviews(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) { mount.innerHTML = `<p data-i18n="view.trade_reviews.hint.no_account_selected" class="boot">No account selected.</p>`; return; }
    mount.innerHTML = `
        <h1 class="view-title">// REVIEWS — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p class="muted small">Every closed trade with <strong>|R| ≥ 2</strong> auto-queues
            here for forced reflection. Big wins teach as much as big losses — review them
            both. Five fixed questions per review keep the dataset comparable across hundreds
            of trades.</p>

        <div id="tr-stats" class="cards"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text">loading…</div></div></div>
        <div id="tr-inbox"></div>
        <div id="tr-modal"></div>
        <div id="tr-history"></div>
    `;
    await refresh(acct.id, mount, tok);
}

async function refresh(accountId, mount, tok) {
    try {
        const [s, needed, history] = await Promise.all([
            api.reviewStats(accountId),
            api.reviewsNeeded(accountId),
            api.listReviews(20),
        ]);
        if (!viewIsCurrent(tok)) return;
        renderStats(s, mount);
        renderInbox(needed, accountId, mount, tok);
        renderHistory(history, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const inbox = mount.querySelector('#tr-inbox');
        if (inbox) inbox.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderStats(s, mount) {
    const el = mount.querySelector('#tr-stats');
    if (!el) return;
    const cls = s.completion_pct >= 80 ? 'pos' : s.completion_pct >= 50 ? '' : 'neg';
    el.innerHTML = `
        <div class="card"><div class="label">High-|R| trades</div>
            <div class="value">${s.total_high_r_trades}</div>
            <div class="small muted">closed, |R| ≥ 2</div></div>
        <div class="card"><div class="label">Reviewed</div>
            <div class="value pos">${s.reviewed}</div></div>
        <div class="card"><div class="label">Pending</div>
            <div class="value ${s.pending > 0 ? 'neg' : ''}">${s.pending}</div></div>
        <div class="card"><div class="label">Completion</div>
            <div class="value ${cls}">${s.completion_pct.toFixed(1)}%</div>
            ${s.last_review_at ? `<div class="small muted">last ${new Date(s.last_review_at).toLocaleString()}</div>` : ''}
        </div>
    `;
}

function renderInbox(items, accountId, mount, tok) {
    const el = mount.querySelector('#tr-inbox');
    if (!el) return;
    if (!items.length) {
        el.innerHTML = '<div class="chart-panel"><p data-i18n="view.trade_reviews.hint.inbox_zero_every_high_r_trade_has_been_reviewed" class="muted small">✓ Inbox zero — every high-|R| trade has been reviewed.</p></div>';
        return;
    }
    el.innerHTML = `<div class="chart-panel">
        <h2>Needs review (${items.length})</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.trade_reviews.th.closed">Closed</th><th data-i18n="view.trade_reviews.th.symbol">Symbol</th><th data-i18n="view.trade_reviews.th.side">Side</th>
                <th data-i18n="view.trade_reviews.th.net_p_l">Net P/L</th><th>R</th><th></th>
            </tr></thead>
            <tbody>
            ${items.map(i => `<tr>
                <td class="small">${i.closed_at ? new Date(i.closed_at).toLocaleString() : '—'}</td>
                <td><a href="#trade/${i.trade_id}">${esc(i.symbol)}</a></td>
                <td>${esc(i.side)}</td>
                <td class="${i.net_pnl >= 0 ? 'pos' : 'neg'}">$${fmt(i.net_pnl)}</td>
                <td class="${i.r_multiple >= 0 ? 'pos' : 'neg'}">${(i.r_multiple >= 0 ? '+' : '') + i.r_multiple.toFixed(2)}R</td>
                <td><button data-i18n="view.trade_reviews.btn.review" class="btn tr-open" data-tid="${i.trade_id}" data-sym="${esc(i.symbol)}" data-r="${i.r_multiple.toFixed(2)}">Review</button></td>
            </tr>`).join('')}
            </tbody>
        </table>
    </div>`;
    el.querySelectorAll('.tr-open').forEach(b =>
        b.addEventListener('click', () => openModal(b.dataset.tid, b.dataset.sym, b.dataset.r, accountId, mount, tok)));
}

async function openModal(tradeId, symbol, rMult, accountId, mount, tok) {
    let existing = null;
    try { existing = await api.reviewForTrade(tradeId); } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const m = mount.querySelector('#tr-modal');
    if (!m) return;
    m.innerHTML = `
        <div style="position:fixed;inset:0;background:rgba(7,7,20,0.85);z-index:100;display:flex;align-items:center;justify-content:center;padding:20px;">
            <div class="chart-panel" style="max-width:640px;width:100%;">
                <h2>Review ${esc(symbol)} (${esc(rMult)}R)</h2>
                <form id="tr-form" class="inline-form" style="flex-direction:column;align-items:stretch;gap:10px;">
                    <label style="display:flex;justify-content:space-between;">
                        <span>1. Was the entry per plan?</span>
                        <span>
                            <label><input type="radio" name="entry_per_plan" value="yes" ${existing?.entry_per_plan === true ? 'checked' : ''}> yes</label>
                            <label><input type="radio" name="entry_per_plan" value="no"  ${existing?.entry_per_plan === false ? 'checked' : ''}> no</label>
                        </span>
                    </label>
                    <label style="display:flex;justify-content:space-between;">
                        <span>2. Was the exit per plan?</span>
                        <span>
                            <label><input type="radio" name="exit_per_plan" value="yes" ${existing?.exit_per_plan === true ? 'checked' : ''}> yes</label>
                            <label><input type="radio" name="exit_per_plan" value="no"  ${existing?.exit_per_plan === false ? 'checked' : ''}> no</label>
                        </span>
                    </label>
                    <label style="display:flex;flex-direction:column;">
                        <span>3. What would you change next time?</span>
                        <textarea name="would_change" rows="3"
                            style="background:#070714;color:#cfd2e8;border:1px solid var(--border);padding:6px;font-family:inherit;">${esc(existing?.would_change || '')}</textarea>
                    </label>
                    <label style="display:flex;justify-content:space-between;align-items:center;">
                        <span>4. Mood at exit</span>
                        <select name="mood_at_exit">
                            <option value="">—</option>
                            ${MOOD_OPTS.map(([v, l]) =>
                                `<option value="${v}" ${existing?.mood_at_exit === v ? 'selected' : ''}>${esc(l)}</option>`
                            ).join('')}
                        </select>
                    </label>
                    <label style="display:flex;flex-direction:column;">
                        <span>5. Setup classifier (one tag)</span>
                        <input name="setup_tag" placeholder="e.g. breakout / fade / news / squeeze"
                            value="${esc(existing?.setup_tag || '')}"
                            style="background:#070714;color:#cfd2e8;border:1px solid var(--border);padding:6px;">
                    </label>
                    <div style="display:flex;gap:8px;justify-content:flex-end;">
                        <button data-i18n="view.trade_reviews.btn.cancel" type="button" class="btn" id="tr-cancel">Cancel</button>
                        <button data-i18n="view.trade_reviews.btn.save_review" class="primary" type="submit">Save review</button>
                    </div>
                </form>
            </div>
        </div>
    `;
    const close = () => { m.innerHTML = ''; };
    m.querySelector('#tr-cancel').addEventListener('click', close);
    m.querySelector('#tr-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {
            trade_id: tradeId,
            entry_per_plan: fd.get('entry_per_plan') === 'yes' ? true
                          : fd.get('entry_per_plan') === 'no'  ? false : null,
            exit_per_plan:  fd.get('exit_per_plan')  === 'yes' ? true
                          : fd.get('exit_per_plan')  === 'no'  ? false : null,
            would_change: fd.get('would_change') || null,
            mood_at_exit: fd.get('mood_at_exit') ? Number(fd.get('mood_at_exit')) : null,
            setup_tag:    fd.get('setup_tag') || null,
        };
        try {
            await api.saveReview(body);
            if (!viewIsCurrent(tok)) return;
            close();
            await refresh(accountId, mount, tok);
        } catch (err) { alert(err.message); }
    });
}

function renderHistory(rows, mount) {
    const el = mount.querySelector('#tr-history');
    if (!el) return;
    if (!rows.length) { el.innerHTML = ''; return; }
    el.innerHTML = `<div class="chart-panel">
        <h2>Recent reviews (${rows.length})</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.trade_reviews.th.when">When</th><th data-i18n="view.trade_reviews.th.trade">Trade</th><th data-i18n="view.trade_reviews.th.entry_exit_per_plan">Entry / Exit per plan</th>
                <th data-i18n="view.trade_reviews.th.mood">Mood</th><th data-i18n="view.trade_reviews.th.setup">Setup</th><th data-i18n="view.trade_reviews.th.would_change">Would change</th>
            </tr></thead>
            <tbody>
            ${rows.map(r => {
                const mood = (MOOD_OPTS.find(([v]) => v === r.mood_at_exit) || ['', '—'])[1];
                const tick = b => b === true ? '<span class="pos">✓</span>'
                                : b === false ? '<span class="neg">✗</span>'
                                : '<span class="muted">—</span>';
                return `<tr>
                    <td class="small">${new Date(r.completed_at).toLocaleString()}</td>
                    <td><a href="#trade/${r.trade_id}">${r.trade_id.slice(0, 8)}…</a></td>
                    <td class="small">${tick(r.entry_per_plan)} / ${tick(r.exit_per_plan)}</td>
                    <td>${esc(mood)}</td>
                    <td class="small">${esc(r.setup_tag || '')}</td>
                    <td class="small muted">${esc((r.would_change || '').slice(0, 80))}</td>
                </tr>`;
            }).join('')}
            </tbody>
        </table>
    </div>`;
}
