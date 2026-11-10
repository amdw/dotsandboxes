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

"""Tests for benchmark_report"""

import contextlib
import io
import json
import tempfile
import unittest
import unittest.mock
from pathlib import Path

import benchmark_report
from benchmark_report import Change, ReportError

# A change/estimates.json saved by Criterion 0.8 (composite OLMT, before and after #72)
REAL_ESTIMATES = """{"mean":{"confidence_interval":{"confidence_level":0.95,
"lower_bound":0.2661266747268243,"upper_bound":0.3450412191531316},
"point_estimate":0.30456361706650115,"standard_error":0.02426742280724704},
"median":{"confidence_interval":{"confidence_level":0.95,
"lower_bound":0.2661266747268243,"upper_bound":0.3450412191531316},
"point_estimate":0.30456361706650115,"standard_error":0.02426742280724704}}"""

# The matching new/benchmark.json
REAL_BENCHMARK = """{"group_id":"fast-evaluations","function_id":"Eval composite OLMT",
"value_str":null,"throughput":null,"full_id":"fast-evaluations/Eval composite OLMT",
"directory_name":"fast-evaluations/Eval composite OLMT",
"title":"fast-evaluations/Eval composite OLMT"}"""


class CriterionDirTest(unittest.TestCase):
    """Tests which build a Criterion output directory"""

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()  # pylint: disable=consider-using-with
        self.addCleanup(tmp.cleanup)
        self.home = Path(tmp.name)

    def add_benchmark(self, name, change=None, estimates=None, benchmark=None):
        """Add a benchmark's results, with its change as (lower, estimate, upper) fractions,
        or else the given file contents; with neither, it has no baseline to compare with"""
        bench_dir = self.home / name
        (bench_dir / "new").mkdir(parents=True)
        if benchmark is None:
            benchmark = json.dumps({"title": name, "full_id": name})
        (bench_dir / "new" / "benchmark.json").write_text(benchmark, encoding="utf-8")
        if change is not None:
            lower, estimate, upper = change
            interval = {"confidence_level": 0.95, "lower_bound": lower, "upper_bound": upper}
            stats = {"confidence_interval": interval, "point_estimate": estimate,
                     "standard_error": 0.01}
            estimates = json.dumps({"mean": stats, "median": stats})
        if estimates is not None:
            (bench_dir / "change").mkdir()
            (bench_dir / "change" / "estimates.json").write_text(estimates, encoding="utf-8")
        return bench_dir

    def run_main(self, *args):
        """Run main() with the directory and arguments, returning (status, output, summary)"""
        summary_path = self.home.parent / (self.home.name + "-summary.md")
        self.addCleanup(summary_path.unlink, missing_ok=True)
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            status = benchmark_report.main(
                [str(self.home), "--summary", str(summary_path), *args])
        summary = summary_path.read_text(encoding="utf-8") if summary_path.exists() else ""
        return status, out.getvalue(), summary


class ReadChangesTest(CriterionDirTest):
    """Reading Criterion's results"""

    def test_real_files(self):
        self.add_benchmark("fast-evaluations/Eval composite OLMT",
                           estimates=REAL_ESTIMATES, benchmark=REAL_BENCHMARK)
        [change] = benchmark_report.read_changes(self.home)
        self.assertEqual("fast-evaluations/Eval composite OLMT", change.name)
        self.assertAlmostEqual(26.61, change.lower, places=2)
        self.assertAlmostEqual(30.46, change.estimate, places=2)
        self.assertAlmostEqual(34.50, change.upper, places=2)

    def test_sorted_by_name(self):
        self.add_benchmark("group/b", change=(0, 0, 0))
        self.add_benchmark("group/a", change=(0, 0, 0))
        self.add_benchmark("group/c", change=(0, 0, 0))
        names = [c.name for c in benchmark_report.read_changes(self.home)]
        self.assertEqual(["group/a", "group/b", "group/c"], names)

    def test_benchmark_without_baseline_ignored(self):
        self.add_benchmark("group/new")
        self.add_benchmark("group/old", change=(0.01, 0.02, 0.03))
        names = [c.name for c in benchmark_report.read_changes(self.home)]
        self.assertEqual(["group/old"], names)

    def test_no_results(self):
        with self.assertRaisesRegex(ReportError, "No Criterion comparison results"):
            benchmark_report.read_changes(self.home)

    def test_only_benchmarks_without_baseline(self):
        self.add_benchmark("group/new")
        with self.assertRaisesRegex(ReportError, "No Criterion comparison results"):
            benchmark_report.read_changes(self.home)

    def test_invalid_json(self):
        self.add_benchmark("group/a", estimates="{not json")
        with self.assertRaisesRegex(ReportError, "Can't read"):
            benchmark_report.read_changes(self.home)

    def test_missing_fields(self):
        self.add_benchmark("group/a", estimates=json.dumps({"mean": {"point_estimate": 0.1}}))
        with self.assertRaisesRegex(ReportError, "Unexpected contents"):
            benchmark_report.read_changes(self.home)

    def test_missing_benchmark_file(self):
        bench_dir = self.add_benchmark("group/a", change=(0, 0, 0))
        (bench_dir / "new" / "benchmark.json").unlink()
        with self.assertRaisesRegex(ReportError, "Can't read"):
            benchmark_report.read_changes(self.home)


