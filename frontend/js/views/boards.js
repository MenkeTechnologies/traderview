// Custom dashboards / boards — drag-from-palette grid builder.
// Routes:
//   #boards              → list / create boards
//   #boards/<id>         → builder for one board
//
// Layout JSON shape per widget:
//   { id: "wid_xxx", kind: "quote", params: { symbol: "SPY" },
//     x: 0, y: 0, w: 4, h: 2 }
//
// Grid is 12 columns wide; row height fixed at 96px. Widgets fill x..x+w
// columns and y..y+h rows. We don't pack — overlap is allowed and on the
// user to fix; this keeps the builder simple and predictable.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const WIDGET_KINDS = [
    { kind: 'quote',      label: 'Quote',        defaults: { symbol: 'SPY' },     w: 3, h: 2 },
    { kind: 'mini_chart', label: t('chart.series.mini_chart'),   defaults: { symbol: 'SPY' },     w: 6, h: 3 },
    { kind: 'breadth',    label: t('chart.series.breadth_pulse'),defaults: {},                    w: 4, h: 2 },
    { kind: 'vix',        label: 'VIX',          defaults: {},                    w: 3, h: 2 },
    { kind: 'watchlist',  label: 'Watchlist',    defaults: { limit: 10 },         w: 4, h: 4 },
    { kind: 'alerts',     label: t('chart.series.alerts_feed'),  defaults: { limit: 10 },         w: 6, h: 4 },
    { kind: 'fear_greed', label: t('chart.series.fear_greed'), defaults: {},                    w: 3, h: 2 },
    { kind: 'news',       label: t('chart.series.news_symbol'),defaults: { symbol: 'SPY', limit: 6 }, w: 6, h: 4 },
    { kind: 'note',       label: t('chart.series.sticky_note'),  defaults: { text: '' },          w: 3, h: 2 },
];

// Look up a widget's display label, preferring the i18n catalog and
// falling back to the literal `label:` field in WIDGET_KINDS (or the
// raw kind id if the widget is unknown to the palette).
function widgetLabel(kind) {
    const k = WIDGET_KINDS.find(w => w.kind === kind);
    const key = `view.boards.widget.${kind}.label`;
    const v = t(key);
    if (v && v !== key) return v;
    return (k && k.label) || kind;
}

const COLS = 12;
const ROW_HEIGHT = 96;
let refreshTimers = [];

export async function renderBoards(mount, _state, id = '') {
    clearTimers();
    if (!id) return renderList(mount);
    return renderBoard(mount, id);
}

// ---------------------------------------------------------------------------
// List view
// ---------------------------------------------------------------------------

