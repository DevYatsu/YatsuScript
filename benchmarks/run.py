#!/usr/bin/env python3
"""Benchmark YatsuScript against Python and Node.js."""

from __future__ import annotations

import argparse
import shutil
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EXAMPLES = ROOT / "examples"
BENCH_ROOT = ROOT / "benchmarks"


@dataclass(frozen=True)
class Benchmark:
    name: str
    file: str
    runtimes: dict[str, list[str]] = field(default_factory=dict)


BENCHMARKS: dict[str, Benchmark] = {
    "fib": Benchmark(
        name="fib",
        file=str(EXAMPLES / "fib.ys"),
        runtimes={
            "yatsuscript": ["target/release/ysc", str(EXAMPLES / "fib.ys")],
            "python": ["python3", str(BENCH_ROOT / "python" / "fib.py")],
            "node": ["node", str(BENCH_ROOT / "node" / "fib.js")],
        },
    ),
    "prime": Benchmark(
        name="prime",
        file=str(EXAMPLES / "prime.ys"),
        runtimes={
            "yatsuscript": ["target/release/ysc", str(EXAMPLES / "prime.ys")],
            "python": ["python3", str(BENCH_ROOT / "python" / "prime.py")],
            "node": ["node", str(BENCH_ROOT / "node" / "prime.js")],
        },
    ),
    "1million_loop": Benchmark(
        name="1million_loop",
        file=str(EXAMPLES / "loop.ys"),
        runtimes={
            "yatsuscript": ["target/release/ysc", str(EXAMPLES / "loop.ys")],
            "python": ["python3", str(BENCH_ROOT / "python" / "1million_loop.py")],
            "node": ["node", str(BENCH_ROOT / "node" / "1million_loop.js")],
        },
    ),
}

RUNTIME_NAMES = {"yatsuscript": "YatsuScript", "python": "Python", "node": "Node.js"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Benchmark YatsuScript vs Python and Node.js."
    )
    parser.add_argument(
        "benchmarks",
        nargs="*",
        choices=sorted(BENCHMARKS),
        metavar="{" + ",".join(BENCHMARKS) + "}",
        help="Benchmarks to run (default: all).",
    )
    parser.add_argument(
        "--runtime",
        choices=list(RUNTIME_NAMES),
        action="append",
        dest="runtimes",
        help="Limit to specific runtime(s).",
    )
    parser.add_argument(
        "--runs",
        type=int,
        default=5,
        metavar="N",
        help="Number of timed runs per benchmark (default: 5).",
    )
    parser.add_argument(
        "--build",
        action="store_true",
        help="Run `cargo build --release` before benchmarks.",
    )
    return parser.parse_args()


def check_command(name: str, command: list[str]) -> None:
    """Verify the runtime executable exists before benchmarking."""
    executable = command[0]
    path = ROOT / executable if executable.startswith("target/") else None
    if path is not None:
        if not path.exists():
            raise SystemExit(
                f"Missing {name} executable: {path}\n"
                "Build it first with: cargo build --release"
            )
        return
    if shutil.which(executable) is None:
        raise SystemExit(
            f"Required runtime not found in PATH: {executable}"
        )


def run_once(command: list[str], label: str) -> float:
    """Run a benchmark command once and return elapsed seconds."""
    start = time.perf_counter()
    result = subprocess.run(
        command,
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    elapsed = time.perf_counter() - start
    if result.returncode != 0:
        print(f"  [{label}] FAILED (exit {result.returncode})", file=sys.stderr)
        if result.stderr:
            print(f"    stderr: {result.stderr.strip()}", file=sys.stderr)
        raise SystemExit(result.returncode)
    return elapsed


def fmt(seconds: float) -> str:
    return f"{seconds:.6f}s"


def main() -> int:
    args = parse_args()

    if args.build:
        print("Building release binary...")
        subprocess.run(
            ["cargo", "build", "--release", "-p", "ys-cli"],
            cwd=ROOT,
            check=True,
        )

    names = args.benchmarks or list(BENCHMARKS)
    runtimes = args.runtimes or list(RUNTIME_NAMES)

    for name in names:
        benchmark = BENCHMARKS[name]
        print(f"\n\u2501 {benchmark.name} \u2501" + "\u2501" * (40 - len(benchmark.name)))

        for runtime in runtimes:
            command = benchmark.runtimes.get(runtime)
            if command is None:
                continue

            check_command(RUNTIME_NAMES[runtime], command)

            timings = [run_once(command, RUNTIME_NAMES[runtime]) for _ in range(args.runs)]
            avg = statistics.mean(timings)
            best = min(timings)
            worst = max(timings)

            print(
                f"  {RUNTIME_NAMES[runtime]:>12}  "
                f"avg {fmt(avg)}  "
                f"best {fmt(best)}  "
                f"worst {fmt(worst)}"
            )

    return 0


if __name__ == "__main__":
    sys.exit(main())
