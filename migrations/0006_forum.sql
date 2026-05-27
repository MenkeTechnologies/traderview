-- 0006 — community message board
-- TraderVue parity: discussion forum with categories, threads, posts, votes.

CREATE TABLE forum_categories (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug            TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    position        INTEGER NOT NULL DEFAULT 0,
    is_archived     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE forum_threads (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id     UUID NOT NULL REFERENCES forum_categories(id) ON DELETE CASCADE,
    author_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    slug            TEXT NOT NULL,
    is_pinned       BOOLEAN NOT NULL DEFAULT FALSE,
    is_locked       BOOLEAN NOT NULL DEFAULT FALSE,
    view_count      BIGINT NOT NULL DEFAULT 0,
    post_count      INTEGER NOT NULL DEFAULT 0,
    last_post_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (category_id, slug)
);
CREATE INDEX forum_threads_category_idx ON forum_threads(category_id, is_pinned DESC, last_post_at DESC);
CREATE INDEX forum_threads_author_idx ON forum_threads(author_id);

CREATE TABLE forum_posts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    thread_id       UUID NOT NULL REFERENCES forum_threads(id) ON DELETE CASCADE,
    author_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body_md         TEXT NOT NULL,
    is_op           BOOLEAN NOT NULL DEFAULT FALSE,           -- first post in thread
    edited_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX forum_posts_thread_idx ON forum_posts(thread_id, created_at);
CREATE INDEX forum_posts_author_idx ON forum_posts(author_id);

-- Trade symbol → "find users trading the same symbol" — denormalized aggregate.
-- Refreshed on a schedule (or on every trade insert) by traderview-core.
CREATE TABLE symbol_activity (
    symbol          TEXT NOT NULL,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_count     INTEGER NOT NULL DEFAULT 0,
    last_trade_at   TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (symbol, user_id)
);
CREATE INDEX symbol_activity_symbol_idx ON symbol_activity(symbol, last_trade_at DESC);

-- Seed default categories.
INSERT INTO forum_categories (slug, name, description, position) VALUES
    ('general',       'General',           'Anything trading-related',                 0),
    ('strategies',    'Strategies',        'Setups, plans, post-mortems',              1),
    ('psychology',    'Psychology',        'Mindset, emotions, routine',               2),
    ('education',     'Education',         'Books, courses, ideas',                    3),
    ('platforms',     'Platforms & Tools', 'Brokers, data, charting, importers',       4),
    ('feedback',      'TraderView',        'Bugs, feature requests, this codebase',    5);
