import { api } from '../api.js';
import { esc, fmtDate, fmtDateTime, fmtMoney, pnlClass } from '../util.js';

export async function renderSearch(mount) {
    mount.innerHTML = `
        <h1 class="view-title">// SEARCH</h1>
        <form id="search-form" class="inline-form" style="margin-bottom:14px">
            <input name="q" placeholder="symbol, journal text, forum post…" autofocus required style="min-width:300px">
            <select name="scope">
                <option value="all">all</option>
                <option value="trades">trades</option>
                <option value="journal">journal</option>
                <option value="forum">forum</option>
            </select>
            <button class="primary" type="submit">Search</button>
        </form>
        <div id="search-results"></div>
    `;
    document.getElementById('search-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const q = fd.get('q');
        const scope = fd.get('scope');
        const el = document.getElementById('search-results');
        el.innerHTML = '<div class="boot">searching…</div>';
        try {
            const r = await api.search(q, scope);
            el.innerHTML = renderHits(r);
        } catch (err) {
            el.innerHTML = `<p class="boot">${err.message}</p>`;
        }
    });
}

function renderHits(r) {
    const blocks = [];
    if (r.trades.length) {
        blocks.push(`
            <div class="chart-panel">
              <h2>Trades · ${r.trades.length}</h2>
              <table class="trades">
                <thead><tr><th>Symbol</th><th>Side</th><th>Status</th><th>Opened</th><th>Net P&L</th></tr></thead>
                <tbody>${r.trades.map(t => `
                    <tr>
                      <td><a href="#trade/${t.id}">${esc(t.symbol)}</a></td>
                      <td>${t.side}</td><td>${t.status}</td>
                      <td>${fmtDateTime(t.opened_at)}</td>
                      <td class="${pnlClass(t.net_pnl)}">${t.net_pnl !== null ? fmtMoney(t.net_pnl) : '—'}</td>
                    </tr>`).join('')}</tbody>
              </table>
            </div>
        `);
    }
    if (r.journal.length) {
        blocks.push(`
            <div class="chart-panel">
              <h2>Journal · ${r.journal.length}</h2>
              ${r.journal.map(j => `
                <div class="journal-entry">
                  <div class="meta">
                    ${j.day ? fmtDate(j.day) : fmtDateTime(j.created_at)}
                    ${j.trade_id ? `· <a href="#trade/${j.trade_id}">trade</a>` : ''}
                    · rank ${j.rank.toFixed(3)}
                  </div>
                  <div class="body">${j.snippet}</div>
                </div>`).join('')}
            </div>
        `);
    }
    if (r.forum.length) {
        blocks.push(`
            <div class="chart-panel">
              <h2>Forum · ${r.forum.length}</h2>
              ${r.forum.map(f => `
                <div class="forum-post">
                  <div class="meta">
                    <a href="#community/${f.category_slug}/${f.thread_slug}">${esc(f.thread_title)}</a>
                    · ${fmtDateTime(f.created_at)} · rank ${f.rank.toFixed(3)}
                  </div>
                  <div class="body">${f.snippet}</div>
                </div>`).join('')}
            </div>
        `);
    }
    return blocks.length
        ? blocks.join('')
        : `<p class="boot">No matches for "${esc(r.query)}".</p>`;
}
