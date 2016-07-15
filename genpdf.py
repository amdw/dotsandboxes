#!/usr/bin/env python3

# Copyright 2016 Andrew Medworth (github@medworth.org.uk)
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

def main():
    """Entry point"""
    # Run pdflatex until it no longer gives "re-run to get cross references right" output
    # But no more than five times, to avoid an infinite loop
    runs = 0
    max_runs = 5
    success = False
    command = "pdflatex -halt-on-error dotsandboxes.tex"
    search_text = "Rerun to get cross-references right"
    log_filename = "pdflatex.log"
    while runs < max_runs:
        runs += 1
        subprocess.check_call("{0} | tee {1}".format(command, log_filename), shell=True)
        grep_result = subprocess.call(["grep", search_text, log_filename])
        if grep_result not in [0, 1]:
            raise Exception("grep failed with status {0}".format(grep_result))
        if grep_result == 1:
            success = True
            break

    if not success:
        raise Exception("Still got cross-references warning after {0} runs".format(max_runs))

    print("pdflatex success in {0} runs".format(runs))

if __name__ == '__main__':
    main()
