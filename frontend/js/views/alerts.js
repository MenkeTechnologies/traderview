import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { playSound, speak } from '../alert_engine.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

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
    const tok = currentViewToken();
    const rules = await api.alerts();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.alerts.h1.alerts" class="view-title">// ALERTS</h1>
        <p data-i18n="view.alerts.hint.audio_voice_alerts_on_price_volume_signal_triggers" class="muted small">Audio + voice alerts on price / % / volume / signal triggers. Polls every 60s.</p>

        <div class="chart-panel">
            <h2 data-i18n="view.alerts.h2.new_alert">New alert</h2>
            <form id="alert-form" class="inline-form">
                <input name="symbol" placeholder="symbol" data-i18n-placeholder="common.placeholder.symbol" required style="text-transform:uppercase">
                <select name="trigger">
                    ${TRIGGERS.map(t => `<option value="${t}">${t}</option>`).join('')}
                </select>
                <input name="threshold" type="number" step="any" placeholder="threshold" data-i18n-placeholder="common.placeholder.threshold">
                <select name="sound">
                    ${SOUNDS.map(s => `<option value="${s}">${s}</option>`).join('')}
                </select>
                <input name="voice_text" placeholder="voice message (optional)" data-i18n-placeholder="view.alerts.placeholder.voice">
                <button data-i18n="view.alerts.btn.create" class="primary" type="submit">Create</button>
                <button data-i18n="view.alerts.btn.test_bell" type="button" class="link" id="test-bell">test bell</button>
                <button data-i18n="view.alerts.btn.test_voice" type="button" class="link" id="test-voice">test voice</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.alerts.h2.active_rules">Active rules</h2>
            ${rules.length ? `<table class="trades">
                <thead><tr><th data-i18n="view.alerts.th.symbol">Symbol</th><th data-i18n="view.alerts.th.trigger">Trigger</th><th data-i18n="view.alerts.th.threshold">Threshold</th>
                    <th data-i18n="view.alerts.th.sound">Sound</th><th data-i18n="view.alerts.th.voice">Voice</th><th data-i18n="view.alerts.th.last_fired">Last fired</th><th data-i18n="view.alerts.th.count">Count</th><th></th></tr></thead>
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
                                ${r.enabled ? t('common.btn.disable_lc') : t('common.btn.enable_lc')}
                            </button>
                            <button data-i18n="view.alerts.btn.delete" class="link" data-del="${r.id}">delete</button>
                        </td>
                    </tr>`).join('')}</tbody></table>` : '<p data-i18n="view.alerts.hint.no_alert_rules_yet" class="muted">No alert rules yet.</p>'}
        </div>
    `;
    mount.querySelector('#alert-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.createAlert({
            symbol: fd.get('symbol').trim().toUpperCase(),
            trigger: fd.get('trigger'),
            threshold: fd.get('threshold') ? Number(fd.get('threshold')) : null,
            sound: fd.get('sound'),
            voice_text: fd.get('voice_text') || null,
        });
        if (!viewIsCurrent(tok)) return;
        renderAlerts(mount);
    });
    mount.querySelector('#test-bell').addEventListener('click', () => playSound('bell'));
    mount.querySelector('#test-voice').addEventListener('click', () =>
        speak(t('view.alerts.test_voice_phrase')));
    mount.querySelectorAll('[data-tog]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.toggleAlert(b.dataset.tog, b.dataset.en !== 'true');
            if (!viewIsCurrent(tok)) return;
            renderAlerts(mount);
        }));
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteAlert(b.dataset.del);
            if (!viewIsCurrent(tok)) return;
            renderAlerts(mount);
        }));
}
