#!/usr/bin/env python3

# Copyright 2026 Andrew Medworth <github@medworth.org.uk>
#
# This file is part of Dots-and-Boxes Engine.
#
# Dots-and-Boxes Engine is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# Dots-and-Boxes Engine is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with Dots-and-Boxes Engine.  If not, see <http://www.gnu.org/licenses/>.

"""Report the changes Criterion measured against a baseline, for GitHub Actions.

Reads the results Criterion saved after a run with --baseline (or --baseline-lenient),
prints a GitHub Actions warning for each benchmark which is slower by more than a
threshold, even at the lower bound of its 95% confidence interval, and writes a table of
every benchmark's change as Markdown, to the job summary if there is one.

Criterion saves each benchmark's change from the baseline in change/estimates.json, as
fractions, and the benchmark's name in new/benchmark.json. These files aren't a
documented interface, so it is an error (exit status 1) if there are none, or they can't
be read, rather than silently reporting nothing.
"""

import argparse
import json
import os
import sys
from dataclasses import dataclass
from pathlib import Path


class ReportError(Exception):
    """Criterion's results are missing or not in the expected form"""


@dataclass(frozen=True)
class Change:
    """A benchmark's change in mean time from the baseline, in percent"""
    name: str
    lower: float
    estimate: float
    upper: float

    def is_regression(self, threshold_percent):
        """Whether the benchmark is slower by more than the threshold, even at the lower bound"""
        return self.lower > threshold_percent


def read_json(path):
    """Read a JSON file, raising ReportError if it is missing or invalid"""
    try:
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError) as e:
        raise ReportError(f"Can't read {path}: {e}") from e


def read_change(estimates_path):
    """Read one benchmark's change, given the path of its change/estimates.json"""
    benchmark = read_json(estimates_path.parent.parent / "new" / "benchmark.json")
    estimates = read_json(estimates_path)
    try:
        mean = estimates["mean"]
        interval = mean["confidence_interval"]
        return Change(
            name=str(benchmark["title"]),
            lower=float(interval["lower_bound"]) * 100,
            estimate=float(mean["point_estimate"]) * 100,
            upper=float(interval["upper_bound"]) * 100,
        )
    except (KeyError, TypeError, ValueError) as e:
        raise ReportError(f"Unexpected contents in {estimates_path}: {e!r}") from e


def read_changes(criterion_home):
    """Read every benchmark's change from a Criterion output directory, sorted by name"""
    paths = sorted(Path(criterion_home).glob("**/change/estimates.json"))
    if not paths:
        raise ReportError(
            f"No Criterion comparison results found in {criterion_home}; "
            "has its output format changed?")
    return sorted((read_change(p) for p in paths), key=lambda c: c.name)


def escape_command_data(text):
    """Escape text for the message of a GitHub Actions workflow command"""
    return text.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")


def warnings(changes, threshold_percent):
    """GitHub Actions warning commands for the benchmarks which have regressed"""
    return [
        "::warning title=Benchmark regression::" + escape_command_data(
            f"{c.name} is {c.estimate:.1f}% slower "
            f"(95% interval {c.lower:+.1f}% to {c.upper:+.1f}%)")
        for c in changes if c.is_regression(threshold_percent)
    ]


def summary(changes, threshold_percent):
    """A Markdown table of every benchmark's change"""
    lines = [
        "## Benchmarks against base branch",
        "",
        "Quick-mode `fast-evaluations` benchmarks. A warning is given if the lower bound of "
        f"a benchmark's 95% confidence interval is a slowdown of more than {threshold_percent:g}%.",
        "",
        "| Benchmark | Change in mean time | 95% confidence interval | |",
        "|---|---|---|---|",
    ]
    for c in changes:
        name = c.name.replace("|", "\\|")
        mark = "Slower" if c.is_regression(threshold_percent) else ""
        interval = f"{c.lower:+.1f}% to {c.upper:+.1f}%"
        lines.append(f"| {name} | {c.estimate:+.1f}% | {interval} | {mark} |")
    return "\n".join(lines) + "\n"


def main(argv=None):
    """Command-line entry point, returning the exit status"""
    parser = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    parser.add_argument("criterion_home", help="Criterion's output directory")
    parser.add_argument("--threshold-percent", type=float, default=10,
                        help="warn about slowdowns larger than this (default: %(default)s)")
    parser.add_argument("--summary",
                        default=os.environ.get("GITHUB_STEP_SUMMARY"),
                        help="file to append the summary table to "
                             "(default: $GITHUB_STEP_SUMMARY, or else standard output)")
    args = parser.parse_args(argv)

    try:
        changes = read_changes(args.criterion_home)
    except ReportError as e:
        print("::error::" + escape_command_data(str(e)))
        return 1

    for warning in warnings(changes, args.threshold_percent):
        print(warning)
    table = summary(changes, args.threshold_percent)
    if args.summary:
        with open(args.summary, "a", encoding="utf-8") as f:
            f.write(table)
    else:
        print(table, end="")
    return 0


if __name__ == "__main__":
    sys.exit(main())
