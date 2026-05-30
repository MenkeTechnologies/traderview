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
                    ${TRIGGERS.map(trig => `<option value="${trig}" data-i18n="view.alerts.trigger.${trig}">${esc(trig)}</option>`).join('')}
                </select>
                <input name="threshold" type="number" step="any" placeholder="threshold" data-i18n-placeholder="common.placeholder.threshold">
                <select name="sound">
                    ${SOUNDS.map(s => `<option value="${s}" data-i18n="view.alerts.sound.${s}">${esc(s)}</option>`).join('')}
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
                        <td>${esc(t(`view.alerts.trigger.${r.trigger}`))}</td>
                        <td>${r.threshold != null ? r.threshold : '—'}</td>
                        <td>${esc(t(`view.alerts.sound.${r.sound}`))}</td>
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

        <div class="chart-panel">
            <h2 data-i18n="view.alerts.h2.fire_chart">Trigger counts by rule (top 20)</h2>
            <div id="alert-chart" style="width:100%;height:240px"></div>
        </div>
    `;
    renderFireChart(rules);
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

function renderFireChart(rules) {
    const el = document.getElementById('alert-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const top = (rules || [])
        .filter(r => Number.isFinite(Number(r.trigger_count)))
        .sort((a, b) => Number(b.trigger_count) - Number(a.trigger_count))
        .slice(0, 20);
    if (top.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.alerts.empty_chart">${esc(t('view.alerts.empty_chart'))}</div>`;
        return;
    }
    const labels = top.map(r => r.symbol);
    const ys = top.map(r => Number(r.trigger_count));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.alerts.chart.rule_idx') },
            { label: t('view.alerts.chart.count'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}
