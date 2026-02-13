#!/usr/bin/env python3
"""Validate Criterion guardrails for newly added parity benchmark paths."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Fail CI when transformed-base/sparse-index benchmark guardrails are exceeded."
        )
    )
    parser.add_argument(
        "--criterion-root",
        default="target/criterion",
        help="Path to Criterion output root directory.",
    )
    parser.add_argument(
        "--dynamic-base-bench-name",
        default="price_scale_transformed_base_dynamic_refresh_visible_window",
        help="Criterion benchmark name for transformed-base visible-window refresh path.",
    )
    parser.add_argument(
        "--nearest-bench-name",
        default="time_scale_sparse_nearest_filled_slot_lookup",
        help="Criterion benchmark name for sparse nearest-filled-slot lookup path.",
    )
    parser.add_argument(
        "--next-prev-bench-name",
        default="time_scale_sparse_next_prev_filled_lookup",
        help="Criterion benchmark name for sparse next/prev lookup path.",
    )
    parser.add_argument(
        "--max-dynamic-base-mean-ns",
        type=float,
        default=700_000.0,
        help="Maximum allowed mean latency (ns) for transformed-base refresh benchmark.",
    )
    parser.add_argument(
        "--max-nearest-mean-ns",
        type=float,
        default=350_000.0,
        help="Maximum allowed mean latency (ns) for sparse nearest-slot benchmark.",
    )
    parser.add_argument(
        "--max-next-prev-mean-ns",
        type=float,
        default=450_000.0,
        help="Maximum allowed mean latency (ns) for sparse next/prev benchmark.",
    )
    parser.add_argument(
        "--max-next-prev-nearest-ratio",
        type=float,
        default=2.20,
        help=(
            "Maximum allowed next-prev/nearest mean ratio. "
            "Smaller values enforce bounded overhead for navigation helpers."
        ),
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
        dynamic_base_mean_ns = load_mean_point_estimate_ns(
            criterion_root, args.dynamic_base_bench_name
        )
        nearest_mean_ns = load_mean_point_estimate_ns(
            criterion_root, args.nearest_bench_name
        )
        next_prev_mean_ns = load_mean_point_estimate_ns(
            criterion_root, args.next_prev_bench_name
        )
    except (FileNotFoundError, ValueError) as exc:
        print(f"[perf-guard] {exc}", file=sys.stderr)
        return 1

    if nearest_mean_ns <= 0.0:
        print(
            f"[perf-guard] invalid nearest mean latency: {nearest_mean_ns}",
            file=sys.stderr,
        )
        return 1

    next_prev_nearest_ratio = next_prev_mean_ns / nearest_mean_ns

    print(
        "[perf-guard] parity-path means: "
        f"dynamic-base={dynamic_base_mean_ns:.2f} ns ({ns_to_us(dynamic_base_mean_ns):.2f} us), "
        f"nearest={nearest_mean_ns:.2f} ns ({ns_to_us(nearest_mean_ns):.2f} us), "
        f"next-prev={next_prev_mean_ns:.2f} ns ({ns_to_us(next_prev_mean_ns):.2f} us), "
        f"next-prev/nearest={next_prev_nearest_ratio:.4f}"
    )

    failures: list[str] = []
    if dynamic_base_mean_ns > args.max_dynamic_base_mean_ns:
        failures.append(
            "dynamic transformed-base refresh latency exceeded budget: "
            f"{dynamic_base_mean_ns:.2f} ns > {args.max_dynamic_base_mean_ns:.2f} ns"
        )
    if nearest_mean_ns > args.max_nearest_mean_ns:
        failures.append(
            "sparse nearest-slot latency exceeded budget: "
            f"{nearest_mean_ns:.2f} ns > {args.max_nearest_mean_ns:.2f} ns"
        )
    if next_prev_mean_ns > args.max_next_prev_mean_ns:
        failures.append(
            "sparse next/prev latency exceeded budget: "
            f"{next_prev_mean_ns:.2f} ns > {args.max_next_prev_mean_ns:.2f} ns"
        )
    if next_prev_nearest_ratio > args.max_next_prev_nearest_ratio:
        failures.append(
            "sparse next/prev overhead exceeded ratio budget: "
            f"next-prev/nearest={next_prev_nearest_ratio:.4f} > {args.max_next_prev_nearest_ratio:.4f}"
        )

    if failures:
        print("[perf-guard] parity benchmark regression detected:", file=sys.stderr)
        for failure in failures:
            print(f"[perf-guard] - {failure}", file=sys.stderr)
        return 1

    print("[perf-guard] parity benchmark guardrails satisfied.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
