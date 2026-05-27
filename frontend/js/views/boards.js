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

const WIDGET_KINDS = [
    { kind: 'quote',      label: 'Quote',        defaults: { symbol: 'SPY' },     w: 3, h: 2 },
    { kind: 'mini_chart', label: 'Mini chart',   defaults: { symbol: 'SPY' },     w: 6, h: 3 },
    { kind: 'breadth',    label: 'Breadth pulse',defaults: {},                    w: 4, h: 2 },
    { kind: 'vix',        label: 'VIX',          defaults: {},                    w: 3, h: 2 },
    { kind: 'watchlist',  label: 'Watchlist',    defaults: { limit: 10 },         w: 4, h: 4 },
    { kind: 'alerts',     label: 'Alerts feed',  defaults: { limit: 10 },         w: 6, h: 4 },
    { kind: 'fear_greed', label: 'Fear & Greed', defaults: {},                    w: 3, h: 2 },
    { kind: 'note',       label: 'Sticky note',  defaults: { text: '' },          w: 3, h: 2 },
];

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
    const boards = await api.listDashboards().catch(() => []);
    mount.innerHTML = `
        <h1 class="view-title">// BOARDS</h1>
        <p class="muted small">Custom dashboards. Open each board in its own browser window
            and snap to a monitor for a multi-screen trading rig. Drag widgets from the
            palette to add; click × to remove; drag widget headers to reposition.</p>

        <div class="chart-panel">
            <form id="b-new" class="inline-form">
                <input name="name" placeholder="board name" required style="min-width:240px;">
                <button class="primary" type="submit">Create board</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2>Your boards (${boards.length})</h2>
            ${boards.length === 0
                ? '<p class="muted small">No boards yet.</p>'
                : `<table class="trades">
                    <thead><tr><th>Name</th><th>Widgets</th><th>Updated</th><th></th></tr></thead>
                    <tbody>
                        ${boards.map(b => `<tr>
                            <td><a href="#boards/${b.id}">${esc(b.name)}</a></td>
                            <td>${Array.isArray(b.layout) ? b.layout.length : 0}</td>
                            <td class="small">${new Date(b.updated_at).toLocaleString()}</td>
                            <td><button class="btn b-del" data-id="${b.id}">Delete</button></td>
                        </tr>`).join('')}
                    </tbody>
                </table>`}
        </div>
    `;
    document.getElementById('b-new').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        try {
            const b = await api.createDashboard({ name: fd.get('name').trim(), layout: [] });
            window.location.hash = `boards/${b.id}`;
        } catch (err) { alert(err.message); }
    });
    mount.querySelectorAll('.b-del').forEach(btn => {
        btn.addEventListener('click', async () => {
            if (!confirm('Delete this board?')) return;
            try { await api.deleteDashboard(btn.dataset.id); renderList(mount); }
            catch (e) { alert(e.message); }
        });
    });
}

// ---------------------------------------------------------------------------
// Board view
// ---------------------------------------------------------------------------

