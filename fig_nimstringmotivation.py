#!/usr/bin/env python3

# Copyright 2016-2017 Andrew Medworth (github@medworth.org.uk)
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

import svg

def make_position():
    """Build the position - separate function for reuse in the solution"""
    pos = svg.StringsAndCoinsPosition()
    grid = pos.create_dotsandboxes_start(5, 4)

    # Just make the moves in any order - it doesn't have to be sensible
    pos.cut_ground_string(grid[0][0], direction="up")
    pos.cut_ground_string(grid[0][0], direction="left")
    pos.cut_ground_string(grid[1][0], direction="up")
    pos.cut_ground_string(grid[0][1], direction="left")

    pos.cut_2coin_string(grid[1][0], grid[1][1])
    pos.cut_2coin_string(grid[2][0], grid[2][1])
    pos.cut_2coin_string(grid[3][0], grid[3][1])
    pos.cut_2coin_string(grid[3][0], grid[4][0])

    pos.cut_2coin_string(grid[0][1], grid[0][2])
    pos.cut_2coin_string(grid[1][1], grid[1][2])
    pos.cut_2coin_string(grid[2][1], grid[2][2])
    pos.cut_2coin_string(grid[3][1], grid[3][2])
    pos.cut_2coin_string(grid[4][1], grid[4][2])

    pos.cut_2coin_string(grid[1][2], grid[1][3])
    pos.cut_2coin_string(grid[2][2], grid[3][2])
    pos.cut_2coin_string(grid[2][3], grid[3][3])

    pos.make_pending_moves()
    return (pos, grid)

def main():
    """Entry point"""
    (pos, _grid) = make_position()
    pos.render()

if __name__ == '__main__':
    main()
