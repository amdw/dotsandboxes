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

def original(layout):
    """Render start position"""
    layout.add_default_text("Original position:")
    layout.move_below()
    pos = svg.StringsAndCoinsPosition(layout)
    pos.add_horizontal_chain(5)
    pos.next_line()
    pos.add_horizontal_chain(5)
    pos.next_line()
    pos.add_to_layout()
    layout.move_below()

def opened(layout):
    """Render first chain opening"""
    layout.add_default_text("Player $A$ must open a chain:")
    layout.move_below()
    pos = svg.StringsAndCoinsPosition(layout)
    pos.add_horizontal_chain(5)
    pos.next_line()
    coins = pos.add_horizontal_open_chain(5)
    pos.add_link(svg.GroundLink(coins[-1], "right", colour="lightgray", thickness=2))
    pos.add_to_layout()
    layout.move_below()

def doubledeal(layout):
    """Render double-dealing move"""
    layout.add_default_text("Player $B$ double-deals:")
    layout.move_below()
    pos = svg.StringsAndCoinsPosition(layout)
    pos.add_horizontal_chain(5)
    pos.next_line()
    demo_coins = pos.add_horizontal_unlinked_row(5, x_offset=layout.default_gap)
    highlight = {"colour": "lightgray", "thickness": 2}
    pos.add_horizontal_row_links(demo_coins, properties=[None, highlight, highlight, highlight])
    pos.add_link(svg.GroundLink(demo_coins[0], "left", **highlight))
    pos.add_to_layout()
    layout.move_below()

def final(layout):
    """Render final position"""
    layout.add_default_text("Player $A$ must now open the final chain:")
    layout.move_below()
    pos = svg.StringsAndCoinsPosition(layout)
    pos.add_horizontal_chain(5)
    pos.next_line()
    pos.add_horizontal_row(2, x_offset=layout.default_gap)
    pos.add_to_layout()
    layout.move_below()
    layout.add_default_text("Player $B$ wins 8--2.")

def main():
    """Entry point"""
    layout = svg.Layout()

    original(layout)
    opened(layout)
    doubledeal(layout)
    final(layout)

    print("<svg>")
    layout.render()
    print("</svg>")

if __name__ == "__main__":
    main()
