#!/usr/bin/env python3
"""Validate Criterion fallback benchmark guardrails for display-mode cache paths."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Fail CI when display-mode fallback benchmark guardrails are exceeded."
        )
    )
    parser.add_argument(
        "--criterion-root",
        default="target/criterion",
        help="Path to Criterion output root directory.",
    )
    parser.add_argument(
        "--hot-bench-name",
        default="price_axis_display_mode_fallback_cache_hot_mixed",
        help="Criterion benchmark name for the cache-hot mixed fallback path.",
    )
    parser.add_argument(
        "--cold-bench-name",
        default="price_axis_display_mode_fallback_cache_cold_mixed",
        help="Criterion benchmark name for the cache-cold mixed fallback path.",
    )
    parser.add_argument(
        "--max-hot-cold-ratio",
        type=float,
        default=0.90,
        help=(
            "Maximum allowed hot/cold mean ratio. "
            "Smaller values enforce a larger cache-hot advantage."
        ),
    )
    parser.add_argument(
        "--max-hot-mean-ns",
        type=float,
        default=250_000.0,
        help="Maximum allowed mean latency (ns) for the hot fallback benchmark.",
    )
    parser.add_argument(
        "--max-cold-mean-ns",
        type=float,
        default=350_000.0,
        help="Maximum allowed mean latency (ns) for the cold fallback benchmark.",
    )
    return parser.parse_args()


def load_mean_point_estimate_ns(criterion_root: pathlib.Path, bench_name: str) -> float:
    estimates_path = criterion_root / bench_name / "new" / "estimates.json"
    if not estimates_path.exists():
        raise FileNotFoundError(
            f"missing criterion estimates file for '{bench_name}': {estimates_path}"
        )

    with estimates_path.open("r", encoding="utf-8") as handle:
        payload = json.load(handle)

    try:
        return float(payload["mean"]["point_estimate"])
    except (KeyError, TypeError, ValueError) as exc:
        raise ValueError(
            f"invalid criterion estimates schema in {estimates_path}: {exc}"
        ) from exc


def ns_to_us(value_ns: float) -> float:
    return value_ns / 1_000.0


def main() -> int:
    args = parse_args()
    criterion_root = pathlib.Path(args.criterion_root)

    try:
        hot_mean_ns = load_mean_point_estimate_ns(criterion_root, args.hot_bench_name)
        cold_mean_ns = load_mean_point_estimate_ns(criterion_root, args.cold_bench_name)
    except (FileNotFoundError, ValueError) as exc:
        print(f"[perf-guard] {exc}", file=sys.stderr)
        return 1

    if cold_mean_ns <= 0.0:
        print(
            f"[perf-guard] invalid cold mean latency: {cold_mean_ns}",
            file=sys.stderr,
        )
        return 1

    hot_cold_ratio = hot_mean_ns / cold_mean_ns

    print(
        "[perf-guard] fallback means: "
        f"hot={hot_mean_ns:.2f} ns ({ns_to_us(hot_mean_ns):.2f} us), "
        f"cold={cold_mean_ns:.2f} ns ({ns_to_us(cold_mean_ns):.2f} us), "
        f"hot/cold={hot_cold_ratio:.4f}"
    )

    failures: list[str] = []
    if hot_cold_ratio > args.max_hot_cold_ratio:
        failures.append(
            "cache-hot fallback path lost expected advantage: "
            f"hot/cold={hot_cold_ratio:.4f} > {args.max_hot_cold_ratio:.4f}"
        )
    if hot_mean_ns > args.max_hot_mean_ns:
        failures.append(
            "cache-hot fallback latency exceeded budget: "
            f"{hot_mean_ns:.2f} ns > {args.max_hot_mean_ns:.2f} ns"
        )
    if cold_mean_ns > args.max_cold_mean_ns:
        failures.append(
            "cache-cold fallback latency exceeded budget: "
            f"{cold_mean_ns:.2f} ns > {args.max_cold_mean_ns:.2f} ns"
        )

    if failures:
        print("[perf-guard] fallback benchmark regression detected:", file=sys.stderr)
        for failure in failures:
            print(f"[perf-guard] - {failure}", file=sys.stderr)
        return 1

    print("[perf-guard] fallback benchmark guardrails satisfied.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
