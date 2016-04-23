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

"""Render SVG of example dots-and-boxes game"""

import svg

def main():
    """Entry point"""
    layout = svg.Layout(grid_width=4, grid_margin=100)
    pos = svg.DotsAndBoxesPosition(2, 2, layout=layout)
    pos.add_to_layout()
    layout.next_grid_position()
    pos.move_highlight_and_add(0, 0, "right")
    pos.move_highlight_and_add(1, 0, "right")
    pos.move_highlight_and_add(0, 0, "bottom")
    pos.move_highlight_and_add(0, 1, "right")
    pos.move_highlight_and_add(1, 1, "bottom")
    pos.move_highlight_and_add(0, 0, "left")
    pos.move_highlight_and_add(0, 0, "top")
    pos.move_highlight_and_add(0, 1, "left")
    pos.move_highlight_and_add(0, 1, "bottom")
    pos.move_highlight_and_add(1, 1, "right")
    pos.move_highlight_and_add(1, 1, "top")
    pos.move_highlight_and_add(1, 0, "top")
    layout.render()

if __name__ == "__main__":
    main()
