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

def make_original_position(layout):
    """Create original position with coins"""
    pos = svg.StringsAndCoinsPosition(layout)
    other_chain = pos.add_horizontal_chain(5)
    pos.next_line()
    chain = pos.add_horizontal_chain(5)
    return [pos, chain, other_chain]

def original(layout, pos):
    """Render start position"""
    layout.add_default_text("Original position:")
    layout.move_below()
    pos.add_to_layout()
    layout.move_below()

def opened(layout, pos, chain):
    """Render first chain opening"""
    layout.add_default_text("Player $A$ must open a chain:")
    layout.move_below()
    pos.cut_ground_string(chain[-1])
    pos.highlight_pending_moves(colour="red")
    pos.add_to_layout()
    pos.make_pending_moves()
    layout.move_below()

def doubledeal(layout, pos, chain):
    """Render double-dealing move"""
    layout.add_default_text("Player $B$ double-deals:")
    layout.move_below()
    pos.cut_ground_string(chain[0])
    pos.cut_2coin_string(chain[1], chain[2])
    pos.cut_2coin_string(chain[2], chain[3])
    pos.cut_2coin_string(chain[3], chain[4])
    pos.highlight_pending_moves(colour="red")
    pos.add_to_layout()
    pos.make_pending_moves()
    layout.move_below()

def final(layout, pos, chain, other_chain):
    """Render final position"""
    layout.add_default_text("Player $A$ must now open the final chain:")
    layout.move_below()
    pos.cut_2coin_string(chain[0], chain[1])
    pos.cut_2coin_string(other_chain[3], other_chain[4])
    pos.highlight_pending_moves(colour="red")
    pos.add_to_layout()
    layout.move_below()
    layout.add_default_text("Player $B$ takes and wins 8--2.")

def main():
    """Entry point"""
    layout = svg.Layout()
    [pos, chain, other_chain] = make_original_position(layout)

    original(layout, pos)
    opened(layout, pos, chain)
    doubledeal(layout, pos, chain)
    final(layout, pos, chain, other_chain)

    print("<svg>")
    layout.render()
    print("</svg>")

if __name__ == "__main__":
    main()
