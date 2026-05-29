// Per-trade order-flow tape replay.
//
// Routes:
//   #tape-replay         → trade picker (list of recent closed trades)
//   #tape-replay/<id>    → animated replay for one trade
//
// Animation: progressive bar reveal + exec marker fade-in. Cursor walks
// the bar array at a speed derived from real bar duration ÷ playback rate.
// User can pause / scrub via slider / drop "what I should've done" notes
// at any cursor point (kept client-side per replay session).

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

const SPEEDS = [1, 10, 100, 500];
let raf = null;
let timerHandle = null;

export async function renderTapeReplay(mount, state, tradeId = '') {
    cancelAnims();
    const tok = currentViewToken();
    if (!tradeId) return renderPicker(mount, state, tok);
    return renderReplay(mount, tradeId, tok);
}

async function renderPicker(mount, state, tok) {
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) { mount.innerHTML = `<p data-i18n="view.tape_replay.hint.no_account_selected" class="boot">No account selected.</p>`; return; }
    mount.innerHTML = `
        <h1 data-i18n="view.tape_replay.h1.tape_replay" class="view-title">// TAPE REPLAY</h1>
        <p data-i18n="view.tape_replay.hint.re_watch_the_exact_bar_sequence_around_any_of_your" class="muted small">Re-watch the exact bar sequence around any of your closed trades
            with execution markers fading in at their fill timestamps. Auto-picks bar interval
            from hold duration (1m/5m/1h/1d). Use the replay to ask "what would I do differently"
            and drop client-side journal notes at the cursor.</p>
        <div id="tr-list" class="chart-panel"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="view.tape_replay.loading_list">loading recent closed trades…</div></div></div>
    `;
    try {
        const trades = await api.trades(acct.id, { status: 'closed', limit: 60 });
        if (!viewIsCurrent(tok)) return;
        const rows = trades.map(t => `<tr>
            <td><a href="#tape-replay/${t.id}">${esc(t.symbol)}</a></td>
            <td>${esc(t.side)}</td>
            <td>${t.qty}</td>
            <td class="small">${new Date(t.opened_at).toLocaleString()}</td>
            <td class="small">${t.closed_at ? new Date(t.closed_at).toLocaleString() : '—'}</td>
            <td class="${(t.net_pnl ?? 0) >= 0 ? 'pos' : 'neg'}">${
                t.net_pnl != null ? '$' + fmt(Number(t.net_pnl)) : '—'
            }</td>
        </tr>`).join('');
        const listEl = mount.querySelector('#tr-list');
        if (!listEl) return;
        listEl.innerHTML = trades.length === 0
            ? '<p data-i18n="view.tape_replay.hint.no_closed_trades_on_this_account" class="muted small">No closed trades on this account.</p>'
            : `<table class="trades">
                <thead><tr><th data-i18n="view.tape_replay.th.symbol">Symbol</th><th data-i18n="view.tape_replay.th.side">Side</th><th data-i18n="view.tape_replay.th.qty">Qty</th><th data-i18n="view.tape_replay.th.opened">Opened</th><th data-i18n="view.tape_replay.th.closed">Closed</th><th data-i18n="view.tape_replay.th.net_p_l">Net P/L</th></tr></thead>
                <tbody>${rows}</tbody>
            </table>`;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const listEl = mount.querySelector('#tr-list');
        if (listEl) listEl.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

async function renderReplay(mount, tradeId, tok) {
    mount.innerHTML = `<h1 data-i18n="view.tape_replay.h1.tape_replay_2" class="view-title">// TAPE REPLAY</h1>
        <div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="view.tape_replay.loading_replay">loading replay…</div></div>`;
    let data;
    try { data = await api.tapeReplay(tradeId); }
    catch (e) {
        if (!viewIsCurrent(tok)) return;
        mount.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const sessionNotes = [];      // { time_iso, cursor_idx, text }
    const state = { idx: data.bars.length ? 0 : -1, playing: false, speed: 10 };

    mount.innerHTML = `
        <h1 class="view-title">// REPLAY — ${esc(data.symbol)} · ${esc(data.side)} ·
            ${esc(data.interval)} bars</h1>
        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.tape_replay.card.qty">Qty</div><div class="value">${fmt(data.qty, 0)}</div></div>
            <div class="card"><div class="label" data-i18n="view.tape_replay.card.entry_exit">Entry / Exit</div>
                <div class="value">${fmt(data.entry_avg, 2)} → ${data.exit_avg != null ? fmt(data.exit_avg, 2) : '—'}</div></div>
            <div class="card"><div class="label" data-i18n="view.tape_replay.card.stop_target">Stop / Target</div>
                <div class="value">${data.stop_loss != null ? fmt(data.stop_loss, 2) : '—'} /
                ${data.initial_target != null ? fmt(data.initial_target, 2) : '—'}</div></div>
            <div class="card"><div class="label" data-i18n="view.tape_replay.card.net_pnl">Net P/L</div>
                <div class="value ${(data.net_pnl ?? 0) >= 0 ? 'pos' : 'neg'}">${
                    data.net_pnl != null ? '$' + fmt(data.net_pnl) : '—'
                }</div></div>
        </div>

        <div class="chart-panel">
            <div class="inline-form" style="margin-bottom:8px;">
                <button data-i18n="view.tape_replay.btn.play" class="btn" id="tr-play">▶ Play</button>
                <button data-i18n="view.tape_replay.btn.rewind" class="btn" id="tr-rewind">⏮ Rewind</button>
                <label><span data-i18n="view.tape_replay.label.speed">Speed</span>
                    <select id="tr-speed">
                        ${SPEEDS.map(s => `<option value="${s}" ${s === state.speed ? 'selected' : ''}>${s}×</option>`).join('')}
                    </select>
                </label>
                <input id="tr-scrub" type="range" min="0" max="${Math.max(data.bars.length - 1, 0)}" value="0" style="flex:1;">
                <span id="tr-pos" class="muted small"></span>
            </div>
            <div id="tr-chart"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.tape_replay.h2.what_i_d_do_differently_client_side_this_session_o">What I'd do differently (client-side, this session only)</h2>
            <form id="tr-note-form" class="inline-form">
                <input id="tr-note" data-i18n-placeholder="view.tape_replay.note.placeholder" placeholder="Drop note at current cursor" style="flex:1;min-width:240px;">
                <button data-i18n="view.tape_replay.btn.pin_at_cursor" class="primary" type="submit">Pin at cursor</button>
            </form>
            <div id="tr-notes" style="margin-top:8px;"></div>
        </div>
    `;

    const $ = (id) => mount.querySelector('#' + id);
    $('tr-play').addEventListener('click', () => {
        state.playing = !state.playing;
        $('tr-play').textContent = t(state.playing ? 'view.tape_replay.btn.pause' : 'view.tape_replay.btn.play');
        if (state.playing) play(data, state, mount, tok);
        else cancelAnims();
    });
    $('tr-rewind').addEventListener('click', () => {
        cancelAnims();
        state.playing = false;
        $('tr-play').textContent = t('view.tape_replay.btn.play');
        state.idx = 0;
        renderChart(data, state, mount);
    });
    $('tr-speed').addEventListener('change', (e) => {
        state.speed = Number(e.target.value);
        if (state.playing) { cancelAnims(); play(data, state, mount, tok); }
    });
    $('tr-scrub').addEventListener('input', (e) => {
        cancelAnims();
        state.playing = false;
        $('tr-play').textContent = t('view.tape_replay.btn.play');
        state.idx = Number(e.target.value);
        renderChart(data, state, mount);
    });
    $('tr-note-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const text = $('tr-note').value.trim();
        if (!text) return;
        const bar = data.bars[state.idx];
        sessionNotes.push({ time_iso: bar?.time, cursor_idx: state.idx, text });
        $('tr-note').value = '';
        renderNotes(sessionNotes, mount);
    });

    renderChart(data, state, mount);
    renderNotes(sessionNotes, mount);
}

function play(data, state, mount, tok) {
    if (state.idx >= data.bars.length - 1) state.idx = 0;
    // realTickMs = bar_duration / speed_multiplier. speed=1 plays in real
    // time (1m bar takes 60s); speed=500 collapses a 1m bar into 120ms.
    const intervalMs = { '1m': 60_000, '5m': 300_000, '1h': 3_600_000, '1d': 86_400_000 }[data.interval] || 60_000;
    const realTickMs = Math.max(20, intervalMs / state.speed);
    const tick = () => {
        if (!viewIsCurrent(tok)) { cancelAnims(); state.playing = false; return; }
        if (!state.playing) return;
        state.idx++;
        renderChart(data, state, mount);
        const slider = mount.querySelector('#tr-scrub');
        if (slider) slider.value = String(state.idx);
        if (state.idx >= data.bars.length - 1) {
            state.playing = false;
            const btn = mount.querySelector('#tr-play');
            if (btn) btn.textContent = t('view.tape_replay.btn.play');
            return;
        }
        timerHandle = setTimeout(tick, realTickMs);
    };
    tick();
}

function cancelAnims() {
    if (raf) { cancelAnimationFrame(raf); raf = null; }
    if (timerHandle) { clearTimeout(timerHandle); timerHandle = null; }
}

function renderChart(data, state, mount) {
    const el = mount.querySelector('#tr-chart');
    if (!el) return;
    const bars = data.bars;
    if (!bars.length) { el.innerHTML = '<p data-i18n="view.tape_replay.hint.no_bars_cached_for_this_trade_window" class="muted small">No bars cached for this trade window.</p>'; return; }
    const visible = bars.slice(0, state.idx + 1);
    const W = 980, H = 360, padL = 50, padR = 10, padT = 10, padB = 28;
    const innerW = W - padL - padR, innerH = H - padT - padB;
    // Y range from full bar set so the y-axis doesn't jump while playing.
    const allLows  = bars.map(b => b.low),  allHighs = bars.map(b => b.high);
    let yMin = Math.min(...allLows), yMax = Math.max(...allHighs);
    [data.entry_avg, data.exit_avg, data.stop_loss, data.initial_target].forEach(v => {
        if (v != null && Number.isFinite(v)) {
            yMin = Math.min(yMin, v); yMax = Math.max(yMax, v);
        }
    });
    const yPad = (yMax - yMin) * 0.05 || 1;
    yMin -= yPad; yMax += yPad;
    const sx = i => padL + (i / Math.max(bars.length - 1, 1)) * innerW;
    const sy = v => padT + (1 - (v - yMin) / Math.max(yMax - yMin, 1e-9)) * innerH;
    const cellW = Math.max(1.5, innerW / bars.length * 0.6);

    const candles = visible.map((b, i) => {
        const x = sx(i);
        const yO = sy(b.open), yC = sy(b.close);
        const yH = sy(b.high), yL = sy(b.low);
        const up = b.close >= b.open;
        const color = up ? '#7af0a8' : '#ff1f7a';
        const top = Math.min(yO, yC), h = Math.max(1, Math.abs(yC - yO));
        return `<line x1="${x}" y1="${yH}" x2="${x}" y2="${yL}" stroke="${color}" stroke-width="1"/>
                <rect x="${x - cellW / 2}" y="${top}" width="${cellW}" height="${h}" fill="${color}"/>`;
    }).join('');

    // Reference lines
    const ref = (price, color, label) => {
        if (price == null || !Number.isFinite(price)) return '';
        const y = sy(price);
        return `<line x1="${padL}" y1="${y}" x2="${W - padR}" y2="${y}"
                stroke="${color}" stroke-dasharray="4,3" stroke-width="1"/>
                <text x="${W - padR - 4}" y="${y - 3}" text-anchor="end" fill="${color}" font-size="10">${esc(label)} ${price.toFixed(2)}</text>`;
    };

    // Exec markers — only those whose time ≤ cursor time
    const cursorTime = bars[state.idx]?.time;
    const xForTime = (iso) => {
        const t = new Date(iso).getTime();
        const t0 = new Date(bars[0].time).getTime();
        const tN = new Date(bars[bars.length - 1].time).getTime();
        if (tN <= t0) return padL;
        return padL + Math.max(0, Math.min(1, (t - t0) / (tN - t0))) * innerW;
    };
    const execMarkers = data.execs
        .filter(e => !cursorTime || new Date(e.time) <= new Date(cursorTime))
        .map(e => {
            const x = xForTime(e.time);
            const y = sy(e.price);
            const up = e.side === 'buy' || e.side === 'cover';
            const color = up ? '#00ffaa' : '#ff1f7a';
            const tri = up
                ? `${x},${y + 4} ${x - 6},${y + 14} ${x + 6},${y + 14}`
                : `${x},${y - 4} ${x - 6},${y - 14} ${x + 6},${y - 14}`;
            return `<polygon points="${tri}" fill="${color}" opacity="0.9"/>
                    <text x="${x + 8}" y="${y + (up ? 14 : -10)}" fill="${color}" font-size="9">
                        ${esc(e.side.toUpperCase())} ${e.qty} @ ${e.price.toFixed(2)}
                    </text>`;
        }).join('');

    // Cursor line
    const cx = sx(state.idx);
    const yAxis = `${[0, 0.25, 0.5, 0.75, 1].map(t => {
        const y = padT + t * innerH;
        const v = yMax - t * (yMax - yMin);
        return `<text x="${padL - 4}" y="${y + 3}" text-anchor="end" fill="#9aa0c8" font-size="10">${v.toFixed(2)}</text>`;
    }).join('')}`;

    el.innerHTML = `<svg viewBox="0 0 ${W} ${H}" width="100%" style="display:block;">
        <rect x="${padL}" y="${padT}" width="${innerW}" height="${innerH}"
              fill="#0d0d22" stroke="#222"/>
        ${ref(data.entry_avg, '#00e5ff', 'entry')}
        ${ref(data.exit_avg, '#00ffaa', 'exit')}
        ${ref(data.stop_loss, '#ff1f7a', 'stop')}
        ${ref(data.initial_target, '#ffd24a', 'target')}
        ${candles}
        ${execMarkers}
        <line x1="${cx}" y1="${padT}" x2="${cx}" y2="${padT + innerH}" stroke="#fff" stroke-dasharray="3,3" opacity="0.5"/>
        ${yAxis}
    </svg>`;

    const pos = mount.querySelector('#tr-pos');
    if (pos && cursorTime) {
        pos.textContent = t('view.tape_replay.status.bar', { i: state.idx + 1, n: bars.length, when: new Date(cursorTime).toLocaleString() });
    }
}

function renderNotes(notes, mount) {
    const el = mount.querySelector('#tr-notes');
    if (!el) return;
    if (!notes.length) { el.innerHTML = '<p data-i18n="view.tape_replay.hint.no_notes_yet" class="muted small">No notes yet.</p>'; return; }
    el.innerHTML = `<ol style="padding-left:18px;">
        ${notes.map(n => `<li class="small">
            <span class="muted">${esc(new Date(n.time_iso || Date.now()).toLocaleString())}</span>
            (bar #${n.cursor_idx + 1}) — ${esc(n.text)}
        </li>`).join('')}
    </ol>`;
}
