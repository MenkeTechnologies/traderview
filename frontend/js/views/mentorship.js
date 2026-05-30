import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

export async function renderMentorship(mount) {
    const tok = currentViewToken();
    const [mentors, mentees] = await Promise.all([api.mentors(), api.mentees()]);
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.mentorship.h1.mentorship" class="view-title">// MENTORSHIP</h1>

        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.mentorship.h2.your_mentors_people_watching_you">Your mentors (people watching you)</h2>
                ${listTable(mentors, 'mentor_id', false)}
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.mentorship.h2.your_mentees_people_you_watch">Your mentees (people you watch)</h2>
                ${listTable(mentees, 'mentee_id', true)}
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.mentorship.h2.invite_a_mentor_paste_their_user_id">Invite a mentor (paste their user ID)</h2>
                <form id="mentor-form" class="inline-form">
                    <input name="mentor_id" placeholder="mentor user UUID" data-i18n-placeholder="view.mentorship.placeholder.mentor" required>
                    <select name="scope">
                        <option data-i18n="view.mentorship.opt.read_only" value="read">read-only</option>
                        <option data-i18n="view.mentorship.opt.read_comment" value="comment">read + comment</option>
                    </select>
                    <button data-i18n="view.mentorship.btn.invite" class="primary" type="submit">Invite</button>
                </form>
            </div>
            <div class="chart-panel" style="grid-column: 1 / -1">
                <h2 data-i18n="view.mentorship.h2.status_chart">Mentorship status counts</h2>
                <div id="m-chart" style="width:100%;height:240px"></div>
            </div>
            <div class="chart-panel" style="grid-column: 1 / -1">
                <h2 data-i18n="view.mentorship.h2.scope_chart">Mentorship scope distribution (read-only vs read+comment)</h2>
                <div id="m-scope-chart" style="width:100%;height:220px"></div>
                <p data-i18n="view.mentorship.hint.scope_chart" class="muted small">Permission level across all connections. Complementary to the lifecycle status chart — reveals how much trust your mentors and mentees actually share.</p>
            </div>
        </div>
    `;
    renderStatusChart(mentors, mentees);
    renderScopeChart(mentors, mentees);

    mount.querySelector('#mentor-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.mentorshipRequest(fd.get('mentor_id'), fd.get('scope'));
        if (!viewIsCurrent(tok)) return;
        renderMentorship(mount);
    });

    mount.querySelectorAll('[data-accept]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.acceptMentorship(b.dataset.accept);
            if (!viewIsCurrent(tok)) return;
            renderMentorship(mount);
        }));
    mount.querySelectorAll('[data-revoke]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.revokeMentorship(b.dataset.revoke);
            if (!viewIsCurrent(tok)) return;
            renderMentorship(mount);
        }));
}

function renderStatusChart(mentors, mentees) {
    const el = document.getElementById('m-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const buckets = [
        { key: 'mentors_pending',  count: (mentors || []).filter(r => r.status === 'pending').length, color: '#ff7a1f' },
        { key: 'mentors_accepted', count: (mentors || []).filter(r => r.status === 'accepted').length, color: '#7af0a8' },
        { key: 'mentees_pending',  count: (mentees || []).filter(r => r.status === 'pending').length, color: '#ffd84a' },
        { key: 'mentees_accepted', count: (mentees || []).filter(r => r.status === 'accepted').length, color: '#00e5ff' },
    ];
    if (!buckets.some(b => b.count > 0)) {
        el.innerHTML = `<div class="muted" data-i18n="view.mentorship.empty_chart">${esc(t('view.mentorship.empty_chart'))}</div>`;
        return;
    }
    const labels = buckets.map(b => t(`view.mentorship.chart.${b.key}`));
    const xs = labels.map((_, i) => i + 1);
    const series = [{ label: t('view.mentorship.chart.bucket_idx') }];
    const data = [xs];
    buckets.forEach((b, i) => {
        const ys = xs.map((_, j) => j === i ? b.count : null);
        series.push({
            label: labels[i],
            stroke: b.color, width: 0,
            points: { show: true, size: 16, fill: b.color, stroke: b.color },
        });
        data.push(ys);
    });
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series,
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, data, el);
}

function renderScopeChart(mentors, mentees) {
    const el = document.getElementById('m-scope-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const buckets = [
        { key: 'mentors_read',     count: (mentors || []).filter(r => r.scope === 'read').length,    color: '#aab' },
        { key: 'mentors_comment',  count: (mentors || []).filter(r => r.scope === 'comment').length, color: '#7af0a8' },
        { key: 'mentees_read',     count: (mentees || []).filter(r => r.scope === 'read').length,    color: '#ffd84a' },
        { key: 'mentees_comment',  count: (mentees || []).filter(r => r.scope === 'comment').length, color: '#b86bff' },
    ];
    if (!buckets.some(b => b.count > 0)) {
        el.innerHTML = `<div class="muted" data-i18n="view.mentorship.empty_scope_chart">${esc(t('view.mentorship.empty_scope_chart'))}</div>`;
        return;
    }
    const labels = buckets.map(b => t(`view.mentorship.chart.${b.key}`));
    const xs = labels.map((_, i) => i + 1);
    const series = [{ label: t('view.mentorship.chart.bucket_idx') }];
    const data = [xs];
    buckets.forEach((b, i) => {
        const ys = xs.map((_, j) => j === i ? b.count : null);
        series.push({
            label: labels[i],
            stroke: b.color, width: 0,
            points: { show: true, size: 16, fill: b.color, stroke: b.color },
        });
        data.push(ys);
    });
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series,
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, data, el);
}

function listTable(rows, idCol, isMentor) {
    if (!rows.length) return '<p data-i18n="view.mentorship.hint.none" class="muted">None.</p>';
    return `<table class="trades"><thead><tr>
        <th data-i18n="view.mentorship.th.uuid">UUID</th><th data-i18n="view.mentorship.th.status">Status</th><th data-i18n="view.mentorship.th.scope">Scope</th><th data-i18n="view.mentorship.th.created">Created</th><th data-i18n="view.mentorship.th.accepted">Accepted</th><th></th>
    </tr></thead><tbody>${rows.map(r => `
        <tr><td class="muted small">${esc(r[idCol])}</td>
        <td>${r.status}</td>
        <td>${r.scope}</td>
        <td>${fmtDateTime(r.created_at)}</td>
        <td>${r.accepted_at ? fmtDateTime(r.accepted_at) : '—'}</td>
        <td>
            ${isMentor && r.status === 'pending' ? `<button data-i18n="view.mentorship.btn.accept" class="link" data-accept="${r.id}">accept</button>` : ''}
            <button data-i18n="view.mentorship.btn.revoke" class="link" data-revoke="${r.id}">revoke</button>
        </td></tr>
    `).join('')}</tbody></table>`;
}
