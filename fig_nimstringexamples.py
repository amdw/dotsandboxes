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

"""Generate SVG diagram for Nimstring value examples"""

import svg

TEXT_GAP = 0.7

def add_single_string_examples(layout):
    """The simplest"""
    pos = svg.StringsAndCoinsPosition(layout)
    pos.add_horizontal_open_chain(1)
    pos.add_to_layout()
    layout.add_default_text("$0$", y_gap_offset=TEXT_GAP)
    layout.next_grid_position()

    pos = svg.StringsAndCoinsPosition(layout)
    coins = pos.add_horizontal_unlinked_row(2)
    pos.add_horizontal_row_links(coins)
    pos.add_to_layout()
    layout.add_default_text("$0$", y_gap_offset=TEXT_GAP)
    layout.next_grid_position()

def add_two_string_examples(layout):
    """Add examples with two strings"""
    pos = svg.StringsAndCoinsPosition(layout)
    pos.add_horizontal_open_chain(2)
    pos.add_to_layout()
    layout.add_default_text("$\\loony$", y_gap_offset=TEXT_GAP)
    layout.next_grid_position()

    pos = svg.StringsAndCoinsPosition(layout)
    coins = pos.add_horizontal_unlinked_row(3)
    pos.add_horizontal_row_links(coins)
    pos.add_to_layout()
    layout.add_default_text("$0$", y_gap_offset=TEXT_GAP)
    layout.next_grid_position()

def add_chain_examples(layout):
    """Add chain examples"""
    pos = svg.StringsAndCoinsPosition(layout)
    pos.add_horizontal_chain(2)
    pos.add_to_layout()
    layout.add_default_text("$*1$", y_gap_offset=TEXT_GAP)
    layout.next_grid_position()

    pos = svg.StringsAndCoinsPosition(layout)
    pos.add_horizontal_chain(3)
    pos.add_to_layout()
    layout.add_default_text("$0$", y_gap_offset=TEXT_GAP)
    layout.next_grid_position()

def main():
    """Entry point"""
    layout = svg.Layout(grid_width=3, min_grid_column=150)
    add_single_string_examples(layout)
    add_two_string_examples(layout)
    add_chain_examples(layout)
    layout.render()

if __name__ == '__main__':
    main()
