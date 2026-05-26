import { api } from '../api.js';
import { esc, fmt, fmtDateTime, fmtMoney, md, pnlClass } from '../util.js';

export async function renderShares(mount) {
    const [mine, pub] = await Promise.all([api.sharesMine(), api.sharesPublic()]);
    mount.innerHTML = `
        <h1 class="view-title">// SHARES</h1>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2>Your shared trades</h2>
                ${shareTable(mine, true)}
            </div>
            <div class="chart-panel">
                <h2>Public stream</h2>
                ${shareTable(pub, false)}
            </div>
        </div>
    `;
    document.querySelectorAll('[data-del-share]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteShare(b.dataset.delShare);
            renderShares(mount);
        }));
}

function shareTable(rows, mine) {
    if (!rows.length) return '<p class="muted">None.</p>';
    return `<table class="trades">
        <thead><tr><th>Slug</th><th>Views</th><th>Created</th>${mine ? '<th></th>' : ''}</tr></thead>
        <tbody>${rows.map(s => `
            <tr><td><a href="#shared/${s.slug}">${s.slug}</a></td>
            <td>${s.view_count}</td>
            <td>${fmtDateTime(s.created_at)}</td>
            ${mine ? `<td><button class="link" data-del-share="${s.id}">delete</button></td>` : ''}
            </tr>
        `).join('')}</tbody></table>`;
}

export async function renderSharedTrade(mount, _state, slug) {
    if (!slug) { mount.innerHTML = '<p class="boot">No slug.</p>'; return; }
    const [view, comments] = await Promise.all([
        api.viewShared(slug),
        api.comments(slug).catch(_ => []),
    ]);
    const t = view.trade;
    mount.innerHTML = `
        <h1 class="view-title">// SHARED · ${esc(t.symbol)}</h1>
        <div class="cards">
            <div class="card"><div class="label">Net P&L</div>
                <div class="value ${pnlClass(t.net_pnl)}">${fmtMoney(t.net_pnl)}</div></div>
            <div class="card"><div class="label">Side</div><div class="value">${t.side}</div></div>
            <div class="card"><div class="label">Qty</div><div class="value">${fmt(t.qty, 0)}</div></div>
            <div class="card"><div class="label">Entry</div><div class="value">${fmt(t.entry_avg)}</div></div>
            <div class="card"><div class="label">Exit</div>
                <div class="value">${t.exit_avg !== null ? fmt(t.exit_avg) : '—'}</div></div>
            <div class="card"><div class="label">Asset</div><div class="value">${t.asset_class}</div></div>
        </div>

        <div class="chart-panel">
            <h2>Comments</h2>
            <div id="comments">${comments.map(c => `
                <div class="comment">
                    <div class="meta">${fmtDateTime(c.created_at)}</div>
                    <div class="body">${md(c.body_md)}</div>
                </div>
            `).join('') || '<p class="muted">Be the first to comment.</p>'}</div>
            <form id="comment-form">
                <textarea name="body" placeholder="comment (markdown)" required></textarea>
                <button class="primary" type="submit">Post</button>
            </form>
        </div>
    `;
    document.getElementById('comment-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        try {
            await api.postComment(slug, fd.get('body'));
            renderSharedTrade(mount, _state, slug);
        } catch (err) {
            alert(err.message);
        }
    });
}
