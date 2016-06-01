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

"""Generate SVG diagram for Exercise 3.1 from Berlekamp's book"""

import svg

def main():
    """Entry point"""
    pos = svg.StringsAndCoinsPosition()
    grid = pos.create_dotsandboxes_start(3, 3)

    # Just make the moves in any order - it doesn't have to be sensible
    pos.cut_ground_string(grid[0][0], direction="up")
    pos.cut_ground_string(grid[0][0], direction="left")
    pos.cut_ground_string(grid[1][0], direction="up")
    pos.cut_ground_string(grid[0][1], direction="left")
    pos.cut_ground_string(grid[1][2], direction="down")
    pos.cut_ground_string(grid[2][2], direction="right")

    pos.cut_2coin_string(grid[1][0], grid[1][1])
    pos.cut_2coin_string(grid[2][0], grid[2][1])
    pos.cut_2coin_string(grid[0][1], grid[1][1])
    pos.cut_2coin_string(grid[0][2], grid[1][2])

    pos.make_pending_moves()

    pos.cut_2coin_string(grid[2][1], grid[2][2])
    pos.highlight_pending_moves(colour="black", thickness=3)

    pos.render()

if __name__ == '__main__':
    main()
