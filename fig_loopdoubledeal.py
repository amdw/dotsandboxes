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

"""Illustrate double-dealing move on a loop"""

import svg

def make_original_position(layout):
    """Create original position with loops"""
    pos = svg.StringsAndCoinsPosition(layout)
    other_grid = pos.add_horizontal_loop(10)
    pos.next_line()
    pos.next_line()
    grid = pos.add_horizontal_loop(10)
    return [pos, grid, other_grid]

def original(layout, pos):
    """Render original position"""
    layout.add_default_text("Original position:")
    layout.move_below()
    pos.add_to_layout()
    layout.move_below()

def opened(layout, pos, grid):
    """Render position with loop opened"""
    layout.add_default_text("Player $A$ must open a loop:")
    layout.move_below()
    pos.cut_2coin_string(grid[0][3], grid[0][4], pending=True)
    pos.highlight_pending_moves(colour="red")
    pos.add_to_layout()
    pos.make_pending_moves()
    layout.move_below()

def doubledeal(layout, pos, grid):
    """Render double-dealing move"""
    layout.add_default_text("Player $B$ double-deals:")
    layout.move_below()
    # Ends
    pos.cut_2coin_string(grid[0][0], grid[1][0], pending=True)
    pos.cut_2coin_string(grid[0][-1], grid[1][-1], pending=True)
    # Horizontals (leaving [0][0] to [0][1] and [1][0] to [1][1])
    pos.cut_2coin_string(grid[0][1], grid[0][2], pending=True)
    pos.cut_2coin_string(grid[0][2], grid[0][3], pending=True)
    pos.cut_2coin_string(grid[1][1], grid[1][2], pending=True)
    pos.cut_2coin_string(grid[1][2], grid[1][3], pending=True)
    pos.cut_2coin_string(grid[1][3], grid[1][4], pending=True)
    pos.highlight_pending_moves(colour="red")
    pos.add_to_layout()
    pos.make_pending_moves()
    layout.move_below()

def final(layout, pos, grid, other_grid):
    """Render final position"""
    layout.add_default_text("Player $A$ must now open the final loop:")
    layout.move_below()
    pos.cut_2coin_string(grid[0][0], grid[0][1], pending=True)
    pos.cut_2coin_string(grid[1][0], grid[1][1], pending=True)
    pos.cut_2coin_string(other_grid[0][3], other_grid[0][4], pending=True)
    pos.highlight_pending_moves(colour="red")
    pos.add_to_layout()
    layout.move_below()
    layout.add_default_text("Player $B$ wins 16--4.")

def main():
    """Entry point"""
    layout = svg.Layout()
    [pos, grid, other_grid] = make_original_position(layout)
    original(layout, pos)
    opened(layout, pos, grid)
    doubledeal(layout, pos, grid)
    final(layout, pos, grid, other_grid)

    print("<svg>")
    layout.render()
    print("</svg>")

if __name__ == "__main__":
    main()