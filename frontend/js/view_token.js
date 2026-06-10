// Per-view race-token machinery. Extracted from app.js so the helpers can
// be unit-tested under `node --test` without pulling in DOM-dependent code.
//
// Why this exists: the #app mount element is reused across every view, so
// `mount.isConnected` and `document.body.contains(mount)` are useless for
// detecting navigation. Each dispatch bumps the token; views capture the
// token at render start and bail after every await if it no longer matches.

let _viewToken = 0;

export function bumpViewToken() {
    _viewToken++;
    return _viewToken;
}

export function currentViewToken() {
    return _viewToken;
}

export function viewIsCurrent(tok) {
    return _viewToken === tok;
}

// Exact-route check for view-cleanup guards. The previous pattern of
// `window.location.hash.startsWith('#name')` was a bug: it matched
// `#name-anything` too, so when the user navigated from `#sentiment`
// to `#sentiment-velocity` the cleanup branch never ran and the
// sentiment view's interval + WS subscription leaked. Use this instead:
//   if (!routeIs('sentiment')) { clearInterval(timer); ... }
export function routeIs(name) {
    const hash = (typeof window !== 'undefined' && window.location && window.location.hash) || '';
    const route = hash.slice(1).split('/')[0].split('?')[0];
    return route === name;
}
