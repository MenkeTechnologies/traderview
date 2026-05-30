import { api } from '../api.js';
import { esc, fmtDateTime, md } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

export async function renderCommunity(mount, _state, catSlug) {
    const tok = currentViewToken();
    const cats = await api.forumCategories();
    if (!viewIsCurrent(tok)) return;
    if (!cats.length) { mount.innerHTML = '<p data-i18n="view.community.hint.no_forum_categories" class="boot">No forum categories.</p>'; return; }
    if (!catSlug) catSlug = cats[0].slug;
    const cat = cats.find(c => c.slug === catSlug) || cats[0];
    const threads = await api.forumThreadsIn(cat.slug);
    if (!viewIsCurrent(tok)) return;

    mount.innerHTML = `
        <h1 data-i18n="view.community.h1.community" class="view-title">// COMMUNITY</h1>
        <div class="forum-cats">${cats.map(c =>
            `<a class="forum-cat ${c.id === cat.id ? 'active' : ''}" href="#community/${c.slug}">${esc(c.name)}</a>`
        ).join('')}</div>

        <div class="chart-panel">
            <h2>${esc(t('view.community.h2.new_thread', { category: cat.name }))}</h2>
            <form id="thread-form">
                <input name="title" placeholder="title" data-i18n-placeholder="common.placeholder.title"
                       data-tip="view.community.tip.title" data-shortcut="community_focus_title" required>
                <textarea name="body_md" placeholder="markdown body" data-i18n-placeholder="view.community.placeholder.body"
                          data-tip="view.community.tip.body" required></textarea>
                <button data-i18n="view.community.btn.post" data-tip="view.community.tip.post" class="primary" type="submit">Post</button>
            </form>
        </div>

        <table class="trades">
            <thead><tr><th data-i18n="view.community.th.title">Title</th><th data-i18n="view.community.th.posts">Posts</th><th data-i18n="view.community.th.views">Views</th><th data-i18n="view.community.th.last_post">Last post</th></tr></thead>
            <tbody>${threads.map(th => `
                <tr>
                    <td>${th.is_pinned ? '📌 ' : ''}<a href="#community/${cat.slug}/${th.slug}">${esc(th.title)}</a></td>
                    <td>${th.post_count}</td>
                    <td>${th.view_count}</td>
                    <td>${fmtDateTime(th.last_post_at)}</td>
                </tr>`).join('') || `<tr><td colspan="4" class="muted">${esc(t('view.community.empty.threads'))}</td></tr>`}
            </tbody>
        </table>

        <div class="chart-panel">
            <h2 data-i18n="view.community.h2.engagement_chart">Top threads — views vs posts</h2>
            <div id="comm-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.community.h2.recency_chart">Thread last-post recency (activity timing distribution)</h2>
            <div id="comm-recency-chart" style="width:100%;height:220px"></div>
        </div>
    `;
    renderEngagementChart(threads);
    renderRecencyChart(threads);
    mount.querySelector('#thread-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const title = String(fd.get('title') || '').trim();
        try {
            await api.forumCreateThread(cat.id, title, fd.get('body_md'));
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.community.toast.thread_posted', { title }), { level: 'success' });
            renderCommunity(mount, _state, cat.slug);
        } catch (err) {
            showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
        }
    });
}

function renderEngagementChart(threads) {
    const el = document.getElementById('comm-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const top = (threads || [])
        .filter(th => Number.isFinite(Number(th.view_count)) || Number.isFinite(Number(th.post_count)))
        .sort((a, b) => Number(b.view_count || 0) - Number(a.view_count || 0))
        .slice(0, 20);
    if (top.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.community.empty_chart">${esc(t('view.community.empty_chart'))}</div>`;
        return;
    }
    const labels = top.map(th => th.title.slice(0, 20));
    const views = top.map(th => Number(th.view_count || 0));
    const posts = top.map(th => Number(th.post_count || 0));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.community.chart.thread_idx') },
            { label: t('view.community.chart.views'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.community.chart.posts'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 6, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, views, posts], el);
}

function renderRecencyChart(threads) {
    const el = document.getElementById('comm-recency-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!Array.isArray(threads) || threads.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.community.empty_recency_chart">${esc(t('view.community.empty_recency_chart'))}</div>`;
        return;
    }
    const now = Date.now();
    const labels = [
        t('view.community.chart.recency.hour'),
        t('view.community.chart.recency.day'),
        t('view.community.chart.recency.week'),
        t('view.community.chart.recency.month'),
        t('view.community.chart.recency.older'),
    ];
    const counts = new Array(labels.length).fill(0);
    for (const th of threads) {
        const ts = Date.parse(th.last_post_at || th.created_at);
        if (!Number.isFinite(ts)) continue;
        const ageMs = now - ts;
        const hr = 3600 * 1000;
        if (ageMs < hr) counts[0] += 1;
        else if (ageMs < 24 * hr) counts[1] += 1;
        else if (ageMs < 7 * 24 * hr) counts[2] += 1;
        else if (ageMs < 30 * 24 * hr) counts[3] += 1;
        else counts[4] += 1;
    }
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.community.chart.recency_idx') },
            { label: t('view.community.chart.thread_count'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 14, fill: '#7af0a8', stroke: '#7af0a8' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, counts], el);
}

export async function renderCommunityThread(mount, _state, catSlug, threadSlug) {
    const tok = currentViewToken();
    const thread = await api.forumThreadBySlug(catSlug, threadSlug);
    if (!viewIsCurrent(tok)) return;
    api.forumBumpView(thread.id).catch(() => {});
    const posts = await api.forumPosts(thread.id);
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title">// ${esc(thread.title)}
            <a class="link small" href="#community/${catSlug}">← back</a></h1>
        <div class="thread-posts">${posts.map(p => `
            <div class="forum-post ${p.is_op ? 'op' : ''}">
                <div class="meta">${fmtDateTime(p.created_at)}</div>
                <div class="body">${md(p.body_md)}</div>
            </div>
        `).join('')}</div>
        ${thread.is_locked ? '<p data-i18n="view.community.hint.thread_is_locked" class="muted">Thread is locked.</p>' : `
            <div class="chart-panel">
                <h2 data-i18n="view.community.h2.reply">Reply</h2>
                <form id="reply-form">
                    <textarea name="body_md" placeholder="markdown reply" data-i18n-placeholder="view.community.placeholder.reply"
                              data-tip="view.community.tip.reply" required></textarea>
                    <button data-i18n="view.community.btn.post_2" data-tip="view.community.tip.post_reply" class="primary" type="submit">Post</button>
                </form>
            </div>
        `}
    `;
    if (!thread.is_locked) {
        mount.querySelector('#reply-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const fd = new FormData(e.target);
            try {
                await api.forumCreatePost(thread.id, fd.get('body_md'));
                if (!viewIsCurrent(tok)) return;
                showToast(t('view.community.toast.reply_posted'), { level: 'success' });
                renderCommunityThread(mount, _state, catSlug, threadSlug);
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        });
    }
}
