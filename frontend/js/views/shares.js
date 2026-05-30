import { api } from '../api.js';
import { esc, fmt, fmtDateTime, fmtMoney, md, pnlClass } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';

export async function renderShares(mount) {
    const tok = currentViewToken();
    const [mine, pub] = await Promise.all([api.sharesMine(), api.sharesPublic()]);
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.shares.h1.shares" class="view-title">// SHARES</h1>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.shares.h2.your_shared_trades">Your shared trades</h2>
                ${shareTable(mine, true)}
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.shares.h2.public_stream">Public stream</h2>
                ${shareTable(pub, false)}
            </div>
        </div>
    `;
    mount.querySelectorAll('[data-del-share]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteShare(b.dataset.delShare);
            if (!viewIsCurrent(tok)) return;
            renderShares(mount);
        }));
}

function shareTable(rows, mine) {
    if (!rows.length) return '<p data-i18n="view.shares.hint.none" class="muted">None.</p>';
    return `<table class="trades">
        <thead><tr><th data-i18n="view.shares.th.slug">Slug</th><th data-i18n="view.shares.th.views">Views</th><th data-i18n="view.shares.th.created">Created</th>${mine ? '<th></th>' : ''}</tr></thead>
        <tbody>${rows.map(s => `
            <tr><td><a href="#shared/${s.slug}">${s.slug}</a></td>
            <td>${s.view_count}</td>
            <td>${fmtDateTime(s.created_at)}</td>
            ${mine ? `<td><button data-i18n="view.shares.btn.delete" class="link" data-del-share="${s.id}">delete</button></td>` : ''}
            </tr>
        `).join('')}</tbody></table>`;
}

export async function renderSharedTrade(mount, _state, slug) {
    const tok = currentViewToken();
    if (!slug) { mount.innerHTML = '<p data-i18n="view.shares.hint.no_slug" class="boot">No slug.</p>'; return; }
    const [view, comments] = await Promise.all([
        api.viewShared(slug),
        api.comments(slug).catch(_ => []),
    ]);
    if (!viewIsCurrent(tok)) return;
    const tr = view.trade;
    mount.innerHTML = `
        <h1 class="view-title">// SHARED · ${esc(tr.symbol)}</h1>
        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.shares.card.net_pnl">Net P&L</div>
                <div class="value ${pnlClass(tr.net_pnl)}">${fmtMoney(tr.net_pnl)}</div></div>
            <div class="card"><div class="label" data-i18n="view.shares.card.side">Side</div><div class="value">${tr.side}</div></div>
            <div class="card"><div class="label" data-i18n="view.shares.card.qty">Qty</div><div class="value">${fmt(tr.qty, 0)}</div></div>
            <div class="card"><div class="label" data-i18n="view.shares.card.entry">Entry</div><div class="value">${fmt(tr.entry_avg)}</div></div>
            <div class="card"><div class="label" data-i18n="view.shares.card.exit">Exit</div>
                <div class="value">${tr.exit_avg !== null ? fmt(tr.exit_avg) : '—'}</div></div>
            <div class="card"><div class="label" data-i18n="view.shares.card.asset">Asset</div><div class="value">${tr.asset_class}</div></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.shares.h2.comments">Comments</h2>
            <div id="comments">${comments.map(c => `
                <div class="comment">
                    <div class="meta">${fmtDateTime(c.created_at)}</div>
                    <div class="body">${md(c.body_md)}</div>
                </div>
            `).join('') || '<p data-i18n="view.shares.hint.be_the_first_to_comment" class="muted">Be the first to comment.</p>'}</div>
            <form id="comment-form">
                <textarea name="body" placeholder="comment (markdown)" data-i18n-placeholder="view.shares.placeholder.comment" required></textarea>
                <button data-i18n="view.shares.btn.post" class="primary" type="submit">Post</button>
            </form>
        </div>
    `;
    mount.querySelector('#comment-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        try {
            await api.postComment(slug, fd.get('body'));
            if (!viewIsCurrent(tok)) return;
            renderSharedTrade(mount, _state, slug);
        } catch (err) {
            showToast(t('common.error', { err: err.message }), { level: 'error' });
        }
    });
}
