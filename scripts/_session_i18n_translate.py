"""Translate only the keys added in this session across every locale.

Re-running `gen_all_app_i18n_locales.py` would try to translate ~5k
pre-existing stubs per locale (hours of network calls). This script
scopes the work to the 83 keys I just added:

  - common.month.<jan..dec>
  - common.dow.short.<sun..sat>
  - view.journal.day_step.<prev|next>
  - view.calendar.tv.total
  - view.dashboard.period.<today|wtd|mtd|qtd|ytd|all>
  - view.dashboard.empty.<no_data|no_fires_today|no_open_trades>
  - view.dashboard.tv.*    (~50 dashboard tile keys)
  - nav.dashboards, nav.tip.dashboards
  - toast.graph_pinned

For each locale, only those keys are sent to Google Translate; every
other key is read from the existing locale file verbatim. Output is
written back sorted, matching the rest of the i18n pipeline.
"""

from __future__ import annotations

import json
import re
import sys
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
I18N_DIR = REPO_ROOT / "frontend" / "i18n"
EN_PATH = I18N_DIR / "app_i18n_en.json"

LOCALES = [
    "cs", "da", "de", "el", "es", "es_419", "fi", "fr", "hi", "hu",
    "id", "it", "ja", "ko", "nb", "nl", "pl", "pt", "pt_br", "ro",
    "ru", "sv", "tr", "uk", "vi", "zh",
]

# Google Translate target codes that differ from our filename suffix.
GOOGLE_TARGET_OVERRIDE = {
    "es_419": "es",   # Latin-American Spanish — Google has no -419
    "pt_br":  "pt",   # Brazilian Portuguese
    "nb":     "no",   # Norwegian Bokmål
    "zh":     "zh-CN",
}

KEY_PREFIXES = (
    "common.month.",
    "common.dow.short.",
    "view.dashboard.period.",
    "view.dashboard.empty.no_data",
    "view.dashboard.empty.no_fires_today",
    "view.dashboard.empty.no_open_trades",
    "view.dashboard.tv.",
)
KEY_EXACT = {
    "view.journal.day_step.prev",
    "view.journal.day_step.next",
    "view.calendar.tv.total",
    "nav.dashboards",
    "nav.tip.dashboards",
    "toast.graph_pinned",
}


def is_session_key(k: str) -> bool:
    return k in KEY_EXACT or any(k.startswith(p) for p in KEY_PREFIXES)


_PLACEHOLDER_RE = re.compile(r"\{([A-Za-z_][A-Za-z0-9_]*)\}")


def repair_placeholders(en_val: str, translated: str) -> str:
    """Restore {token} casing the translator may have mangled."""
    tokens = _PLACEHOLDER_RE.findall(en_val)
    for tok in tokens:
        # Repair case-insensitive matches; preserve the rest.
        translated = re.sub(
            r"\{\s*" + re.escape(tok) + r"\s*\}",
            "{" + tok + "}",
            translated,
            flags=re.IGNORECASE,
        )
    return translated


def main() -> int:
    try:
        from deep_translator import GoogleTranslator
    except ImportError:
        print("install: .venv-i18n/bin/pip install deep-translator", file=sys.stderr)
        return 1

    en: dict[str, str] = json.loads(EN_PATH.read_text(encoding="utf-8"))
    session_keys = sorted(k for k in en if is_session_key(k))
    en_values_for_session = sorted({en[k] for k in session_keys})
    print(
        f"session_keys={len(session_keys)} unique_session_values={len(en_values_for_session)}",
        file=sys.stderr,
    )

    for loc in LOCALES:
        target = GOOGLE_TARGET_OVERRIDE.get(loc, loc)
        path = I18N_DIR / f"app_i18n_{loc}.json"
        try:
            existing: dict[str, str] = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as e:
            print(f"[{loc}] skip — unreadable existing file: {e}", file=sys.stderr)
            continue

        translator = GoogleTranslator(source="en", target=target)
        val_map: dict[str, str] = {}
        # Seed from any non-stub existing locale translations of these values.
        for k, en_v in en.items():
            if not is_session_key(k):
                continue
            ex_v = existing.get(k)
            if ex_v and ex_v != en_v and en_v not in val_map:
                val_map[en_v] = ex_v

        todo = [v for v in en_values_for_session if v not in val_map]
        print(
            f"[{loc}] target={target} session_unique_values={len(en_values_for_session)} "
            f"cached={len(en_values_for_session) - len(todo)} to_translate={len(todo)}",
            file=sys.stderr,
            flush=True,
        )
        t0 = time.time()
        for i, v in enumerate(todo, 1):
            if not v.strip() or not re.search(r"[A-Za-z]", v):
                val_map[v] = v
                continue
            try:
                tv = translator.translate(v)
                if not tv:
                    tv = v
            except Exception as e:  # network blip, rate limit, etc.
                print(f"[{loc}] translate failed for {v!r}: {e}; using EN", file=sys.stderr)
                tv = v
            val_map[v] = repair_placeholders(v, tv)
            time.sleep(0.06)
            if i % 10 == 0:
                print(f"  [{loc}] {i}/{len(todo)} elapsed={time.time()-t0:.1f}s",
                      file=sys.stderr, flush=True)

        # Apply translations for session keys; leave every other key untouched.
        for k in session_keys:
            existing[k] = val_map.get(en[k], en[k])
        # Write sorted, matching the rest of the pipeline.
        out = {k: existing[k] for k in sorted(existing)}
        path.write_text(
            json.dumps(out, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
        print(f"[{loc}] wrote {len(out)} keys "
              f"(elapsed={time.time()-t0:.1f}s)",
              file=sys.stderr, flush=True)

    return 0


if __name__ == "__main__":
    sys.exit(main())