async function renderBoard(mount, id) {
    let board;
    try { board = await api.getDashboard(id); }
    catch (e) { mount.innerHTML = `<p class="boot">${esc(e.message)}</p>`; return; }

    const state = { board, dirty: false, drag: null };

    mount.innerHTML = `
        <h1 class="view-title">// BOARD — <span id="b-name">${esc(board.name)}</span>
            <button id="b-rename" class="btn" style="font-size:10px;">rename</button>
            <span id="b-save" class="muted small" style="margin-left:8px;">saved</span>
        </h1>

        <div class="chart-panel">
            <h2>Palette — drag onto grid</h2>
            <div id="palette" style="display:flex;flex-wrap:wrap;gap:6px;">
                ${WIDGET_KINDS.map(k => `
                    <div class="palette-tile" draggable="true" data-kind="${k.kind}"
                         style="border:1px solid var(--cyan);padding:6px 10px;cursor:grab;background:var(--bg-secondary);">
                        ${esc(k.label)}
                    </div>
                `).join('')}
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

    document.getElementById('b-rename').addEventListener('click', async () => {
        const next = prompt('Board name:', state.board.name);
        if (!next || next === state.board.name) return;
        state.board.name = next;
        document.getElementById('b-name').textContent = next;
        await persist(state);
    });

    bindPaletteDrag(state);
    renderGrid(state);
}

// ---------------------------------------------------------------------------
// Grid render + drag
// ---------------------------------------------------------------------------

function bindPaletteDrag(state) {
    document.querySelectorAll('.palette-tile').forEach(el => {
        el.addEventListener('dragstart', (e) => {
            state.drag = { kind: el.dataset.kind, source: 'palette' };
            e.dataTransfer.effectAllowed = 'copy';
        });
    });
    const grid = document.getElementById('b-grid');
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
    const grid = document.getElementById('b-grid');
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
        head.innerHTML = `<span>${esc(WIDGET_KINDS.find(k => k.kind === w.kind)?.label || w.kind)}</span>
            <span>
                <button class="btn" data-act="cfg" style="font-size:9px;padding:1px 4px;">⚙</button>
                <button class="btn" data-act="del" style="font-size:9px;padding:1px 4px;color:var(--red);border-color:var(--red);">×</button>
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
        mountWidget(body, w);
    }
}

function configureWidget(w) {
    if (w.kind === 'quote' || w.kind === 'mini_chart') {
        const s = prompt('Symbol:', w.params.symbol || 'SPY');
        return s ? { symbol: s.toUpperCase() } : null;
    }
    if (w.kind === 'note') {
        const t = prompt('Note text:', w.params.text || '');
        return t === null ? null : { text: t };
    }
    if (w.kind === 'watchlist' || w.kind === 'alerts') {
        const n = prompt('Limit:', w.params.limit || 10);
        return n ? { limit: Number(n) || 10 } : null;
    }
    return null;
}

