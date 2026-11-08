#!/usr/bin/env python3

# Copyright 2016-2026 Andrew Medworth (github@medworth.org.uk)
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.

"""Generate PDF file"""

import subprocess
import sys

def main():
    """Entry point"""
    # Run pdflatex until it no longer gives "re-run to get cross references right" output,
    # but no more than five times, to avoid an infinite loop. Earlier runs report undefined
    # references which later runs resolve, so only show the output of the final run (or of
    # a run which fails), which is also kept in pdflatex.log.
    max_runs = 5
    command = ["pdflatex", "-interaction=nonstopmode", "-halt-on-error", "dotsandboxes.tex"]
    search_text = "Rerun to get cross-references right"
    log_filename = "pdflatex.log"
    for runs in range(1, max_runs + 1):
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                encoding="utf-8", errors="replace", check=False)
        with open(log_filename, "w", encoding="utf-8") as log_file:
            log_file.write(result.stdout)
        if result.returncode != 0:
            sys.stdout.write(result.stdout)
            raise Exception("pdflatex failed with status {0}".format(result.returncode))
        if search_text not in result.stdout:
            sys.stdout.write(result.stdout)
            print("pdflatex success in {0} runs".format(runs))
            return
        print("pdflatex run {0} done, re-running to get cross-references right".format(runs))

    sys.stdout.write(result.stdout)
    raise Exception("Still got cross-references warning after {0} runs".format(max_runs))

if __name__ == '__main__':
    main()