async function renderList(mount) {
    const tok = currentViewToken();
    const boards = await api.listDashboards().catch(() => []);
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.boards.h1.boards" class="view-title">// BOARDS</h1>
        <p data-i18n="view.boards.hint.custom_dashboards_open_each_board_in_its_own_brows" class="muted small">Custom dashboards. Open each board in its own browser window
            and snap to a monitor for a multi-screen trading rig. Drag widgets from the
            palette to add; click × to remove; drag widget headers to reposition.</p>

        <div class="chart-panel">
            <form id="b-new" class="inline-form">
                <input name="name" placeholder="board name" data-i18n-placeholder="view.boards.placeholder.name" required style="min-width:240px;">
                <button data-i18n="view.boards.btn.create_board" class="primary" type="submit">Create board</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2>${esc(t('view.boards.h2.your_boards', { count: boards.length }))}</h2>
            ${boards.length === 0
                ? '<p data-i18n="view.boards.hint.no_boards_yet" class="muted small">No boards yet.</p>'
                : `<table class="trades">
                    <thead><tr><th data-i18n="view.boards.th.name">Name</th><th data-i18n="view.boards.th.widgets">Widgets</th><th data-i18n="view.boards.th.updated">Updated</th><th></th></tr></thead>
                    <tbody>
                        ${boards.map(b => `<tr>
                            <td><a href="#boards/${b.id}">${esc(b.name)}</a></td>
                            <td>${Array.isArray(b.layout) ? b.layout.length : 0}</td>
                            <td class="small">${new Date(b.updated_at).toLocaleString()}</td>
                            <td><button data-i18n="view.boards.btn.delete" class="btn b-del" data-id="${b.id}">Delete</button></td>
                        </tr>`).join('')}
                    </tbody>
                </table>`}
        </div>
    `;
    mount.querySelector('#b-new').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        try {
            const b = await api.createDashboard({ name: fd.get('name').trim(), layout: [] });
            window.location.hash = `boards/${b.id}`;
        } catch (err) { alert(t('common.error', { err: err.message })); }
    });
    mount.querySelectorAll('.b-del').forEach(btn => {
        btn.addEventListener('click', async () => {
            if (!confirm(t('view.boards.confirm.delete'))) return;
            try { await api.deleteDashboard(btn.dataset.id); if (viewIsCurrent(tok)) renderList(mount); }
            catch (e) { alert(t('common.error', { err: e.message })); }
        });
    });
}

// ---------------------------------------------------------------------------
// Board view
// ---------------------------------------------------------------------------

async function renderBoard(mount, id) {
    const tok = currentViewToken();
    let board;
    try { board = await api.getDashboard(id); }
    catch (e) { if (!viewIsCurrent(tok)) return; mount.innerHTML = `<p class="boot">${esc(e.message)}</p>`; return; }
    if (!viewIsCurrent(tok)) return;

    const state = { board, dirty: false, drag: null, tok, mount };

    mount.innerHTML = `
        <h1 class="view-title">// BOARD — <span id="b-name">${esc(board.name)}</span>
            <button data-i18n="view.boards.btn.rename" id="b-rename" class="btn" style="font-size:10px;">rename</button>
            <span id="b-save" class="muted small" style="margin-left:8px;" data-i18n="common.saved">saved</span>
        </h1>

        <div class="chart-panel">
            <h2 data-i18n="view.boards.h2.palette_drag_onto_grid">Palette — drag onto grid</h2>
            <div id="palette" style="display:flex;flex-wrap:wrap;gap:6px;">
                ${WIDGET_KINDS.map(k => {
                    const labelKey = `view.boards.widget.${k.kind}.label`;
                    const lv = t(labelKey);
                    const labelTr = (lv && lv !== labelKey) ? lv : k.label;
                    return `<div class="palette-tile" draggable="true" data-kind="${k.kind}"
                         style="border:1px solid var(--cyan);padding:6px 10px;cursor:grab;background:var(--bg-secondary);">
                        ${esc(labelTr)}
                    </div>`;
                }).join('')}
            </div>
        </div>

        <div class="chart-panel">
            <div id="b-grid" style="
                display:grid;
                grid-template-columns: repeat(${COLS}, 1fr);
                grid-auto-rows: ${ROW_HEIGHT}px;
                gap: 8px;
                min-height: ${ROW_HEIGHT * 4}px;
                background: repeating-linear-gradient(
                    to right, transparent 0,
                    transparent calc(100%/${COLS} - 1px),
                    rgba(0,229,255,0.06) calc(100%/${COLS} - 1px),
                    rgba(0,229,255,0.06) calc(100%/${COLS}));
                position: relative;
                padding: 4px;
            "></div>
        </div>
    `;

    mount.querySelector('#b-rename').addEventListener('click', async () => {
        const next = prompt(t('view.boards.prompt.name'), state.board.name);
        if (!next || next === state.board.name) return;
        state.board.name = next;
        const nm = mount.querySelector('#b-name');
        if (nm) nm.textContent = next;
        await persist(state);
    });

    bindPaletteDrag(state);
    renderGrid(state);
}

// ---------------------------------------------------------------------------
// Grid render + drag
// ---------------------------------------------------------------------------

function bindPaletteDrag(state) {
    state.mount.querySelectorAll('.palette-tile').forEach(el => {
        el.addEventListener('dragstart', (e) => {
            state.drag = { kind: el.dataset.kind, source: 'palette' };
            e.dataTransfer.effectAllowed = 'copy';
        });
    });
    const grid = state.mount.querySelector('#b-grid');
    if (!grid) return;
    grid.addEventListener('dragover', (e) => {
        if (!state.drag) return;
        e.preventDefault();
        e.dataTransfer.dropEffect = state.drag.source === 'palette' ? 'copy' : 'move';
    });
    grid.addEventListener('drop', (e) => {
        if (!state.drag) return;
        e.preventDefault();
        const { col, row } = pointToCell(grid, e);
        if (state.drag.source === 'palette') {
            const k = WIDGET_KINDS.find(w => w.kind === state.drag.kind);
            const widget = {
                id: `wid_${Math.random().toString(36).slice(2, 10)}`,
                kind: k.kind, params: { ...k.defaults },
                x: clamp(col, 0, COLS - k.w), y: Math.max(row, 0),
                w: k.w, h: k.h,
            };
            state.board.layout = [...(state.board.layout || []), widget];
        } else if (state.drag.source === 'widget') {
            const w = state.board.layout.find(x => x.id === state.drag.id);
            if (w) {
                w.x = clamp(col, 0, COLS - w.w);
                w.y = Math.max(row, 0);
            }
        }
        state.drag = null;
        persist(state);
        renderGrid(state);
    });
}

function pointToCell(grid, e) {
    const rect = grid.getBoundingClientRect();
    const cellW = rect.width / COLS;
    const col = Math.floor((e.clientX - rect.left) / cellW);
    const row = Math.floor((e.clientY - rect.top) / ROW_HEIGHT);
    return { col, row };
}

function clamp(v, lo, hi) { return Math.max(lo, Math.min(hi, v)); }

function renderGrid(state) {
    clearTimers();
    const grid = state.mount.querySelector('#b-grid');
    if (!grid) return;
    grid.innerHTML = '';
    for (const w of (state.board.layout || [])) {
        const el = document.createElement('div');
        el.style.gridColumn = `${w.x + 1} / span ${w.w}`;
        el.style.gridRow = `${w.y + 1} / span ${w.h}`;
        el.style.background = 'var(--bg-card)';
        el.style.border = '1px solid var(--border)';
        el.style.borderLeft = '2px solid var(--cyan)';
        el.style.display = 'flex';
        el.style.flexDirection = 'column';
        el.style.overflow = 'hidden';
        el.draggable = true;
        el.addEventListener('dragstart', (e) => {
            state.drag = { id: w.id, source: 'widget' };
            e.dataTransfer.effectAllowed = 'move';
        });
        const head = document.createElement('div');
        head.style.cssText = 'display:flex;align-items:center;justify-content:space-between;padding:4px 8px;background:var(--bg-secondary);font-size:10px;color:var(--cyan);text-transform:uppercase;letter-spacing:1px;cursor:grab;';
        head.innerHTML = `<span>${esc(widgetLabel(w.kind))}</span>
            <span>
                <button class="btn" data-act="cfg" data-i18n-aria-label="common.aria.settings" aria-label="Settings" style="font-size:9px;padding:1px 4px;">⚙</button>
                <button class="btn" data-act="del" data-i18n-aria-label="common.aria.remove" aria-label="Remove" style="font-size:9px;padding:1px 4px;color:var(--red);border-color:var(--red);">×</button>
            </span>`;
        el.appendChild(head);
        const body = document.createElement('div');
        body.style.cssText = 'flex:1 1 auto;overflow:auto;padding:6px 8px;';
        el.appendChild(body);

        head.querySelector('[data-act=del]').addEventListener('click', () => {
            state.board.layout = state.board.layout.filter(x => x.id !== w.id);
            persist(state); renderGrid(state);
        });
        head.querySelector('[data-act=cfg]').addEventListener('click', () => {
            const cfg = configureWidget(w);
            if (cfg) { Object.assign(w.params, cfg); persist(state); renderGrid(state); }
        });

        grid.appendChild(el);
        mountWidget(body, w, state.tok);
    }
}

function configureWidget(w) {
    if (w.kind === 'quote' || w.kind === 'mini_chart') {
        const s = prompt(t('view.boards.prompt.symbol'), w.params.symbol || 'SPY');
        return s ? { symbol: s.toUpperCase() } : null;
    }
    if (w.kind === 'note') {
        const txt = prompt(t('view.boards.prompt.note_text'), w.params.text || '');
        return txt === null ? null : { text: txt };
    }
    if (w.kind === 'watchlist' || w.kind === 'alerts') {
        const n = prompt(t('view.boards.prompt.limit'), w.params.limit || 10);
        return n ? { limit: Number(n) || 10 } : null;
    }
    return null;
}

async function persist(state) {
    state.dirty = true;
    const tag = state.mount.querySelector('#b-save');
    if (tag) tag.textContent = t('common.status.saving');
    try {
        await api.updateDashboard(state.board.id, {
            name: state.board.name, layout: state.board.layout || [],
        });
        if (!viewIsCurrent(state.tok)) return;
        const tag2 = state.mount.querySelector('#b-save');
        if (tag2) tag2.textContent = t('view.boards.status.saved', { when: new Date().toLocaleTimeString(undefined, { hour12: false }) });
        state.dirty = false;
    } catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        const tag2 = state.mount.querySelector('#b-save');
        if (tag2) tag2.textContent = t('common.error.save_failed', { err: e.message });
    }
}

// ---------------------------------------------------------------------------
// Widget mount helpers — each calls api.* and renders into the body div.
// All auto-refresh via refreshTimers.
// ---------------------------------------------------------------------------

function clearTimers() {
    for (const t of refreshTimers) clearInterval(t);
    refreshTimers = [];
}

function tickEvery(secs, fn, tok) {
    const wrapped = async () => {
        if (tok != null && !viewIsCurrent(tok)) return;
        await fn();
    };
    wrapped();
    const handle = setInterval(() => {
        if (tok != null && !viewIsCurrent(tok)) { clearInterval(handle); return; }
        wrapped();
    }, secs * 1000);
    refreshTimers.push(handle);
}

function mountWidget(body, w, tok) {
    body.innerHTML = `<div class="muted small">${esc(t('common.loading'))}</div>`;
    switch (w.kind) {
        case 'quote':       return mountQuote(body, w, tok);
        case 'mini_chart':  return mountMiniChart(body, w, tok);
        case 'breadth':     return mountBreadth(body, tok);
        case 'vix':         return mountVix(body, tok);
        case 'watchlist':   return mountWatchlist(body, w, tok);
        case 'alerts':      return mountAlerts(body, w, tok);
        case 'fear_greed':  return mountFearGreed(body, tok);
        case 'news':        return mountNews(body, w, tok);
        case 'note':        return mountNote(body, w);
        default:            body.innerHTML = `<p class="muted small">unknown widget: ${esc(w.kind)}</p>`;
    }
}

async function mountQuote(body, w, tok) {
    tickEvery(60, async () => {
        try {
            const q = await api.quote(w.params.symbol);
            if (!viewIsCurrent(tok)) return;
            const ch = q.change_pct;
            const cls = ch == null ? '' : ch >= 0 ? 'pos' : 'neg';
            body.innerHTML = `
                <div class="muted small">${esc(q.symbol)}</div>
                <div style="font-size:24px;font-weight:700;">${fmt(q.price, q.price < 10 ? 4 : 2)}</div>
                <div class="small ${cls}">${ch == null ? '—' : (ch >= 0 ? '+' : '') + ch.toFixed(2) + '%'}</div>
            `;
        } catch (e) { if (viewIsCurrent(tok)) body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    }, tok);
}

async function mountMiniChart(body, w, tok) {
    tickEvery(120, async () => {
        try {
            const to = Math.floor(Date.now() / 1000);
            const from = to - 30 * 86400;
            const resp = await api.bars(w.params.symbol, '1d', from, to);
            if (!viewIsCurrent(tok)) return;
            const bars = resp.bars || [];
            if (!bars.length) { body.innerHTML = '<p data-i18n="view.boards.hint.no_bars" class="muted small">no bars</p>'; return; }
            const closes = bars.map(b => Number(b.close));
            const lo = Math.min(...closes), hi = Math.max(...closes);
            const last = closes[closes.length - 1], first = closes[0];
            const change = ((last - first) / first) * 100;
            const W = body.clientWidth || 200, H = 60;
            const sx = i => (i / (closes.length - 1)) * W;
            const sy = v => H - ((v - lo) / Math.max(hi - lo, 1e-9)) * H;
            const path = closes.map((v, i) => (i ? 'L' : 'M') + sx(i) + ',' + sy(v)).join(' ');
            const color = change >= 0 ? '#7af0a8' : '#ff1f7a';
            body.innerHTML = `
                <div class="small muted">${esc(w.params.symbol)} · 30d ${change >= 0 ? '+' : ''}${change.toFixed(2)}%</div>
                <svg viewBox="0 0 ${W} ${H}" width="100%" height="${H}" style="display:block;">
                    <path d="${path}" stroke="${color}" stroke-width="1.5" fill="none"/>
                </svg>
                <div class="small">${fmt(last, last < 10 ? 4 : 2)}</div>
            `;
        } catch (e) { if (viewIsCurrent(tok)) body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    }, tok);
}

async function mountBreadth(body, tok) {
    tickEvery(60, async () => {
        try {
            const s = await api.breadthSnapshot();
            if (!viewIsCurrent(tok)) return;
            const cls = s.composite_score >= 30 ? 'pos' : s.composite_score <= -30 ? 'neg' : '';
            body.innerHTML = `
                <div class="muted small" data-i18n="view.boards.tile.composite">Composite</div>
                <div style="font-size:22px;font-weight:700;" class="${cls}">${s.composite_score >= 0 ? '+' : ''}${s.composite_score}</div>
                <div class="small ${cls}">${(s.regime || '').toUpperCase()}</div>
            `;
        } catch (e) { if (viewIsCurrent(tok)) body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    }, tok);
}

async function mountVix(body, tok) {
    tickEvery(60, async () => {
        try {
            const q = await api.quote('^VIX');
            if (!viewIsCurrent(tok)) return;
            const ch = q.change_pct;
            const cls = ch == null ? '' : ch >= 0 ? 'neg' : 'pos'; // VIX up = risk-off
            body.innerHTML = `
                <div class="muted small" data-i18n="view.boards.tile.vix">VIX</div>
                <div style="font-size:22px;font-weight:700;">${fmt(q.price, 2)}</div>
                <div class="small ${cls}">${ch == null ? '—' : (ch >= 0 ? '+' : '') + ch.toFixed(2) + '%'}</div>
            `;
        } catch (e) { if (viewIsCurrent(tok)) body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    }, tok);
}

async function mountFearGreed(body, tok) {
    tickEvery(120, async () => {
        try {
            const r = await api.fearGreed();
            if (!viewIsCurrent(tok)) return;
            const c = r.score <= 24 ? '#ff1f7a' :
                      r.score <= 44 ? '#ff7a1f' :
                      r.score <= 55 ? '#9aa0c8' :
                      r.score <= 74 ? '#7af0a8' : '#00ffaa';
            body.innerHTML = `
                <div class="muted small" data-i18n="view.boards.tile.fear_greed">Fear & Greed</div>
                <div style="font-size:22px;font-weight:700;color:${c};">${r.score}</div>
                <div class="small" style="color:${c};">${esc(r.label)}</div>
            `;
        } catch (e) { if (viewIsCurrent(tok)) body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    }, tok);
}

async function mountWatchlist(body, w, tok) {
    tickEvery(90, async () => {
        try {
            const wls = await api.watchlists();
            if (!viewIsCurrent(tok)) return;
            const def = wls.find(x => x.is_default) || wls[0];
            if (!def) { body.innerHTML = '<p data-i18n="view.boards.hint.no_watchlist" class="muted small">no watchlist</p>'; return; }
            const syms = await api.watchlistSymbols(def.id);
            if (!viewIsCurrent(tok)) return;
            const rows = (syms || []).slice(0, w.params.limit || 10)
                                     .map(r => typeof r === 'string' ? r : r.symbol);
            const quotes = await Promise.all(rows.map(s => api.quote(s).catch(() => null)));
            if (!viewIsCurrent(tok)) return;
            body.innerHTML = `
                <table class="trades" style="font-size:11px;">
                    <thead><tr><th data-i18n="view.boards.th.sym">Sym</th><th data-i18n="view.boards.th.last">Last</th><th>%</th></tr></thead>
                    <tbody>
                    ${rows.map((s, i) => {
                        const q = quotes[i];
                        if (!q) return `<tr><td>${esc(s)}</td><td>—</td><td>—</td></tr>`;
                        const ch = q.change_pct;
                        const cls = ch == null ? '' : ch >= 0 ? 'pos' : 'neg';
                        return `<tr>
                            <td>${esc(s)}</td>
                            <td>${fmt(q.price, q.price < 10 ? 4 : 2)}</td>
                            <td class="${cls}">${ch == null ? '—' : (ch >= 0 ? '+' : '') + ch.toFixed(2) + '%'}</td>
                        </tr>`;
                    }).join('')}
                    </tbody>
                </table>
            `;
        } catch (e) { if (viewIsCurrent(tok)) body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    }, tok);
}

async function mountAlerts(body, w, tok) {
    tickEvery(60, async () => {
        try {
            const rules = await api.alerts().catch(() => []);
            if (!viewIsCurrent(tok)) return;
            const rows = rules.slice(0, w.params.limit || 10);
            if (!rows.length) { body.innerHTML = '<p data-i18n="view.boards.hint.no_alert_rules" class="muted small">no alert rules</p>'; return; }
            body.innerHTML = `<table class="trades" style="font-size:11px;">
                <thead><tr><th data-i18n="view.boards.th.name_2">Name</th><th data-i18n="view.boards.th.symbol">Symbol</th><th data-i18n="view.boards.th.status">Status</th><th data-i18n="view.boards.th.fired">Fired</th></tr></thead>
                <tbody>
                ${rows.map(r => `<tr>
                    <td>${esc(r.name || '')}</td>
                    <td>${esc(r.symbol || '')}</td>
                    <td class="small ${r.enabled === false ? 'muted' : 'pos'}">${t(r.enabled === false ? 'common.off_lc' : 'common.on_lc')}</td>
                    <td>${r.fire_count ?? 0}</td>
                </tr>`).join('')}
                </tbody></table>`;
        } catch (e) { if (viewIsCurrent(tok)) body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    }, tok);
}

async function mountNews(body, w, tok) {
    tickEvery(120, async () => {
        try {
            const items = await api.newsBySymbol(w.params.symbol, w.params.limit || 6);
            if (!viewIsCurrent(tok)) return;
            if (!items.length) {
                body.innerHTML = `<p class="muted small">${esc(t('view.boards.hint.no_news', { symbol: w.params.symbol }))}</p>`;
                return;
            }
            body.innerHTML = items.map(n => {
                const s = n.sentiment;
                const color = s == null ? '#444' :
                    s > 0.1 ? '#7af0a8' : s < -0.1 ? '#ff1f7a' : '#9aa0c8';
                const link = n.link
                    ? `<a href="${esc(n.link)}" target="_blank" rel="noopener" style="color:var(--text);">${esc(n.title)}</a>`
                    : esc(n.title);
                const when = n.published_at || n.fetched_at;
                return `<div style="display:flex;border-left:2px solid ${color};padding:3px 6px;margin-bottom:3px;font-size:11px;">
                    <div style="flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">${link}</div>
                    <span class="muted small" style="margin-left:6px;white-space:nowrap;">${esc(relTime(when))}</span>
                </div>`;
            }).join('');
        } catch (e) { if (viewIsCurrent(tok)) body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    }, tok);
}

function relTime(iso) {
    if (!iso) return '';
    const diff = (Date.now() - new Date(iso).getTime()) / 1000;
    if (diff < 60) return `${Math.floor(diff)}s`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
    return `${Math.floor(diff / 86400)}d`;
}

function mountNote(body, w) {
    body.innerHTML = `<div style="white-space:pre-wrap;font-family:'Share Tech Mono',monospace;font-size:12px;">${esc(w.params.text || t('view.boards.note.empty'))}</div>`;
}
