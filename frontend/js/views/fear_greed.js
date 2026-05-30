// Fear & Greed gauge — CNN-style 0..100 composite of 7 risk-appetite signals.
import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let timer = null;

export async function renderFearGreed(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.fear_greed.h1.fear_and_greed" class="view-title">// FEAR &amp; GREED</h1>
        <p data-i18n="view.fear_greed.hint.cnn_methodology_composite_of_seven_risk_appetite_s" class="muted small">CNN-methodology composite of seven risk-appetite signals.
            Score 0 = extreme fear, 100 = extreme greed. Each component is normalized
            to 0..100 and the average is taken. Refreshes every 90s.</p>

        <div class="chart-panel" id="fgGauge"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
        <div id="fgComps"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.fear_greed.h2.score_bands">Score bands</h2>
            <table class="trades">
                <thead><tr><th>0–24</th><th>25–44</th><th>45–55</th><th>56–74</th><th>75–100</th></tr></thead>
                <tbody><tr>
                    <td class="neg" data-i18n="view.fear_greed.band.extreme_fear">Extreme Fear</td>
                    <td class="neg" data-i18n="view.fear_greed.band.fear">Fear</td>
                    <td data-i18n="view.fear_greed.band.neutral">Neutral</td>
                    <td class="pos" data-i18n="view.fear_greed.band.greed">Greed</td>
                    <td class="pos" data-i18n="view.fear_greed.band.extreme_greed">Extreme Greed</td>
                </tr></tbody>
            </table>
            <p data-i18n="view.fear_greed.hint.like_all_sentiment_composites_this_is_contrarian_a" class="muted small">Like all sentiment composites this is contrarian at the extremes —
                Extreme Fear historically aligns with intermediate-term lows, Extreme Greed with
                local tops. Use as a regime filter, not a timing signal.</p>
        </div>
    `;
    if (timer) clearInterval(timer);
    timer = setInterval(() => {
        if (!viewIsCurrent(tok)) { clearInterval(timer); timer = null; return; }
        refresh(mount, tok);
    }, 90_000);
    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#fear-greed')) { clearInterval(timer); timer = null; }
    }, { once: true });
    await refresh(mount, tok);
}

async function refresh(mount, tok) {
    try {
        const s = await api.fearGreed();
        if (!viewIsCurrent(tok)) return;
        renderGauge(s, mount);
        renderComponents(s, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#fgGauge');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function bandColor(score) {
    if (score <= 24) return '#ff1f7a';
    if (score <= 44) return '#ff7a1f';
    if (score <= 55) return '#9aa0c8';
    if (score <= 74) return '#7af0a8';
    return '#00ffaa';
}

function renderGauge(s, mount) {
    // Simple SVG semicircle gauge, 0 (left) → 100 (right).
    const color = bandColor(s.score);
    const angle = (s.score / 100) * 180 - 90; // -90..+90 deg
    const rad = angle * Math.PI / 180;
    const r = 120;
    const cx = 160, cy = 140;
    const nx = cx + r * Math.sin(rad);
    const ny = cy - r * Math.cos(rad);
    const gaugeEl = mount.querySelector('#fgGauge');
    if (!gaugeEl) return;
    gaugeEl.innerHTML = `
        <h2>${esc(t('view.fear_greed.h2.score', { label: s.label, score: s.score }))}</h2>
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
            <text x="160" y="170" text-anchor="middle" fill="#9aa0c8" font-size="12">${esc(t('view.fear_greed.axis'))}</text>
        </svg>
        <p class="muted small" style="text-align:center;">${esc(t('view.fear_greed.hint.updated', { time: new Date(s.fetched_at).toLocaleTimeString(undefined, { hour12: false }) }))}</p>
    `;
}

function renderComponents(s, mount) {
    const el = mount.querySelector('#fgComps');
    if (!el) return;
    el.innerHTML = `
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
