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
Python functions to support the creation of SVG vector graphics
files representing strings-and-coins positions.
"""

import copy
import sys

def render_tag(tag, attribs, to, content=None):
    """Print an XML tag"""
    attrib_text = ' '.join(['{0}="{1}"'.format(k, attribs[k]) for k in sorted(attribs)])
    out = '<' + tag + ' ' + attrib_text
    if content:
        out += '>' + content + '</' + tag + '>'
    else:
        out += '/>'
    print(out, file=to)

class Coin:
    """Representation of a coin"""
    def __init__(self, x, y, r=10, colour="white", line_colour="black", thickness=1):
        self.x = x
        self.y = y
        self.r = r
        self.colour = colour
        self.line_colour = line_colour
        self.thickness = thickness

    def render(self, to=sys.stdout):
        """Render this coin as SVG"""
        attribs = {"cx": self.x, "cy": self.y, "r": self.r,
                   "stroke": self.line_colour, "stroke-width": self.thickness,
                   "fill": self.colour}
        render_tag('circle', attribs, to)

class TwoCoinLink:
    """Link between two coins"""
    def __init__(self, coin1, coin2, colour="black", thickness=1):
        self.coin1 = coin1
        self.coin2 = coin2
        self.colour = colour
        self.thickness = thickness

    def is_link_to(self, coin):
        """Indicate whether this is a link to a coin"""
        return self.coin1 == coin or self.coin2 == coin

    def is_link_between(self, coin1, coin2):
        """Indicate whether this is a link between two coins, order-independent"""
        match1 = (self.coin1 == coin1 and self.coin2 == coin2)
        match2 = (self.coin2 == coin1 and self.coin1 == coin2)
        return match1 or match2

    # pylint: disable=R0201
    def is_link_to_ground(self, _coin):
        """Two-coin links never link to ground"""
        return False

    def all_coins(self):
        """All coins we link to"""
        return [self.coin1, self.coin2]

    def replace_coins(self, new_coins):
        """Replace coins with new ones from input map"""
        self.coin1 = new_coins[self.coin1]
        self.coin2 = new_coins[self.coin2]
        return self

    def render(self, to=sys.stdout):
        """Render link as SVG"""
        attribs = {"x1": self.coin1.x, "y1": self.coin1.y,
                   "x2": self.coin2.x, "y2": self.coin2.y,
                   "stroke": self.colour, "stroke-width": self.thickness}
        render_tag('line', attribs, to)

class GroundLink:
    """Link between a coin and the ground"""
    def __init__(self, coin, direction, length=50, colour="black", thickness=1,
                 f_size=None):
        self.coin = coin
        self.direction = direction
        self.length = length
        self.colour = colour
        self.thickness = thickness
        self.f_size = f_size if f_size else max(1, int(length / 5))

    def is_link_to(self, coin):
        """Indicate whether this is a link to a coin"""
        return self.coin == coin

    # pylint: disable=R0201
    def is_link_between(self, _coin1, _coin2):
        """Ground links never link two coins"""
        return False

    def is_link_to_ground(self, coin):
        """Indicate whether this links a given coin to the ground"""
        return self.coin == coin

    def all_coins(self):
        """All coins we link to"""
        return [self.coin]

    def replace_coins(self, new_coins):
        """Replace coin with new one from input map"""
        self.coin = new_coins[self.coin]
        return self

    def dir_vector(self):
        """Unit vector pointing in the direction of the arrow"""
        direction = self.direction.lower()
        x = 0
        y = 0
        if direction == 'left':
            x = -1
        elif direction == 'right':
            x = 1
        elif direction == 'up':
            y = -1
        elif direction == 'down':
            y = 1
        else:
            raise ValueError('Illegal direction "{0}"'.format(self.direction))
        return [x, y]

    def arrow_points(self):
        """Positions of the arrow tip and 'feathers'"""
        [dir_x, dir_y] = self.dir_vector()

        point_x = self.coin.x + (self.length * dir_x)
        point_y = self.coin.y + (self.length * dir_y)

        f1_x = int(point_x - (self.f_size * (dir_x if dir_x != 0 else 0.5)))
        f1_y = int(point_y - (self.f_size * (dir_y if dir_y != 0 else 0.5)))
        f2_x = int(point_x - (self.f_size * (dir_x if dir_x != 0 else -0.5)))
        f2_y = int(point_y - (self.f_size * (dir_y if dir_y != 0 else -0.5)))

        return [point_x, point_y, f1_x, f1_y, f2_x, f2_y]

    def render(self, to=sys.stdout):
        """Render link as SVG"""
        [arrow_x, arrow_y, f1_x, f1_y, f2_x, f2_y] = self.arrow_points()
        attribs = {"x1": self.coin.x, "y1": self.coin.y,
                   "x2": arrow_x, "y2": arrow_y,
                   "stroke": self.colour, "stroke-width": self.thickness}
        render_tag("line", attribs, to)
        attribs["x1"] = f1_x
        attribs["y1"] = f1_y
        render_tag("line", attribs, to)
        attribs["x1"] = f2_x
        attribs["y1"] = f2_y
        render_tag("line", attribs, to)

class TextElement:
    """A text element to be placed in the output SVG"""
    def __init__(self, text, x, y, colour="black"):
        self.text = text
        self.x = x
        self.y = y
        self.colour = colour

    def render(self, to=sys.stdout):
        """Render text as SVG"""
        attribs = {"x": self.x, "y": self.y, "fill": self.colour}
        render_tag("text", attribs, to, self.text)

class Layout:
    """
    Representation of a layout on the page. Can have a mixture of
    elements, including dots-and-boxes and strings-and-coins
    positions.

    This class knows how to lay out elements on a SVG diagram, but
    not about the logic of the game.
    """
    def __init__(self, default_coin_r=10, default_thickness=1, default_gap=50,
                 default_line_colour="black", default_fill_colour="white",
                 grid_width=1):
        self.coins = []
        self.links = []
        self.dots = []
        self.lines = []
        self.completed_boxes = []
        self.other_elements = []
        self.default_coin_r = default_coin_r
        self.default_thickness = default_thickness
        self.default_gap = default_gap
        self.default_line_colour = default_line_colour
        self.default_fill_colour = default_fill_colour
        self.x_base = 10
        self.y_base = 10
        self.grid_width = grid_width
        self.current_grid_x = 0
        self.grid_left_xs = [self.x_base]

    def _all_coord_elements(self):
        """All elements which have direct coordinates specified"""
        return self.coins + self.dots + self.other_elements

    def move_below(self):
        """Move the y base below anything drawn so far"""
        all_coord_elements = self._all_coord_elements()
        if not all_coord_elements:
            return
        max_elt_y = max([e.y for e in all_coord_elements])
        self.y_base = max_elt_y + self.default_gap

        # If any coins at the bottom have downward-pointing ground links, need to move a bit further
        lowest_coins = [c for c in self.coins if c.y == max_elt_y]
        down_ground_links = [l for l in self.links for c in lowest_coins
                             if l.is_link_to_ground(c) and l.direction == "down"]
        if down_ground_links:
            self.y_base += int(self.default_gap * 0.3)

    def move_right(self):
        """Move the x to the right of anything drawn so far"""
        all_coord_elements = self._all_coord_elements()
        if not all_coord_elements:
            return
        max_elt_x = max([e.x for e in all_coord_elements])
        self.x_base = max_elt_x + self.default_gap

        # If any coins at the right have rightward-pointing ground links, need to move a bit further
        rightmost_coins = [c for c in self.coins if c.x == max_elt_x]
        right_ground_links = [l for l in self.links for c in rightmost_coins
                              if l.is_link_to_ground(c) and l.direction == "right"]
        if right_ground_links:
            self.x_base += self.default_gap

    def next_grid_position(self):
        """Move the coordinates to the next position on the grid"""
        self.current_grid_x += 1
        if self.current_grid_x >= self.grid_width:
            # New row
            self.current_grid_x = 0
            self.move_below()

        # Have we been in this column before? If so, use the same
        if len(self.grid_left_xs) > self.current_grid_x:
            self.x_base = self.grid_left_xs[self.current_grid_x]
        else:
            self.move_right()
            self.grid_left_xs.append(self.x_base)

    def add_coin(self, coin):
        """
        Add a coin to the layout.
        Coordinates on the coin are expected to be relative to our own base.
        This is also expected to hand over ownership of the coin to the layout,
        so it can be modified.
        """
        coin.x += self.x_base
        coin.y += self.y_base
        self.coins.append(coin)

    def add_link(self, link):
        """Add a link to the position"""
        self.links.append(link)

    def add_dot(self, dot):
        """
        Add a dot to the layout. Coordinates expected to be relative.
        Hands over ownership of the dot to the layout.
        """
        dot.x += self.x_base
        dot.y += self.y_base
        self.dots.append(dot)

    def add_line(self, line):
        """Add a line to the layout."""
        self.lines.append(line)

    def add_completed_box(self, box):
        """Add a completed box to the layout."""
        self.completed_boxes.append(box)

    def add_other_element(self, elem):
        """
        Add another renderable element.
        Coordinates are expected to be relative to our own base.
        This is also expected to hand over ownership of the element to the layout,
        so it can be modified.
        """
        elem.x += self.x_base
        elem.y += self.y_base
        self.other_elements.append(elem)

    def make_default_coin(self, x, y):
        """Create coin with default size, colour etc"""
        return Coin(x, y,
                    r=self.default_coin_r,
                    colour=self.default_fill_colour,
                    line_colour=self.default_line_colour,
                    thickness=self.default_thickness)

    def make_default_2clink(self, coin1, coin2):
        """Create a two-coin link with default colour, thickness etc"""
        return TwoCoinLink(coin1, coin2,
                           colour=self.default_line_colour,
                           thickness=self.default_thickness)

    def make_default_glink(self, coin, direction):
        """Create a ground link with default colour, thickness etc"""
        return GroundLink(coin, direction,
                          length=self.default_gap,
                          colour=self.default_line_colour,
                          thickness=self.default_thickness)

    def add_default_text(self, text, x=None, y=None):
        """Add text with default colour etc"""
        if x is None:
            x = 0
        if y is None:
            y = 0
        self.add_other_element(TextElement(text, x, y, colour=self.default_line_colour))

    def render(self, to=sys.stdout):
        """Render position as SVG"""
        print("<svg>", file=to)
        # Render links first, so the coins go on top
        for link in self.links:
            link.render(to)
        for coin in self.coins:
            coin.render(to)
        # Render lines before dots
        for line in self.lines:
            line.render(to)
        for dot in self.dots:
            dot.render(to)
        for box in self.completed_boxes:
            box.render(to)
        for elem in self.other_elements:
            elem.render(to)
        print("</svg>", file=to)

class StringsAndCoinsPosition:
    """
    Represent a strings-and-coins position.

    Knows the logic of the game, and also how to lay out the elements
    relative to one another (but not in absolute terms on the layout).
    """
    def __init__(self, layout=None):
        self.coins = []
        self.links = []
        self.player_to_move = "A"
        self.a_score = 0
        self.b_score = 0
        self.layout = layout if layout else Layout()
        self.x_pos = 0
        self.y_pos = 0
        self.pending_moves = []

    def next_line(self, small=False):
        """Move relative position to the next line"""
        self.y_pos += int(self.layout.default_gap * (0.7 if small else 1))

    def move_right(self):
        """Move relative position to right hand side of whatever we already have"""
        if not self.coins:
            return
        self.x_pos = max([c.x for c in self.coins]) + self.layout.default_gap

    def add_coin(self, coin):
        """Add a coin to the position"""
        self.coins.append(coin)

    def add_link(self, link):
        """Add a link to the position"""
        self.links.append(link)

    def add_default_glink(self, coin, direction):
        """Add a ground link with default colour, thickness etc"""
        self.add_link(self.layout.make_default_glink(coin, direction))

    def add_default_2clink(self, coin1, coin2):
        """Add a two-coin link with default colour, thickness etc"""
        self.add_link(self.layout.make_default_2clink(coin1, coin2))

    def add_horizontal_unlinked_row(self, num_coins, x_offset=0, y_offset=0):
        """Add a horizontal row of unlinked coins"""
        coins = []
        for i in range(num_coins):
            x = self.x_pos + (i * self.layout.default_gap) + x_offset
            y = self.y_pos + y_offset
            coin = self.layout.make_default_coin(x, y)
            coins.append(coin)
            self.add_coin(coin)
        return coins

    def add_horizontal_row_links(self, coins, properties=None):
        """Add links to a horizontal row"""
        if not properties:
            properties = [None] * len(coins)
        for i in range(len(coins) - 1):
            link_properties = properties[i]
            if link_properties:
                link = TwoCoinLink(coins[i], coins[i+1], **link_properties)
                self.add_link(link)
            else:
                self.add_default_2clink(coins[i], coins[i+1])

    def add_horizontal_row(self, num_coins, x_offset=0, y_offset=0):
        """Add a horizontal row of linked coins"""
        coins = self.add_horizontal_unlinked_row(num_coins, x_offset=x_offset, y_offset=y_offset)
        self.add_horizontal_row_links(coins)
        return coins

    def add_horizontal_open_chain(self, num_coins):
        """Add a horizontal chain open at the right end"""
        coins = self.add_horizontal_row(num_coins, self.layout.default_gap)
        self.add_default_glink(coins[0], "left")
        return coins

    def add_horizontal_chain(self, num_coins):
        """Add a horizontal chain of a specified number of coins"""
        coins = self.add_horizontal_open_chain(num_coins)
        self.add_default_glink(coins[-1], "right")
        return coins

    def add_horizontal_loop(self, num_coins):
        """Add a horizontal loop with a specified number of coins"""
        if num_coins % 2 != 0:
            raise ValueError("Only even coin counts are currently supported: {0}".format(num_coins))
        row_length = int(num_coins / 2)
        row1_coins = self.add_horizontal_row(row_length)
        row2_coins = self.add_horizontal_row(row_length, y_offset=self.layout.default_gap)
        self.add_default_2clink(row1_coins[0], row2_coins[0])
        self.add_default_2clink(row1_coins[-1], row2_coins[-1])
        return [row1_coins, row2_coins]

    def create_dotsandboxes_start(self, width, height):
        """Lay out a dots-and-boxes position of the given dimensions"""
        grid = []
        self.next_line() # Make room for the upward ground links
        for i in range(height):
            grid.append(self.add_horizontal_chain(width))
            if i > 0:
                for j in range(width):
                    link = self.layout.make_default_2clink(grid[i-1][j], grid[i][j])
                    self.add_link(link)
            self.next_line()
        for coin in grid[0]:
            link = self.layout.make_default_glink(coin, "up")
            self.add_link(link)
        for coin in grid[-1]:
            link = self.layout.make_default_glink(coin, "down")
            self.add_link(link)
        self.next_line(small=True) # To clear the downward links
        return grid

    def _check_captures(self):
        """
        Check to see if any coins were captured by the last move; if so,
        remove them and update the score and player to move
        """
        captured = self._would_capture()
        if self.player_to_move == "A":
            self.a_score += len(captured)
        elif self.player_to_move == "B":
            self.b_score += len(captured)

        # The turn is completed iff at least one of the pending moves wasn't a capture
        non_captures = [l for l in self.pending_moves
                        if not [c for c in l.all_coins() if c in captured]]
        if non_captures:
            self.player_to_move = "A" if self.player_to_move == "B" else "B"

        for coin in captured:
            self.coins.remove(coin)

    def _would_capture(self):
        """
        Check which coins would be captured by the current list of pending moves.
        """
        links_after = copy.copy(self.links)
        for link in self.pending_moves:
            links_after.remove(link)

        captured = []
        for coin in self.coins:
            coin_links = [l for l in links_after if l.is_link_to(coin)]
            if not coin_links:
                captured.append(coin)

        return captured

    def make_pending_moves(self):
        """Make any moves which are queued up"""
        if not self.pending_moves:
            return

        self._check_captures()
        for link in self.pending_moves:
            self.links.remove(link)

        self.pending_moves = []

    def highlight_pending_moves(self, colour, thickness):
        """Change the visual attributes of pending moves so they show up clearly"""
        for link in self.pending_moves:
            link.colour = colour
            link.thickness = thickness
        captured = self._would_capture()
        for coin in captured:
            coin.line_colour = colour
            coin.thickness = thickness

    def cut_2coin_string(self, coin1, coin2):
        """
        Make a move by cutting a string connecting two coins.
        This will not make the move but will queue it up for later.
        make_pending_moves() must be called before starting to make moves for another player.
        """
        links = [l for l in self.links if l.is_link_between(coin1, coin2)]
        if not links:
            raise ValueError("Position contains no link between {0} and {1}".format(coin1, coin2))
        self.pending_moves.append(links[0])

    def cut_ground_string(self, coin, direction=None):
        """
        Cut a string connecting a coin to the ground.
        This will not make the move but will queue it up for later.
        make_pending_moves() must be called before starting to make moves for another player.
        """
        links = [l for l in self.links if l.is_link_to_ground(coin)
                 and (direction is None or l.direction.lower() == direction.lower())]
        if not links:
            raise ValueError("Position contains no such link from {0} to ground".format(coin))
        self.pending_moves.append(links[0])

    def add_to_layout(self):
        """Add elements to the given layout."""
        # Copy the elements so the layout can take ownership
        new_coins = dict(zip(self.coins, [copy.copy(c) for c in self.coins]))
        new_links = [copy.copy(l).replace_coins(new_coins) for l in self.links]
        for coin in new_coins.values():
            self.layout.add_coin(coin)
        for link in new_links:
            self.layout.add_link(link)

    def highlight_add_and_move(self, colour="black", thickness=3):
        """
        Highlight pending moves, add the position to the layout, and move to
        the next grid position.
        """
        self.highlight_pending_moves(colour=colour, thickness=thickness)
        self.add_to_layout()
        player_label = self.player_to_move
        self.make_pending_moves()
        score_text = "$${0}$$, {1}--{2}".format(player_label, self.a_score, self.b_score)
        self.layout.add_default_text(score_text, y=self.y_pos)
        self.layout.next_grid_position()

    def render(self, to=sys.stdout):
        """Shortcut method to render layout when it contains only one position"""
        self.add_to_layout()
        self.layout.render(to)

class Dot:
    """A dot element from dots-and-boxes"""
    def __init__(self, x, y, colour="black", r=3):
        self.x = x
        self.y = y
        self.colour = colour
        self.r = r

    def render(self, to):
        """Render as SVG"""
        attribs = {"cx": self.x, "cy": self.y, "r": self.r,
                   "stroke": self.colour, "stroke-width": 1,
                   "fill": self.colour}
        render_tag("circle", attribs, to)

class Line:
    """A line element from dots-and-boxes"""
    def __init__(self, dot1, dot2, colour="black", thickness=1):
        self.dot1 = dot1
        self.dot2 = dot2
        self.colour = colour
        self.thickness = thickness

    def replace_dots(self, new_dots):
        """Replace with copied dots"""
        self.dot1 = new_dots[self.dot1]
        self.dot2 = new_dots[self.dot2]
        return self

    def render(self, to):
        """Render as SVG"""
        attribs = {"x1": self.dot1.x, "y1": self.dot1.y,
                   "x2": self.dot2.x, "y2": self.dot2.y,
                   "stroke": self.colour, "stroke-width": self.thickness}
        render_tag("line", attribs, to)

class CompletedBox:
    """A completed box in dots-and-boxes"""
    def __init__(self, tl_dot, tr_dot, bl_dot, player, colour="black"):
        self.tl_dot = tl_dot
        self.tr_dot = tr_dot
        self.bl_dot = bl_dot
        self.player = player
        self.colour = colour

    def replace_dots(self, new_dots):
        """Replace dot reference with copy"""
        self.tl_dot = new_dots[self.tl_dot]
        self.tr_dot = new_dots[self.tr_dot]
        self.bl_dot = new_dots[self.bl_dot]
        return self

    def render(self, to):
        """Render as SVG"""
        x = int((self.tl_dot.x + self.tr_dot.x) / 2)
        y = int((self.tl_dot.y + self.bl_dot.y) / 2)
        attribs = {"x": x, "y": y, "fill": self.colour}
        render_tag("text", attribs, to, content=self.player)

class DotsAndBoxesPosition:
    """Dots-and-boxes position which renders itself as such on the layout"""

    def __init__(self, width, height, layout=None):
        self.layout = layout if layout else Layout()
        self.dots = []
        for i in range(width + 1):
            column = []
            for j in range(height + 1):
                dot = Dot(i * self.layout.default_gap, j * self.layout.default_gap,
                          colour=self.layout.default_line_colour)
                column.append(dot)
            self.dots.append(column)
        self.valencies = []
        for i in range(width):
            self.valencies.append([4] * height)
        self.lines = []
        self.completed_boxes = []
        self.player_to_move = "A"
        self.a_score = 0
        self.b_score = 0

    def _dots_for_move(self, x, y, direction):
        """Return pair of dots connected by a move"""
        direction = direction.lower()
        if direction == "top":
            return [self.dots[x][y], self.dots[x+1][y]]
        elif direction == "bottom":
            return [self.dots[x][y+1], self.dots[x+1][y+1]]
        elif direction == "left":
            return [self.dots[x][y], self.dots[x][y+1]]
        elif direction == "right":
            return [self.dots[x+1][y], self.dots[x+1][y+1]]
        else:
            raise ValueError("Illegal direction [{0}]".format(direction))

    def _affected_boxes(self, x, y, direction):
        """The one or two boxes affected by a given move"""
        affected_boxes = [[x, y]]
        direction = direction.lower()
        if direction == "top":
            if y > 0:
                affected_boxes.append([x, y-1])
        elif direction == "bottom":
            if y < len(self.valencies[0]) - 1:
                affected_boxes.append([x, y+1])
        elif direction == "left":
            if x > 0:
                affected_boxes.append([x-1, y])
        elif direction == "right":
            if x < len(self.valencies) - 1:
                affected_boxes.append([x+1, y])
        else:
            raise ValueError("Illegal direction [{0}]".format(direction))

        return affected_boxes

    def _check_captures(self, x, y, direction):
        """Check if a move captured any boxes (and update valencies)"""
        affected_boxes = self._affected_boxes(x, y, direction)
        end_of_turn = True
        for [box_x, box_y] in affected_boxes:
            self.valencies[box_x][box_y] -= 1
            if self.valencies[box_x][box_y] <= 0:
                completed = CompletedBox(self.dots[x][y], self.dots[x+1][y],
                                         self.dots[x][y+1], self.player_to_move)
                self.completed_boxes.append(completed)
                if self.player_to_move == "A":
                    self.a_score += 1
                elif self.player_to_move == "B":
                    self.b_score += 1
                end_of_turn = False

        if end_of_turn:
            self.player_to_move = "B" if self.player_to_move == "A" else "A"

    def make_move(self, x, y, direction):
        """Draw a line on the board"""
        [dot1, dot2] = self._dots_for_move(x, y, direction)
        line = Line(dot1, dot2, self.layout.default_line_colour, 1)
        self.lines.append(line)
        self._check_captures(x, y, direction)

    def add_to_layout(self):
        """Add all elements to the layout"""
        dots_list = [dot for col in self.dots for dot in col]
        new_dots = dict(zip(dots_list, [copy.copy(dot) for dot in dots_list]))
        new_lines = [copy.copy(line).replace_dots(new_dots) for line in self.lines]
        new_boxes = [copy.copy(box).replace_dots(new_dots) for box in self.completed_boxes]
        for line in new_lines:
            self.layout.add_line(line)
        for dot in new_dots.values():
            self.layout.add_dot(dot)
        for box in new_boxes:
            self.layout.add_completed_box(box)

    def move_highlight_and_add(self, x, y, direction):
        """Make a move, highlight it, and add to the layout"""
        self.make_move(x, y, direction)
        self.add_to_layout()
        self.layout.next_grid_position()
