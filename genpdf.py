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

import os
import subprocess
import sys

AUX_EXTENSIONS = ["aux", "toc", "lof", "lot", "out"]

def read_aux_files(build_dir):
    """Return the contents of pdflatex's auxiliary files, keyed by extension"""
    contents = {}
    for ext in AUX_EXTENSIONS:
        try:
            with open(os.path.join(build_dir, "dotsandboxes." + ext), "rb") as aux_file:
                contents[ext] = aux_file.read()
        except FileNotFoundError:
            contents[ext] = None
    return contents

def main():
    """Entry point: build dotsandboxes.pdf in the directory given as the only argument, which
    must already contain the generated figures and vc.tex"""
    if len(sys.argv) != 2:
        sys.exit("Usage: {0} BUILD_DIR".format(sys.argv[0]))
    build_dir = sys.argv[1]
    # Run pdflatex until it no longer gives "re-run to get cross references right" output
    # and the auxiliary files (e.g. the table of contents) stop changing, but no more than
    # five times, to avoid an infinite loop. Not all changes produce the re-run message
    # (e.g. changing a section heading only changes the .toc file), hence the second check.
    # Earlier runs report undefined references which later runs resolve, so only show the
    # output of the final run (or of a run which fails), which is also kept in pdflatex.log.
    max_runs = 5
    command = ["pdflatex", "-interaction=nonstopmode", "-halt-on-error",
               "-output-directory=" + build_dir, "dotsandboxes.tex"]
    # Look for \input files and figures in the build directory, then the default search path
    env = dict(os.environ, TEXINPUTS=build_dir + os.pathsep)
    search_text = "Rerun to get cross-references right"
    log_filename = os.path.join(build_dir, "pdflatex.log")
    for runs in range(1, max_runs + 1):
        aux_before = read_aux_files(build_dir)
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                encoding="utf-8", errors="replace", check=False, env=env)
        with open(log_filename, "w", encoding="utf-8") as log_file:
            log_file.write(result.stdout)
        if result.returncode != 0:
            sys.stdout.write(result.stdout)
            raise Exception("pdflatex failed with status {0}".format(result.returncode))
        rerun_requested = search_text in result.stdout
        aux_changed = read_aux_files(build_dir) != aux_before
        if not rerun_requested and not aux_changed:
            sys.stdout.write(result.stdout)
            print("pdflatex success in {0} runs".format(runs))
            return
        reason = "cross-references" if rerun_requested else "auxiliary files"
        print("pdflatex run {0} done, re-running to get {1} right".format(runs, reason))

    sys.stdout.write(result.stdout)
    raise Exception("Output still not stable after {0} runs".format(max_runs))

if __name__ == '__main__':
    main()
