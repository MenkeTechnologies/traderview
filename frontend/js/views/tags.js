import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

export async function renderTags(mount) {
    const tok = currentViewToken();
    const tags = await api.tags();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.tags.h1.tags" class="view-title">// TAGS</h1>
        <div class="chart-panel">
            <h2 data-i18n="view.tags.h2.add_tag">Add tag</h2>
            <form id="tag-form" class="inline-form">
                <input name="name" placeholder="tag name" data-i18n-placeholder="view.tags.placeholder.name" required>
                <input type="color" name="color" value="#00e5ff">
                <button data-i18n="view.tags.btn.create" class="primary" type="submit">Create</button>
            </form>
        </div>
        <div class="tag-list">${tags.map(tag => `
            <span class="tag-chip"
                  data-context-scope="tag-chip"
                  data-id="${esc(tag.id)}"
                  data-name="${esc(tag.name)}"
                  style="border-color:${esc(tag.color)}">
                ${esc(tag.name)}
                <button class="link" data-del="${tag.id}" data-i18n-aria-label="common.aria.remove" aria-label="Remove">×</button>
            </span>
        `).join('') || '<p data-i18n="view.tags.hint.no_tags_yet" class="muted">No tags yet.</p>'}</div>

        <div class="chart-panel">
            <h2 data-i18n="view.tags.h2.length_chart">Tag name length per tag</h2>
            <div id="tag-chart" style="width:100%;height:200px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.tags.h2.color_chart">Tag count per color (palette frequency)</h2>
            <div id="tag-color-chart" style="width:100%;height:200px"></div>
            <p data-i18n="view.tags.hint.color_chart" class="muted small">How many tags share each color value. Reveals palette monotony — if most tags pile up on one or two colors, visual distinguishability in the trades table suffers. Orthogonal to per-tag name length.</p>
        </div>
    `;
    renderTagsChart(tags);
    renderTagsColorChart(tags);
    mount.querySelector('#tag-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.createTag(fd.get('name'), fd.get('color'));
        if (!viewIsCurrent(tok)) return;
        renderTags(mount);
    });
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteTag(b.dataset.del);
            if (!viewIsCurrent(tok)) return;
            renderTags(mount);
        }));
}

function renderTagsColorChart(tags) {
    const el = document.getElementById('tag-color-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (tags || []).filter(tag => tag && tag.color);
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.tags.empty_color_chart">${esc(t('view.tags.empty_color_chart'))}</div>`;
        return;
    }
    const buckets = new Map();
    for (const tag of rows) {
        const c = String(tag.color).toLowerCase();
        buckets.set(c, (buckets.get(c) || 0) + 1);
    }
    const sorted = [...buckets.entries()].sort((a, b) => b[1] - a[1]);
    const labels = sorted.map(([c]) => c);
    const ys = sorted.map(([, n]) => n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.tags.chart.color') },
            { label: t('view.tags.chart.count'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderTagsChart(tags) {
    const el = document.getElementById('tag-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (tags || []).filter(tag => tag && tag.name);
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.tags.empty_chart">${esc(t('view.tags.empty_chart'))}</div>`;
        return;
    }
    const labels = rows.map(tag => tag.name);
    const xs = labels.map((_, i) => i + 1);
    const ys = rows.map(tag => String(tag.name).length);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.tags.chart.tag') },
            { label: t('view.tags.chart.len'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 14, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}
