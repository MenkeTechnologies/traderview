// Globally disable spellcheck on every text input + textarea.
//
// Patching ~200 view files to add `spellcheck="false"` to every input
// tag would be churn and would rot — new view files would forget the
// attribute and re-introduce the red squiggles. A mutation observer
// catches every input/textarea added to the DOM at boot (initial
// markup, hydration) and at every subsequent dispatch / modal mount.
//
// We don't touch:
//   * inputs that explicitly want spellcheck (none today)
//   * code blocks / contentEditable surfaces (no contenteditable
//     elements in the topbar today; if any get added later, they can
//     opt back in by setting `spellcheck="true"` themselves)
//
// We also kill iOS-style autocorrect / autocapitalize since they're
// equally annoying for symbol fields, slugs, etc.

const HANDLED = new WeakSet();

function clean(el) {
    if (!el || HANDLED.has(el)) return;
    HANDLED.add(el);
    el.spellcheck = false;
    if (!el.hasAttribute('autocorrect')) el.setAttribute('autocorrect', 'off');
    if (!el.hasAttribute('autocapitalize')) el.setAttribute('autocapitalize', 'off');
}

function sweep(root = document) {
    // querySelectorAll on `document` covers the whole tree at boot.
    // On mutation we get a subtree root; same call still finds inputs
    // anywhere inside it.
    if (!root || !root.querySelectorAll) return;
    root.querySelectorAll('input, textarea').forEach(clean);
}

export function installNoSpellcheck() {
    if (typeof document === 'undefined') return;
    if (typeof window !== 'undefined' && window.__tvNoSpellcheckInstalled) return;
    if (typeof window !== 'undefined') window.__tvNoSpellcheckInstalled = true;

    // Initial pass for any markup already in the DOM.
    sweep(document);

    // Observe additions. The handlers fire for re-renders done with
    // `innerHTML = "..."` (which replaces children, so we see them as
    // added nodes on the parent).
    const obs = new MutationObserver((mutations) => {
        for (const m of mutations) {
            for (const node of m.addedNodes) {
                if (node.nodeType !== 1) continue;
                if (node.tagName === 'INPUT' || node.tagName === 'TEXTAREA') {
                    clean(node);
                } else if (node.querySelectorAll) {
                    sweep(node);
                }
            }
        }
    });
    obs.observe(document.body, { childList: true, subtree: true });
}