class ReportTest(unittest.TestCase):
    """Deciding what to report"""

    def test_regression_uses_lower_bound(self):
        self.assertTrue(Change("a", lower=10.5, estimate=15, upper=20).is_regression(10))
        self.assertFalse(Change("a", lower=9.5, estimate=15, upper=20).is_regression(10))
        self.assertFalse(Change("a", lower=10, estimate=15, upper=20).is_regression(10))

    def test_improvement_not_regression(self):
        self.assertFalse(Change("a", lower=-20, estimate=-15, upper=-10).is_regression(10))

    def test_warnings(self):
        # GitHub shows %25 in a workflow command's message as %
        changes = [Change("slower", 26.6, 30.5, 34.5), Change("same", -1, 0, 1)]
        self.assertEqual(
            ["::warning title=Benchmark regression::"
             "slower is 30.5%25 slower (95%25 interval +26.6%25 to +34.5%25)"],
            benchmark_report.warnings(changes, 10))

    def test_warning_escapes_name(self):
        [warning] = benchmark_report.warnings([Change("100%\nbad", 20, 20, 20)], 10)
        self.assertIn("100%25%0Abad", warning)

    def test_summary(self):
        changes = [Change("slower", 26.6, 30.5, 34.5), Change("faster", -16.7, -16.6, -16.4)]
        table = benchmark_report.summary(changes, 10).splitlines()
        self.assertIn("| slower | +30.5% | +26.6% to +34.5% | Slower |", table)
        self.assertIn("| faster | -16.6% | -16.7% to -16.4% |  |", table)

    def test_summary_escapes_pipes(self):
        table = benchmark_report.summary([Change("a|b", 0, 0, 0)], 10)
        self.assertIn("| a\\|b |", table)


class MainTest(CriterionDirTest):
    """The command line"""

    def test_regression(self):
        self.add_benchmark("group/slower", change=(0.266, 0.305, 0.345))
        self.add_benchmark("group/same", change=(-0.01, 0.0, 0.01))
        status, output, summary = self.run_main()
        self.assertEqual(0, status)
        self.assertEqual(1, output.count("::warning"))
        self.assertIn("group/slower is 30.5%25 slower", output)
        self.assertIn("| group/slower | +30.5% |", summary)
        self.assertIn("| group/same | +0.0% |", summary)

    def test_threshold(self):
        self.add_benchmark("group/a", change=(0.074, 0.077, 0.081))
        self.assertNotIn("::warning", self.run_main()[1])
        self.assertIn("::warning", self.run_main("--threshold-percent", "5")[1])

    def test_no_regressions(self):
        self.add_benchmark("group/a", change=(-0.05, -0.02, 0.01))
        status, output, summary = self.run_main()
        self.assertEqual(0, status)
        self.assertEqual("", output)
        self.assertIn("| group/a |", summary)

    def test_fail_on_regression(self):
        self.add_benchmark("group/slower", change=(0.266, 0.305, 0.345))
        self.add_benchmark("group/same", change=(-0.01, 0.0, 0.01))
        status, output, summary = self.run_main("--fail-on-regression")
        self.assertEqual(2, status)
        self.assertIn("::warning", output)
        self.assertIn("1 of 2 benchmarks regressed", output)
        # The summary is still written, to show what regressed
        self.assertIn("| group/slower | +30.5% |", summary)

    def test_fail_on_regression_without_regressions(self):
        self.add_benchmark("group/a", change=(0.05, 0.07, 0.09))
        status, output, _ = self.run_main("--fail-on-regression")
        self.assertEqual(0, status)
        self.assertEqual("", output)

    def test_without_fail_on_regression_succeeds(self):
        self.add_benchmark("group/slower", change=(0.266, 0.305, 0.345))
        self.assertEqual(0, self.run_main()[0])

    def test_no_results_is_error(self):
        status, output, summary = self.run_main()
        self.assertEqual(1, status)
        self.assertTrue(output.startswith("::error::No Criterion comparison results"))
        self.assertEqual("", summary)

    def test_summary_appended(self):
        self.add_benchmark("group/a", change=(0, 0, 0))
        summary_path = self.home.parent / (self.home.name + "-summary.md")
        summary_path.write_text("Earlier step\n", encoding="utf-8")
        _, _, summary = self.run_main()
        self.assertTrue(summary.startswith("Earlier step\n## Benchmarks"))

    def test_summary_to_stdout_without_file(self):
        self.add_benchmark("group/a", change=(0, 0, 0))
        out = io.StringIO()
        with contextlib.redirect_stdout(out), \
                unittest.mock.patch.dict("os.environ", {}, clear=True):
            status = benchmark_report.main([str(self.home)])
        self.assertEqual(0, status)
        self.assertIn("| group/a |", out.getvalue())


if __name__ == "__main__":
    unittest.main()
