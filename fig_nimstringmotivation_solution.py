#!/usr/bin/env python3

# Copyright 2017 Andrew Medworth (github@medworth.org.uk)
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

"""Generate SVG diagram for Figure 25 from page 50 of Berlekamp's book"""

import fig_nimstringmotivation

def main():
    """Entry point"""
    (pos, grid) = fig_nimstringmotivation.make_position()
    pos.cut_ground_string(grid[0][3], direction="down")
    pos.cut_ground_string(grid[0][3], direction="left")
    pos.cut_2coin_string(grid[0][3], grid[1][3])
    pos.highlight_pending_moves(colour="black", thickness=3)
    pos.render()

if __name__ == '__main__':
    main()
