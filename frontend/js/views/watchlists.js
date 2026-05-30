import { api } from '../api.js';
import { go, currentViewToken, viewIsCurrent } from '../app.js';
import { esc, fmt } from '../util.js';
import { tConfirm, tPrompt } from '../dialog.js';

export async function renderWatchlists(mount) {
    const tok = currentViewToken();
    let lists = await api.watchlists();
    if (!viewIsCurrent(tok)) return;
    if (!lists.length) {
        await api.createWatchlist('Main');
        if (!viewIsCurrent(tok)) return;
        lists = await api.watchlists();
        if (!viewIsCurrent(tok)) return;
    }
    const active = lists[0];

    mount.innerHTML = `
        <h1 data-i18n="view.watchlists.h1.watchlists" class="view-title">// WATCHLISTS</h1>

        <div class="chart-panel">
            <div class="inline-form" id="wl-bar">
                ${lists.map(w => `
                    <button class="report-tab ${w.id === active.id ? 'active' : ''}" data-wl="${w.id}">
                        ${esc(w.name)}
                    </button>`).join('')}
                <form id="wl-create" class="inline-form" style="display:inline-flex">
                    <input name="name" placeholder="new watchlist" data-i18n-placeholder="view.watchlists.placeholder.new" required>
                    <button data-i18n="view.watchlists.btn.add_list" class="primary" type="submit">+ Add list</button>
                </form>
            </div>
        </div>

        <div class="chart-panel">
            <h2 id="wl-name">${esc(active.name)}</h2>
            <form id="add-sym" class="inline-form" style="margin-bottom:10px">
                <input name="symbol" placeholder="symbol (e.g. AAPL)" data-i18n-placeholder="view.watchlists.placeholder.symbol" required style="text-transform:uppercase">
                <button data-i18n="view.watchlists.btn.add_symbol" class="primary" type="submit">+ Add symbol</button>
                <button data-i18n="view.watchlists.btn.rename" class="primary" type="button" id="rename-wl"
                    style="background:linear-gradient(180deg,var(--magenta),#7f00b5);border-color:var(--magenta)">Rename</button>
                <button data-i18n="view.watchlists.btn.delete_list" class="link" id="delete-wl">delete list</button>
            </form>
            <div id="wl-table"></div>
        </div>
    `;

    mount.querySelectorAll('[data-wl]').forEach(b =>
        b.addEventListener('click', async () => {
            const list = lists.find(w => w.id === b.dataset.wl);
            if (list) {
                mount.querySelectorAll('[data-wl]').forEach(x => x.classList.toggle('active', x === b));
                const nameEl = mount.querySelector('#wl-name');
                if (nameEl) nameEl.textContent = list.name;
                await refresh(list.id);
            }
        }));

    mount.querySelector('#wl-create').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.createWatchlist(fd.get('name'));
        if (!viewIsCurrent(tok)) return;
        renderWatchlists(mount);
    });

    mount.querySelector('#add-sym').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.addWatchlistSym(active.id, fd.get('symbol').trim().toUpperCase());
        if (!viewIsCurrent(tok)) return;
        e.target.reset();
        await refresh(active.id);
    });

    mount.querySelector('#rename-wl').addEventListener('click', async () => {
        const name = await tPrompt('view.watchlists.prompt.rename', {}, { defaultValue: active.name });
        if (!name) return;
        await api.renameWatchlist(active.id, name);
        if (!viewIsCurrent(tok)) return;
        renderWatchlists(mount);
    });
    mount.querySelector('#delete-wl').addEventListener('click', async () => {
        if (!await tConfirm('view.watchlists.confirm.delete_named', { name: active.name }, { level: 'danger' })) return;
        await api.deleteWatchlist(active.id);
        if (!viewIsCurrent(tok)) return;
        renderWatchlists(mount);
    });

    await refresh(active.id);

    async function refresh(wid) {
        const el = mount.querySelector('#wl-table');
        if (!el) return;
        el.innerHTML = '<div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="view.watchlists.status.loading_quotes">loading quotes…</div></div>';
        const data = await api.watchlistQuotes(wid);
        if (!viewIsCurrent(tok)) return;
        const elNow = mount.querySelector('#wl-table');
        if (!elNow) return;
        if (!data.symbols.length) {
            elNow.innerHTML = '<p data-i18n="view.watchlists.hint.no_symbols_yet_add_one_above" class="muted">No symbols yet. Add one above.</p>';
            return;
        }
        const byKey = new Map(data.quotes.map(q => [q.symbol, q]));
        elNow.innerHTML = `
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.watchlists.th.symbol">Symbol</th><th data-i18n="view.watchlists.th.price">Price</th><th data-i18n="view.watchlists.th.change">Change</th>
                    <th data-i18n="view.watchlists.th.day_hi_lo">Day Hi/Lo</th><th data-i18n="view.watchlists.th.volume">Volume</th><th data-i18n="view.watchlists.th.state">State</th><th></th>
                </tr></thead>
                <tbody>${data.symbols.map(sym => {
                    const q = byKey.get(sym);
                    const ch = q?.change_pct;
                    const cls = ch == null ? '' : (ch >= 0 ? 'pos' : 'neg');
                    return `<tr>
                        <td><a href="#research/${encodeURIComponent(sym)}">${esc(sym)}</a></td>
                        <td>${q ? fmt(q.price) : '—'}</td>
                        <td class="${cls}">${ch != null ? (ch >= 0 ? '+' : '') + ch.toFixed(2) + '%' : '—'}</td>
                        <td>${q?.day_high != null ? fmt(q.day_high) : '—'} /
                            ${q?.day_low  != null ? fmt(q.day_low)  : '—'}</td>
                        <td>${q?.volume != null ? q.volume.toLocaleString() : '—'}</td>
                        <td>${q?.market_state || '—'}</td>
                        <td><button data-i18n="view.watchlists.btn.remove" class="link" data-rm="${sym}">remove</button></td>
                    </tr>`;
                }).join('')}</tbody>
            </table>`;
        elNow.querySelectorAll('[data-rm]').forEach(b =>
            b.addEventListener('click', async () => {
                await api.removeWatchlistSym(wid, b.dataset.rm);
                if (!viewIsCurrent(tok)) return;
                await refresh(wid);
            }));
    }
    void go;
}
