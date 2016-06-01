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

"""Generate SVG diagram for Berlekamp's winning 3x3 strategy"""

import svg

def main():
    """Entry point"""
    layout = svg.Layout(default_thickness=3)
    pos = svg.DotsAndBoxesPosition(3, 3, layout)

    pos.make_move(0, 0, "bottom")
    pos.make_move(0, 2, "right")
    pos.make_move(2, 2, "top")
    pos.make_move(2, 0, "left")

    pos.render()

if __name__ == '__main__':
    main()
