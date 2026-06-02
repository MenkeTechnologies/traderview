#!/usr/bin/env python3
"""Fast (no network) stub sync.

For each non-English `app_i18n_<loc>.json`:
- Add any key present in English but missing locally — value defaults to the
  English string (will render as English at runtime; loadLocale already
  falls back to English under the hood, but having the key locally lets
  `node --test` parity checks pass).
- Drop any local key that English no longer has (prevents stale junk).
- Preserve any existing translation when the key still exists in English.
- Write sorted, ensure_ascii=False, indent=2, trailing newline.

Run after touching `app_i18n_en.json` if you can't kick off the full
machine-translation pass yet.
"""
from __future__ import annotations

import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
I18N_DIR = ROOT / "frontend" / "i18n"
EN_PATH = I18N_DIR / "app_i18n_en.json"


def main() -> None:
    en: dict[str, str] = json.loads(EN_PATH.read_text(encoding="utf-8"))
    en_keys = set(en.keys())
    touched = 0
    for p in sorted(I18N_DIR.glob("app_i18n_*.json")):
        if p.name == "app_i18n_en.json":
            continue
        loc = p.stem.removeprefix("app_i18n_")
        try:
            cur: dict[str, str] = json.loads(p.read_text(encoding="utf-8"))
        except Exception as e:  # noqa: BLE001
            print(f"[{loc}] read failed: {e}", file=sys.stderr)
            continue
        out: dict[str, str] = {}
        for k in en_keys:
            out[k] = cur.get(k, en[k])
        added = len(en_keys - cur.keys())
        removed = len(cur.keys() - en_keys)
        kept = len(en_keys & cur.keys())
        p.write_text(
            json.dumps(out, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        touched += 1
        print(
            f"[{loc}] kept={kept} added_stubs={added} dropped={removed}",
            file=sys.stderr,
        )
    print(f"\nSynced {touched} locales against {len(en_keys)} English keys.", file=sys.stderr)


if __name__ == "__main__":
    main()
