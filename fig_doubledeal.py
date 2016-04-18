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

"""Generate SVG diagram for double-dealing demo"""

import svg

def original(pos):
    """Render start position"""
    pos.add_default_text("Original position:")
    pos.next_line()
    pos.add_horizontal_chain(5)
    pos.next_line()
    pos.add_horizontal_chain(5)
    pos.next_line()

def opened(pos):
    """Render first chain opening"""
    pos.add_default_text("Player $A$ must open a chain:")
    pos.next_line()
    pos.add_horizontal_chain(5)
    pos.next_line()
    coins = pos.add_horizontal_open_chain(5)
    pos.add_link(svg.GroundLink(coins[-1], "right", colour="lightgray", thickness=2))
    pos.next_line()

def doubledeal(pos):
    """Render double-dealing move"""
    pos.add_default_text("Player $B$ double-deals:")
    pos.next_line()
    pos.add_horizontal_chain(5)
    pos.next_line()
    demo_coins = pos.add_horizontal_unlinked_row(5, x_offset=pos.default_gap)
    highlight = {"colour": "lightgray", "thickness": 2}
    pos.add_horizontal_row_links(demo_coins, properties=[None, highlight, highlight, highlight])
    pos.add_link(svg.GroundLink(demo_coins[0], "left", **highlight))
    pos.next_line()

def final(pos):
    """Render final position"""
    pos.add_default_text("Player $A$ must now open the final chain:")
    pos.next_line()
    pos.add_horizontal_chain(5)
    pos.next_line()
    pos.add_horizontal_row(2, x_offset=pos.default_gap)
    pos.next_line()
    pos.add_default_text("Player $B$ wins 8--2.")

def main():
    """Entry point"""
    pos = svg.StringsAndCoinsPosition()

    original(pos)
    opened(pos)
    doubledeal(pos)
    final(pos)

    print("<svg>")
    pos.render()
    print("</svg>")

if __name__ == "__main__":
    main()