async function persist(state) {
    state.dirty = true;
    const tag = document.getElementById('b-save');
    if (tag) tag.textContent = 'saving…';
    try {
        await api.updateDashboard(state.board.id, {
            name: state.board.name, layout: state.board.layout || [],
        });
        if (tag) tag.textContent = `saved ${new Date().toLocaleTimeString(undefined, { hour12: false })}`;
        state.dirty = false;
    } catch (e) {
        if (tag) tag.textContent = 'save failed: ' + e.message;
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

function tickEvery(secs, fn) {
    fn();
    refreshTimers.push(setInterval(fn, secs * 1000));
}

function mountWidget(body, w) {
    body.innerHTML = '<div class="muted small">loading…</div>';
    switch (w.kind) {
        case 'quote':       return mountQuote(body, w);
        case 'mini_chart':  return mountMiniChart(body, w);
        case 'breadth':     return mountBreadth(body);
        case 'vix':         return mountVix(body);
        case 'watchlist':   return mountWatchlist(body, w);
        case 'alerts':      return mountAlerts(body, w);
        case 'fear_greed':  return mountFearGreed(body);
        case 'note':        return mountNote(body, w);
        default:            body.innerHTML = `<p class="muted small">unknown widget: ${esc(w.kind)}</p>`;
    }
}

async function mountQuote(body, w) {
    tickEvery(60, async () => {
        try {
            const q = await api.quote(w.params.symbol);
            const ch = q.change_pct;
            const cls = ch == null ? '' : ch >= 0 ? 'pos' : 'neg';
            body.innerHTML = `
                <div class="muted small">${esc(q.symbol)}</div>
                <div style="font-size:24px;font-weight:700;">${fmt(q.price, q.price < 10 ? 4 : 2)}</div>
                <div class="small ${cls}">${ch == null ? '—' : (ch >= 0 ? '+' : '') + ch.toFixed(2) + '%'}</div>
            `;
        } catch (e) { body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    });
}

async function mountMiniChart(body, w) {
    tickEvery(120, async () => {
        try {
            const to = Math.floor(Date.now() / 1000);
            const from = to - 30 * 86400;
            const resp = await api.bars(w.params.symbol, '1d', from, to);
            const bars = resp.bars || [];
            if (!bars.length) { body.innerHTML = '<p class="muted small">no bars</p>'; return; }
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
        } catch (e) { body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    });
}

async function mountBreadth(body) {
    tickEvery(60, async () => {
        try {
            const s = await api.breadthSnapshot();
            const cls = s.composite_score >= 30 ? 'pos' : s.composite_score <= -30 ? 'neg' : '';
            body.innerHTML = `
                <div class="muted small">Composite</div>
                <div style="font-size:22px;font-weight:700;" class="${cls}">${s.composite_score >= 0 ? '+' : ''}${s.composite_score}</div>
                <div class="small ${cls}">${(s.regime || '').toUpperCase()}</div>
            `;
        } catch (e) { body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    });
}

async function mountVix(body) {
    tickEvery(60, async () => {
        try {
            const q = await api.quote('^VIX');
            const ch = q.change_pct;
            const cls = ch == null ? '' : ch >= 0 ? 'neg' : 'pos'; // VIX up = risk-off
            body.innerHTML = `
                <div class="muted small">VIX</div>
                <div style="font-size:22px;font-weight:700;">${fmt(q.price, 2)}</div>
                <div class="small ${cls}">${ch == null ? '—' : (ch >= 0 ? '+' : '') + ch.toFixed(2) + '%'}</div>
            `;
        } catch (e) { body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    });
}

async function mountFearGreed(body) {
    tickEvery(120, async () => {
        try {
            const r = await api.fearGreed();
            const c = r.score <= 24 ? '#ff1f7a' :
                      r.score <= 44 ? '#ff7a1f' :
                      r.score <= 55 ? '#9aa0c8' :
                      r.score <= 74 ? '#7af0a8' : '#00ffaa';
            body.innerHTML = `
                <div class="muted small">Fear & Greed</div>
                <div style="font-size:22px;font-weight:700;color:${c};">${r.score}</div>
                <div class="small" style="color:${c};">${esc(r.label)}</div>
            `;
        } catch (e) { body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    });
}

async function mountWatchlist(body, w) {
    tickEvery(90, async () => {
        try {
            const wls = await api.watchlists();
            const def = wls.find(x => x.is_default) || wls[0];
            if (!def) { body.innerHTML = '<p class="muted small">no watchlist</p>'; return; }
            const syms = await api.watchlistSymbols(def.id);
            const rows = (syms || []).slice(0, w.params.limit || 10)
                                     .map(r => typeof r === 'string' ? r : r.symbol);
            const quotes = await Promise.all(rows.map(s => api.quote(s).catch(() => null)));
            body.innerHTML = `
                <table class="trades" style="font-size:11px;">
                    <thead><tr><th>Sym</th><th>Last</th><th>%</th></tr></thead>
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
        } catch (e) { body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    });
}

async function mountAlerts(body, w) {
    tickEvery(60, async () => {
        try {
            const rules = await api.alerts().catch(() => []);
            const rows = rules.slice(0, w.params.limit || 10);
            if (!rows.length) { body.innerHTML = '<p class="muted small">no alert rules</p>'; return; }
            body.innerHTML = `<table class="trades" style="font-size:11px;">
                <thead><tr><th>Name</th><th>Symbol</th><th>Status</th><th>Fired</th></tr></thead>
                <tbody>
                ${rows.map(r => `<tr>
                    <td>${esc(r.name || '')}</td>
                    <td>${esc(r.symbol || '')}</td>
                    <td class="small ${r.enabled === false ? 'muted' : 'pos'}">${r.enabled === false ? 'off' : 'on'}</td>
                    <td>${r.fire_count ?? 0}</td>
                </tr>`).join('')}
                </tbody></table>`;
        } catch (e) { body.innerHTML = `<p class="muted small">${esc(e.message)}</p>`; }
    });
}

function mountNote(body, w) {
    body.innerHTML = `<div style="white-space:pre-wrap;font-family:'Share Tech Mono',monospace;font-size:12px;">${esc(w.params.text || '(empty — click ⚙ to edit)')}</div>`;
}
