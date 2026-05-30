import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { playSound, speak } from '../alert_engine.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm } from '../dialog.js';

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
                <input name="symbol" data-shortcut="focus_search" data-tip="view.alerts.tip.symbol" placeholder="symbol" data-i18n-placeholder="common.placeholder.symbol" required style="text-transform:uppercase">
                <select name="trigger" data-tip="view.alerts.tip.trigger">
                    ${TRIGGERS.map(trig => `<option value="${trig}" data-i18n="view.alerts.trigger.${trig}">${esc(trig)}</option>`).join('')}
                </select>
                <input name="threshold" type="number" step="any" placeholder="threshold" data-i18n-placeholder="common.placeholder.threshold" data-tip="view.alerts.tip.threshold">
                <select name="sound" data-tip="view.alerts.tip.sound">
                    ${SOUNDS.map(s => `<option value="${s}" data-i18n="view.alerts.sound.${s}">${esc(s)}</option>`).join('')}
                </select>
                <input name="voice_text" placeholder="voice message (optional)" data-i18n-placeholder="view.alerts.placeholder.voice" data-tip="view.alerts.tip.voice_text">
                <button data-i18n="view.alerts.btn.create" data-tip="view.alerts.tip.create" class="primary" type="submit">Create</button>
                <button data-i18n="view.alerts.btn.test_bell" data-tip="view.alerts.tip.test_bell" type="button" class="link" id="test-bell">test bell</button>
                <button data-i18n="view.alerts.btn.test_voice" data-tip="view.alerts.tip.test_voice" type="button" class="link" id="test-voice">test voice</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.alerts.h2.active_rules">Active rules</h2>
            ${rules.length ? `<table class="trades">
                <thead><tr><th data-i18n="view.alerts.th.symbol">Symbol</th><th data-i18n="view.alerts.th.trigger">Trigger</th><th data-i18n="view.alerts.th.threshold">Threshold</th>
                    <th data-i18n="view.alerts.th.sound">Sound</th><th data-i18n="view.alerts.th.voice">Voice</th><th data-i18n="view.alerts.th.last_fired">Last fired</th><th data-i18n="view.alerts.th.count">Count</th><th></th></tr></thead>
                <tbody>${rules.map(r => `
                    <tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
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

        <div class="chart-panel">
            <h2 data-i18n="view.alerts.h2.trigger_kind_chart">Rules by trigger kind</h2>
            <div id="alert-kind-chart" style="width:100%;height:200px"></div>
        </div>
    `;
    renderFireChart(rules);
    renderTriggerKindChart(rules);
    mount.querySelector('#alert-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const symbol = String(fd.get('symbol') || '').trim().toUpperCase();
        try {
            await api.createAlert({
                symbol,
                trigger: fd.get('trigger'),
                threshold: fd.get('threshold') ? Number(fd.get('threshold')) : null,
                sound: fd.get('sound'),
                voice_text: fd.get('voice_text') || null,
            });
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.alerts.toast.created', { symbol }), { level: 'success' });
            renderAlerts(mount);
        } catch (err) {
            showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
        }
    });
    mount.querySelector('#test-bell').addEventListener('click', () => playSound('bell'));
    mount.querySelector('#test-voice').addEventListener('click', () =>
        speak(t('view.alerts.test_voice_phrase')));
    mount.querySelectorAll('[data-tog]').forEach(b =>
        b.addEventListener('click', async () => {
            const willEnable = b.dataset.en !== 'true';
            try {
                await api.toggleAlert(b.dataset.tog, willEnable);
                if (!viewIsCurrent(tok)) return;
                showToast(t(willEnable ? 'view.alerts.toast.enabled' : 'view.alerts.toast.disabled'), { level: 'success' });
                renderAlerts(mount);
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        }));
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            if (!await tConfirm('view.alerts.confirm.delete', {}, { level: 'danger' })) return;
            try {
                await api.deleteAlert(b.dataset.del);
                if (!viewIsCurrent(tok)) return;
                showToast(t('view.alerts.toast.deleted'), { level: 'success' });
                renderAlerts(mount);
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        }));
}

function renderTriggerKindChart(rules) {
    const el = document.getElementById('alert-kind-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!Array.isArray(rules) || rules.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.alerts.empty_kind_chart">${esc(t('view.alerts.empty_kind_chart'))}</div>`;
        return;
    }
    const counts = new Map();
    for (const r of rules) {
        const key = (r.trigger || '?').toString();
        counts.set(key, (counts.get(key) || 0) + 1);
    }
    const pairs = Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
    const labels = pairs.map(([k]) => k);
    const ys = pairs.map(([, n]) => n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.alerts.chart.kind_idx') },
            { label: t('view.alerts.chart.count'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
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
