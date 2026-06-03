// Dashboards sync: push-only reconciliation against /dashboards REST API.

import { test, expect, beforeEach, vi } from 'vitest';

vi.mock('../js/api.js', () => {
    class ApiError extends Error {
        constructor(status, msg) { super(msg); this.status = status; }
    }
    return {
        ApiError,
        api: {
            dashboards: vi.fn(),
            createDashboard: vi.fn(),
            updateDashboard: vi.fn(),
            deleteDashboard: vi.fn(),
        },
    };
});

const { api, ApiError } = await import('../js/api.js');
const sync = await import('../js/_dashboards_sync.js');

function makeState(dashboards) {
    const map = {};
    for (const d of dashboards) map[d.id] = d;
    return {
        version: 1,
        active: dashboards[0]?.id ?? 'main',
        dashboards: map,
    };
}

beforeEach(() => {
    sync._resetForTest();
    api.dashboards.mockReset();
    api.createDashboard.mockReset();
    api.updateDashboard.mockReset();
    api.deleteDashboard.mockReset();
    globalThis.fetch = () => {};   // satisfies the hasAuth fetch check
    globalThis.__tvApiToken = 'test-token';
});

test('pushNow is a no-op when token is missing', async () => {
    globalThis.__tvApiToken = '';
    await sync.pushNow(makeState([{ id: 'main', name: 'Main', tiles: [] }]));
    expect(api.dashboards).not.toHaveBeenCalled();
    expect(api.createDashboard).not.toHaveBeenCalled();
});

test('first sync: lists backend then POSTs slugs missing remotely', async () => {
    api.dashboards.mockResolvedValueOnce([]);
    api.createDashboard.mockImplementation(async (body) => ({
        id: `uuid-${body.layout.slug}`, name: body.name, layout: body.layout,
    }));
    const state = makeState([
        { id: 'main', name: 'Main', tiles: [] },
        { id: 'trading', name: 'Trading', tiles: [{ id: 't1', viewId: 'launcher', config: {} }] },
    ]);

    await sync.pushNow(state);

    expect(api.dashboards).toHaveBeenCalledTimes(1);
    expect(api.createDashboard).toHaveBeenCalledTimes(2);
    const slugs = api.createDashboard.mock.calls.map(([b]) => b.layout.slug).sort();
    expect(slugs).toEqual(['main', 'trading']);
    expect(api.updateDashboard).not.toHaveBeenCalled();
    expect(api.deleteDashboard).not.toHaveBeenCalled();
});

test('PUT only fires when layout signature actually changed', async () => {
    api.dashboards.mockResolvedValueOnce([]);
    api.createDashboard.mockResolvedValueOnce({
        id: 'uuid-main',
        name: 'Main',
        layout: { slug: 'main', tiles: [] },
    });

    const initial = makeState([{ id: 'main', name: 'Main', tiles: [] }]);
    await sync.pushNow(initial);
    expect(api.createDashboard).toHaveBeenCalledTimes(1);

    // Re-push identical state — no PUT.
    await sync.pushNow(initial);
    expect(api.updateDashboard).not.toHaveBeenCalled();

    // Mutate tiles — one PUT.
    const changed = makeState([
        { id: 'main', name: 'Main', tiles: [{ id: 't1', viewId: 'launcher', config: {} }] },
    ]);
    api.updateDashboard.mockResolvedValueOnce({});
    await sync.pushNow(changed);
    expect(api.updateDashboard).toHaveBeenCalledTimes(1);
    expect(api.updateDashboard.mock.calls[0][0]).toBe('uuid-main');
    expect(api.updateDashboard.mock.calls[0][1].layout.tiles.length).toBe(1);
});

test('DELETE fires only for slugs this session previously upserted', async () => {
    // Backend already has rows from another device — first list should NOT
    // trigger deletes for slugs we never owned locally.
    api.dashboards.mockResolvedValueOnce([
        { id: 'uuid-other', name: 'OtherDevice', layout: { slug: 'other-device', tiles: [] } },
    ]);
    api.createDashboard.mockResolvedValueOnce({
        id: 'uuid-main', name: 'Main', layout: { slug: 'main', tiles: [] },
    });

    await sync.pushNow(makeState([{ id: 'main', name: 'Main', tiles: [] }]));
    expect(api.deleteDashboard).not.toHaveBeenCalled();

    // Now locally remove 'main' (default kicks in — but force empty dashboards
    // map here to exercise the delete branch directly).
    const empty = { version: 1, active: 'placeholder',
        dashboards: { placeholder: { id: 'placeholder', name: 'X', tiles: [] } } };
    api.createDashboard.mockResolvedValueOnce({
        id: 'uuid-placeholder', name: 'X',
        layout: { slug: 'placeholder', tiles: [] },
    });
    await sync.pushNow(empty);
    expect(api.deleteDashboard).toHaveBeenCalledTimes(1);
    expect(api.deleteDashboard.mock.calls[0][0]).toBe('uuid-main');
});

test('404 on update falls back to create + remaps uuid', async () => {
    api.dashboards.mockResolvedValueOnce([
        { id: 'uuid-stale', name: 'Main', layout: { slug: 'main', tiles: [] } },
    ]);
    api.updateDashboard.mockRejectedValueOnce(new ApiError(404, 'gone'));
    api.createDashboard.mockResolvedValueOnce({
        id: 'uuid-fresh', name: 'Main',
        layout: { slug: 'main', tiles: [{ id: 't1', viewId: 'launcher', config: {} }] },
    });

    await sync.pushNow(makeState([
        { id: 'main', name: 'Main', tiles: [{ id: 't1', viewId: 'launcher', config: {} }] },
    ]));

    expect(api.updateDashboard).toHaveBeenCalledTimes(1);
    expect(api.createDashboard).toHaveBeenCalledTimes(1);

    // Next push with same state should now PUT against uuid-fresh, not create.
    await sync.pushNow(makeState([
        { id: 'main', name: 'Main', tiles: [{ id: 't1', viewId: 'launcher', config: {} }] },
    ]));
    // signature unchanged → no further PUT
    expect(api.updateDashboard).toHaveBeenCalledTimes(1);
});

test('schedulePush debounces rapid calls into a single reconcile', async () => {
    vi.useFakeTimers();
    try {
        api.dashboards.mockResolvedValue([]);
        api.createDashboard.mockImplementation(async (body) => ({
            id: `uuid-${body.layout.slug}`, name: body.name, layout: body.layout,
        }));

        for (let i = 0; i < 5; i++) {
            sync.schedulePush(makeState([
                { id: 'main', name: `Main v${i}`, tiles: [] },
            ]));
        }
        // Nothing fired yet — debounce timer not elapsed.
        expect(api.dashboards).not.toHaveBeenCalled();

        await vi.advanceTimersByTimeAsync(600);
        // exactly one list + one create for the coalesced last state
        expect(api.dashboards).toHaveBeenCalledTimes(1);
        expect(api.createDashboard).toHaveBeenCalledTimes(1);
        expect(api.createDashboard.mock.calls[0][0].name).toBe('Main v4');
    } finally {
        vi.useRealTimers();
    }
});
