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
