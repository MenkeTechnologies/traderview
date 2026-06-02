#!/usr/bin/env python3
"""Run multiple `gen_app_i18n_<loc>.py` generators in parallel.

Each generator hits Google Translate independently (anonymous endpoint).
Empirically 4 concurrent workers are safe — going higher trips IP-level
throttling. Cuts the full 26-locale pass from ~5 days sequential to
~30 hours wall-clock at 4-way parallelism.

Per-locale stdout/stderr stream to /tmp/tv-i18n/<loc>.log; a single
coordinator line `[parallel] start=<loc>` / `[parallel] done=<loc> rc=N`
prints on stdout so the harness Monitor can surface progress.

Usage:
  .venv-i18n/bin/python scripts/gen_app_i18n_parallel.py
  .venv-i18n/bin/python scripts/gen_app_i18n_parallel.py --workers 4
  .venv-i18n/bin/python scripts/gen_app_i18n_parallel.py --workers 3 --skip de
  .venv-i18n/bin/python scripts/gen_app_i18n_parallel.py --only de,fr,es
"""
from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LOG_DIR = Path("/tmp/tv-i18n")
LOG_DIR.mkdir(parents=True, exist_ok=True)

# Same ordering as the sequential orchestrator (European → CJK → other).
LOCALES: tuple[str, ...] = (
    "de", "es", "es_419", "sv", "fr", "nl", "pt", "pt_br", "it", "el",
    "pl", "ru", "zh", "ja", "ko", "fi", "da", "nb", "tr", "cs",
    "hu", "ro", "uk", "vi", "id", "hi",
)


def translate_one(loc: str) -> tuple[str, int, float]:
    """Run scripts/gen_app_i18n_<loc>.py as a subprocess.

    Returns (locale, return_code, elapsed_seconds). Captures stdout +
    stderr to /tmp/tv-i18n/<loc>.log so concurrent runs don't interleave.
    """
    script = ROOT / "scripts" / f"gen_app_i18n_{loc}.py"
    log_path = LOG_DIR / f"{loc}.log"
    t0 = time.time()
    # Stamp the coordinator log so the Monitor sees a "start" event even
    # before the child first writes (deep_translator import takes ~1s).
    print(f"[parallel] start={loc} log={log_path}", flush=True)
    with open(log_path, "w") as fh:
        rc = subprocess.run(
            [sys.executable, str(script)],
            cwd=str(ROOT),
            stdout=fh,
            stderr=subprocess.STDOUT,
        ).returncode
    elapsed = time.time() - t0
    print(f"[parallel] done={loc} rc={rc} elapsed={elapsed:.0f}s", flush=True)
    return (loc, rc, elapsed)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--workers", type=int, default=4,
                    help="concurrent worker count (default 4)")
    ap.add_argument("--skip", default="",
                    help="comma-separated locales to skip (already in flight elsewhere)")
    ap.add_argument("--only", default="",
                    help="comma-separated locales to translate (overrides default list)")
    args = ap.parse_args()

    skip = {s.strip() for s in args.skip.split(",") if s.strip()}
    only = {s.strip() for s in args.only.split(",") if s.strip()}
    todo = [l for l in LOCALES if (not only or l in only) and l not in skip]
    if not todo:
        print("[parallel] nothing to do", flush=True)
        return

    print(
        f"[parallel] workers={args.workers} todo={len(todo)} "
        f"locales={','.join(todo)}",
        flush=True,
    )

    failed: list[str] = []
    t0 = time.time()
    with ProcessPoolExecutor(max_workers=args.workers) as ex:
        futures = {ex.submit(translate_one, l): l for l in todo}
        for fut in as_completed(futures):
            loc, rc, _ = fut.result()
            if rc != 0:
                failed.append(loc)

    wall = time.time() - t0
    print(f"[parallel] WALL_TIME={wall:.0f}s ({wall / 3600:.2f} h)", flush=True)
    if failed:
        raise SystemExit(f"[parallel] failed locales: {', '.join(failed)}")
    print("[parallel] all locales OK", flush=True)


if __name__ == "__main__":
    main()
