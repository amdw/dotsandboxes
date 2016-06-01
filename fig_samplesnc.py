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

"""Generate SVG diagram for sample strings-and-coins game"""

import svg

def main():
    """Entry point"""
    layout = svg.Layout(grid_width=4)
    pos = svg.StringsAndCoinsPosition(layout)
    grid = pos.create_dotsandboxes_start(2, 2)
    pos.highlight_add_and_move()

    pos.cut_2coin_string(grid[0][0], grid[1][0])
    pos.highlight_add_and_move()

    pos.cut_ground_string(grid[1][0], direction="right")
    pos.highlight_add_and_move()

    pos.cut_2coin_string(grid[0][0], grid[0][1])
    pos.highlight_add_and_move()

    pos.cut_2coin_string(grid[0][1], grid[1][1])
    pos.highlight_add_and_move()

    pos.cut_ground_string(grid[1][1], direction="down")
    pos.highlight_add_and_move()

    pos.cut_ground_string(grid[0][0], direction="left")
    pos.highlight_add_and_move()

    pos.cut_ground_string(grid[0][0], direction="up")
    pos.highlight_add_and_move()

    pos.cut_ground_string(grid[0][1], direction="left")
    pos.highlight_add_and_move()

    pos.cut_ground_string(grid[0][1], direction="down")
    pos.highlight_add_and_move()

    pos.cut_ground_string(grid[1][1], direction="right")
    pos.highlight_add_and_move()

    pos.cut_2coin_string(grid[1][0], grid[1][1])
    pos.highlight_add_and_move()

    pos.cut_ground_string(grid[1][0], direction="up")
    pos.highlight_add_and_move()

    layout.render()

if __name__ == "__main__":
    main()
