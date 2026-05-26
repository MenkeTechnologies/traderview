import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { playSound, speak } from '../alert_engine.js';

const TRIGGERS = [
    'price_above', 'price_below',
    'pct_up', 'pct_down',
    'volume_surge',
    'new_high_of_day', 'new_low_of_day',
    'rsi_above', 'rsi_below',
    'cross_sma50', 'cross_sma200',
];
const SOUNDS = ['bell', 'chime', 'voice'];

export async function renderAlerts(mount) {
    const rules = await api.alerts();
    mount.innerHTML = `
        <h1 class="view-title">// ALERTS</h1>
        <p class="muted small">Audio + voice alerts on price / % / volume / signal triggers. Polls every 60s.</p>

        <div class="chart-panel">
            <h2>New alert</h2>
            <form id="alert-form" class="inline-form">
                <input name="symbol" placeholder="symbol" required style="text-transform:uppercase">
                <select name="trigger">
                    ${TRIGGERS.map(t => `<option value="${t}">${t}</option>`).join('')}
                </select>
                <input name="threshold" type="number" step="any" placeholder="threshold">
                <select name="sound">
                    ${SOUNDS.map(s => `<option value="${s}">${s}</option>`).join('')}
                </select>
                <input name="voice_text" placeholder="voice message (optional)">
                <button class="primary" type="submit">Create</button>
                <button type="button" class="link" id="test-bell">test bell</button>
                <button type="button" class="link" id="test-voice">test voice</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2>Active rules</h2>
            ${rules.length ? `<table class="trades">
                <thead><tr><th>Symbol</th><th>Trigger</th><th>Threshold</th>
                    <th>Sound</th><th>Voice</th><th>Last fired</th><th>Count</th><th></th></tr></thead>
                <tbody>${rules.map(r => `
                    <tr>
                        <td>${esc(r.symbol)}</td>
                        <td>${esc(r.trigger)}</td>
                        <td>${r.threshold != null ? r.threshold : '—'}</td>
                        <td>${r.sound}</td>
                        <td>${esc(r.voice_text || '')}</td>
                        <td>${r.triggered_at ? fmtDateTime(r.triggered_at) : '—'}</td>
                        <td>${r.trigger_count}</td>
                        <td>
                            <button class="link" data-tog="${r.id}" data-en="${r.enabled}">
                                ${r.enabled ? 'disable' : 'enable'}
                            </button>
                            <button class="link" data-del="${r.id}">delete</button>
                        </td>
                    </tr>`).join('')}</tbody></table>` : '<p class="muted">No alert rules yet.</p>'}
        </div>
    `;
    document.getElementById('alert-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.createAlert({
            symbol: fd.get('symbol').trim().toUpperCase(),
            trigger: fd.get('trigger'),
            threshold: fd.get('threshold') ? Number(fd.get('threshold')) : null,
            sound: fd.get('sound'),
            voice_text: fd.get('voice_text') || null,
        });
        renderAlerts(mount);
    });
    document.getElementById('test-bell').addEventListener('click', () => playSound('bell'));
    document.getElementById('test-voice').addEventListener('click', () =>
        speak('Test alert from TraderView'));
    document.querySelectorAll('[data-tog]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.toggleAlert(b.dataset.tog, b.dataset.en !== 'true');
            renderAlerts(mount);
        }));
    document.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteAlert(b.dataset.del);
            renderAlerts(mount);
        }));
}
