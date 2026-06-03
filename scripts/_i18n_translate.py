"""Shared engine for `gen_app_i18n_<locale>.py` per-locale generators.

Pattern mirrors `../audio_haxor/scripts/gen_app_i18n_*.py`:
- Read `frontend/i18n/app_i18n_en.json` as source of truth.
- Reuse existing non-stub translations from `app_i18n_<locale>.json`
  (a stub is a key whose locale value equals the English value).
- Translate each remaining *unique* English value once (Google Translate
  via `deep-translator`), then map keys back. Saves ~25% calls vs naive.
- Repair `{Token}` placeholders the translator capitalized.
- Write sorted, ensure_ascii=False, indent=2, trailing newline.
- Idempotent for a given English catalog; incremental across runs —
  re-running only translates new/changed keys. Set `TRADERVIEW_I18N_FORCE=1`
  to force a full re-translate.

Requires: `.venv-i18n` with `pip install deep-translator`.
"""
from __future__ import annotations

import json
import os
import pathlib
import re
import sys
import time
from typing import Iterable

ROOT = pathlib.Path(__file__).resolve().parents[1]
I18N_DIR = ROOT / "frontend" / "i18n"
EN_PATH = I18N_DIR / "app_i18n_en.json"

_TOKEN_RE = re.compile(r"\{([A-Za-z_][A-Za-z0-9_]*)\}")


def _tokens(s: str) -> list[str]:
    return _TOKEN_RE.findall(s)


def _repair_placeholders(translated: str, expected: Iterable[str]) -> str:
    """Google Translate routinely capitalizes / spaces / drops `{tok}`s.

    Walk every expected token; if the translation has any case-variant of
    it inside braces, force it back to the original casing. Keeps the
    multiset parity that `i18n-per-key-placeholder-parity` enforces in
    audio-haxor (and that traderview's appFmt depends on)."""
    for tok in expected:
        if "{" + tok + "}" in translated:
            continue
        pat = re.compile(r"\{\s*" + re.escape(tok) + r"\s*\}", re.IGNORECASE)
        translated = pat.sub("{" + tok + "}", translated)
    return translated


def translate_locale(
    locale: str,
    google_target: str | None = None,
    *,
    sleep_s: float = 0.06,
    progress_every: int = 200,
) -> int:
    """Translate `app_i18n_en.json` → `app_i18n_<locale>.json`.

    Returns the number of keys written. Raises on import/IO failure;
    individual translate failures fall back to the English value so the
    catalog stays complete.
    """
    try:
        from deep_translator import GoogleTranslator
    except ImportError:
        print(
            "Install deep-translator: python3 -m venv .venv-i18n && "
            ".venv-i18n/bin/pip install deep-translator",
            file=sys.stderr,
        )
        raise SystemExit(1) from None

    target = google_target or locale
    out_path = I18N_DIR / f"app_i18n_{locale}.json"
    en: dict[str, str] = json.loads(EN_PATH.read_text(encoding="utf-8"))

    force = os.environ.get("TRADERVIEW_I18N_FORCE", "").lower() in {"1", "true", "yes"}
    val_map: dict[str, str] = {}
    if not force and out_path.exists():
        try:
            existing: dict[str, str] = json.loads(out_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as e:
            print(f"[{locale}] ignoring unreadable existing file: {e}", file=sys.stderr)
            existing = {}
        # Seed cache from existing translations. Skip stubs (locale == en);
        # those need real translation. First non-stub per English value wins.
        for k, en_v in en.items():
            ex_v = existing.get(k)
            if ex_v and ex_v != en_v and en_v not in val_map:
                val_map[en_v] = ex_v

    uniq_vals = list(dict.fromkeys(en.values()))
    todo = [v for v in uniq_vals if v not in val_map]
    print(
        f"[{locale}] target={target} keys={len(en)} "
        f"unique_values={len(uniq_vals)} cached={len(uniq_vals) - len(todo)} "
        f"to_translate={len(todo)} force={force}",
        file=sys.stderr,
        flush=True,
    )

    translator = GoogleTranslator(source="en", target=target)
    t0 = time.time()
    for i, v in enumerate(todo):
        # Skip empty / whitespace / pure-punctuation strings; translator
        # raises on these and there is nothing to translate anyway.
        if not v.strip() or not re.search(r"[A-Za-z]", v):
            val_map[v] = v
        else:
            try:
                t = translator.translate(v)
                val_map[v] = t if isinstance(t, str) and t else v
            except Exception as e:  # noqa: BLE001 - network / API jitter
                if i < 3:
                    print(f"[{locale}] translate failed for {v!r}: {e}", file=sys.stderr)
                val_map[v] = v
        if (i + 1) % progress_every == 0:
            elapsed = time.time() - t0
            rate = (i + 1) / elapsed if elapsed > 0 else 0.0
            eta = (len(todo) - (i + 1)) / rate if rate > 0 else 0.0
            print(
                f"[{locale}] {i + 1}/{len(todo)} "
                f"elapsed={elapsed:.0f}s rate={rate:.1f}/s eta={eta:.0f}s",
                file=sys.stderr,
                flush=True,
            )
        if sleep_s:
            time.sleep(sleep_s)

    out: dict[str, str] = {}
    for k, v in en.items():
        tx = val_map.get(v, v)
        expected = _tokens(v)
        if expected:
            tx = _repair_placeholders(tx, expected)
        out[k] = tx

    out_path.write_text(
        json.dumps(out, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(f"[{locale}] wrote {len(out)} keys → {out_path}", file=sys.stderr)
    return len(out)
