// Fear & Greed gauge — CNN-style 0..100 composite of 7 risk-appetite signals.
import { api } from '../api.js';
import { esc } from '../util.js';

let timer = null;

export async function renderFearGreed(mount) {
    mount.innerHTML = `
        <h1 class="view-title">// FEAR &amp; GREED</h1>
        <p class="muted small">CNN-methodology composite of seven risk-appetite signals.
            Score 0 = extreme fear, 100 = extreme greed. Each component is normalized
            to 0..100 and the average is taken. Refreshes every 90s.</p>

        <div class="chart-panel" id="fgGauge"><div class="boot">loading…</div></div>
        <div id="fgComps"></div>
        <div class="chart-panel">
            <h2>Score bands</h2>
            <table class="trades">
                <thead><tr><th>0–24</th><th>25–44</th><th>45–55</th><th>56–74</th><th>75–100</th></tr></thead>
                <tbody><tr>
                    <td class="neg">Extreme Fear</td>
                    <td class="neg">Fear</td>
                    <td>Neutral</td>
                    <td class="pos">Greed</td>
                    <td class="pos">Extreme Greed</td>
                </tr></tbody>
            </table>
            <p class="muted small">Like all sentiment composites this is contrarian at the extremes —
                Extreme Fear historically aligns with intermediate-term lows, Extreme Greed with
                local tops. Use as a regime filter, not a timing signal.</p>
        </div>
    `;
    if (timer) clearInterval(timer);
    timer = setInterval(refresh, 90_000);
    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#fear-greed')) { clearInterval(timer); timer = null; }
    }, { once: true });
    await refresh();
}

async function refresh() {
    try {
        const s = await api.fearGreed();
        renderGauge(s);
        renderComponents(s);
    } catch (e) {
        document.getElementById('fgGauge').innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function bandColor(score) {
    if (score <= 24) return '#ff1f7a';
    if (score <= 44) return '#ff7a1f';
    if (score <= 55) return '#9aa0c8';
    if (score <= 74) return '#7af0a8';
    return '#00ffaa';
}

function renderGauge(s) {
    // Simple SVG semicircle gauge, 0 (left) → 100 (right).
    const color = bandColor(s.score);
    const angle = (s.score / 100) * 180 - 90; // -90..+90 deg
    const rad = angle * Math.PI / 180;
    const r = 120;
    const cx = 160, cy = 140;
    const nx = cx + r * Math.sin(rad);
    const ny = cy - r * Math.cos(rad);
    document.getElementById('fgGauge').innerHTML = `
        <h2>${esc(s.label)} — ${s.score}/100</h2>
        <svg viewBox="0 0 320 180" width="100%" height="180" style="display:block;max-width:480px;margin:0 auto;">
            <defs>
                <linearGradient id="fg-grad" x1="0" y1="0" x2="1" y2="0">
                    <stop offset="0"   stop-color="#ff1f7a"/>
                    <stop offset="0.25" stop-color="#ff7a1f"/>
                    <stop offset="0.5"  stop-color="#9aa0c8"/>
                    <stop offset="0.75" stop-color="#7af0a8"/>
                    <stop offset="1"    stop-color="#00ffaa"/>
                </linearGradient>
            </defs>
            <path d="M 40 140 A 120 120 0 0 1 280 140" stroke="url(#fg-grad)" stroke-width="22" fill="none" stroke-linecap="round"/>
            <line x1="${cx}" y1="${cy}" x2="${nx}" y2="${ny}" stroke="${color}" stroke-width="3"/>
            <circle cx="${cx}" cy="${cy}" r="6" fill="${color}"/>
            <text x="160" y="170" text-anchor="middle" fill="#9aa0c8" font-size="12">0 fear · 50 neutral · 100 greed</text>
        </svg>
        <p class="muted small" style="text-align:center;">Updated ${new Date(s.fetched_at).toLocaleTimeString(undefined, { hour12: false })}</p>
    `;
}

function renderComponents(s) {
    document.getElementById('fgComps').innerHTML = `
        <div class="cards">
            ${s.components.map(c => {
                const color = bandColor(c.score);
                return `<div class="card">
                    <div class="label">${esc(c.label)}</div>
                    <div class="value" style="color:${color};">${c.score}</div>
                    <div class="muted small">${esc(c.interpretation)}</div>
                </div>`;
            }).join('')}
        </div>
    `;
}
