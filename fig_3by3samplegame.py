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

"""
Generate SVG diagram for sample game played according
to Berlekamp's winning 3x3 strategy
"""

import svg

def main():
    """Entry point"""
    layout = svg.Layout(grid_width=4, grid_margin=100)
    pos = svg.DotsAndBoxesPosition(3, 3, layout=layout)

    pos.move_highlight_and_add(1, 2, "bottom")
    pos.move_highlight_and_add(1, 2, "left")
    pos.move_highlight_and_add(1, 1, "top")
    pos.move_highlight_and_add(0, 1, "top")
    pos.move_highlight_and_add(2, 1, "top")
    pos.move_highlight_and_add(1, 0, "right")
    pos.move_highlight_and_add(2, 1, "right")
    pos.move_highlight_and_add(2, 1, "bottom")

    pos.move_highlight_and_add(2, 1, "left")
    pos.move_highlight_and_add(0, 0, "top")
    pos.move_highlight_and_add(0, 1, "left")
    pos.move_highlight_and_add(0, 2, "left")
    pos.move_highlight_and_add(2, 2, "bottom")

    pos.move_highlight_and_add(2, 0, "right")
    pos.move_highlight_and_add(2, 0, "top")
    pos.move_highlight_and_add(0, 0, "right")

    pos.move_highlight_and_add(0, 0, "left")
    pos.move_highlight_and_add(1, 0, "top")
    pos.move_highlight_and_add(0, 2, "bottom")
    pos.move_highlight_and_add(0, 2, "top")

    layout.render()

if __name__ == '__main__':
    main()
