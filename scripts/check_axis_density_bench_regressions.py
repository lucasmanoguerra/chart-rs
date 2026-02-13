#!/usr/bin/env python3
"""Validate Criterion guardrails for zoom-adaptive axis-density render benches."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Fail CI when zoom-adaptive axis-density benchmark guardrails are exceeded."
        )
    )
    parser.add_argument(
        "--criterion-root",
        default="target/criterion",
        help="Path to Criterion output root directory.",
    )
    parser.add_argument(
        "--zoom-out-bench-name",
        default="axis_density_zoom_adaptive_out_render",
        help="Criterion benchmark name for zoomed-out axis-density render path.",
    )
    parser.add_argument(
        "--zoom-in-bench-name",
        default="axis_density_zoom_adaptive_in_render",
        help="Criterion benchmark name for zoomed-in axis-density render path.",
    )
    parser.add_argument(
        "--max-zoom-in-out-ratio",
        type=float,
        default=1.80,
        help=(
            "Maximum allowed zoom-in/zoom-out mean ratio. "
            "Smaller values enforce bounded density-path overhead under zoom-in."
        ),
    )
    parser.add_argument(
        "--max-zoom-out-mean-ns",
        type=float,
        default=550_000.0,
        help="Maximum allowed mean latency (ns) for zoomed-out density render benchmark.",
    )
    parser.add_argument(
        "--max-zoom-in-mean-ns",
        type=float,
        default=420_000.0,
        help="Maximum allowed mean latency (ns) for zoomed-in density render benchmark.",
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
        zoom_out_mean_ns = load_mean_point_estimate_ns(
            criterion_root, args.zoom_out_bench_name
        )
        zoom_in_mean_ns = load_mean_point_estimate_ns(
            criterion_root, args.zoom_in_bench_name
        )
    except (FileNotFoundError, ValueError) as exc:
        print(f"[perf-guard] {exc}", file=sys.stderr)
        return 1

    if zoom_out_mean_ns <= 0.0:
        print(
            f"[perf-guard] invalid zoom-out mean latency: {zoom_out_mean_ns}",
            file=sys.stderr,
        )
        return 1

    zoom_in_out_ratio = zoom_in_mean_ns / zoom_out_mean_ns

    print(
        "[perf-guard] axis-density means: "
        f"zoom-out={zoom_out_mean_ns:.2f} ns ({ns_to_us(zoom_out_mean_ns):.2f} us), "
        f"zoom-in={zoom_in_mean_ns:.2f} ns ({ns_to_us(zoom_in_mean_ns):.2f} us), "
        f"zoom-in/out={zoom_in_out_ratio:.4f}"
    )

    failures: list[str] = []
    if zoom_in_out_ratio > args.max_zoom_in_out_ratio:
        failures.append(
            "zoom-in density path overhead exceeded ratio budget: "
            f"zoom-in/out={zoom_in_out_ratio:.4f} > {args.max_zoom_in_out_ratio:.4f}"
        )
    if zoom_out_mean_ns > args.max_zoom_out_mean_ns:
        failures.append(
            "zoom-out density latency exceeded budget: "
            f"{zoom_out_mean_ns:.2f} ns > {args.max_zoom_out_mean_ns:.2f} ns"
        )
    if zoom_in_mean_ns > args.max_zoom_in_mean_ns:
        failures.append(
            "zoom-in density latency exceeded budget: "
            f"{zoom_in_mean_ns:.2f} ns > {args.max_zoom_in_mean_ns:.2f} ns"
        )

    if failures:
        print("[perf-guard] axis-density benchmark regression detected:", file=sys.stderr)
        for failure in failures:
            print(f"[perf-guard] - {failure}", file=sys.stderr)
        return 1

    print("[perf-guard] axis-density benchmark guardrails satisfied.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
