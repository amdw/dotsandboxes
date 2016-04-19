# Copyright 2016 Andrew Medworth (github@medworth.org.uk)
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
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
            y = 1
        elif direction == 'down':
            y = -1
        else:
            raise ValueError('Illegal direction "{0}"'.format(self.direction))
        return [x, y]

    def arrow_points(self):
        """Positions of the arrow tip and 'feathers'"""
        [dir_x, dir_y] = self.dir_vector()

        point_x = self.coin.x + (self.length * dir_x)
        point_y = self.coin.y + (self.length * dir_y)

        f1_x = point_x - (self.f_size * (dir_x if dir_x != 0 else 0.5))
        f1_y = point_y - (self.f_size * (dir_y if dir_y != 0 else 0.5))
        f2_x = point_x - (self.f_size * (dir_x if dir_x != 0 else -0.5))
        f2_y = point_y - (self.f_size * (dir_y if dir_y != 0 else -0.5))

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
                 default_line_colour="black", default_fill_colour="white"):
        self.coins = []
        self.links = []
        self.other_elements = []
        self.default_coin_r = default_coin_r
        self.default_thickness = default_thickness
        self.default_gap = default_gap
        self.default_line_colour = default_line_colour
        self.default_fill_colour = default_fill_colour
        self.x_base = 10
        self.y_base = 10

    def move_below(self):
        """Move the y base below anything drawn so far"""
        if not self.coins and not self.other_elements:
            return
        max_elt_y = max([c.y for c in self.coins] + [c.y for c in self.other_elements])
        self.y_base = max_elt_y + self.default_gap

        # If any coins at the bottom have downward-pointing ground links, need to move a bit further
        lowest_coins = [c for c in self.coins if c.y == max_elt_y]
        down_ground_links = [l for l in self.links for c in lowest_coins
                             if l.is_link_to_ground(c) and l.direction == "down"]
        if down_ground_links:
            self.y_base += self.default_gap

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
        # Render links first, so the coins go on top
        for link in self.links:
            link.render(to)
        for coin in self.coins:
            coin.render(to)
        for elem in self.other_elements:
            elem.render(to)

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

    def next_line(self):
        """Move relative position to the next line"""
        self.y_pos += self.layout.default_gap

    def move_to_right(self):
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
        return row1_coins + row2_coins

    def create_dotsandboxes_start(self, width, height):
        """Lay out a dots-and-boxes position of the given dimensions"""
        grid = []
        for _i in range(height):
            grid.append(self.add_horizontal_chain(width))
            self.next_line()
        for coin in grid[0]:
            link = self.layout.make_default_glink(coin, "up")
            self.add_link(link)
        for coin in grid[-1]:
            link = self.layout.make_default_glink(coin, "down")
            self.add_link(link)
        return grid

    def _check_captures(self):
        """
        Check to see if any coins were captured by the last move; if so,
        remove them and update the score and player to move
        """
        captured = []
        for coin in self.coins:
            links = [l for l in self.links if l.is_link_to(coin)]
            if not links:
                captured.append(coin)
                if self.player_to_move == "A":
                    self.a_score += 1
                elif self.player_to_move == "B":
                    self.b_score += 1
        if not captured:
            self.player_to_move = "A" if self.player_to_move == "B" else "B"
        for coin in captured:
            self.coins.remove(coin)

    def make_pending_moves(self):
        """Make any moves which are queued up"""
        for link in self.pending_moves:
            self.links.remove(link)
        self.pending_moves = []
        self._check_captures()

    def highlight_pending_moves(self, colour="lightgray", thickness=2):
        """Change the visual attributes of pending moves so they show up clearly"""
        for link in self.pending_moves:
            link.colour = colour
            link.thickness = thickness

    def cut_2coin_string(self, coin1, coin2, pending=False):
        """
        Make a move by cutting a string connecting two coins.
        Setting pending=True will not make the move but will queue it up for later.
        It is assumed that all pending moves will be by the same player.
        """
        links = [l for l in self.links if l.is_link_between(coin1, coin2)]
        if not links:
            raise ValueError("Position contains no link between {0} and {1}".format(coin1, coin2))
        self.pending_moves.extend(links)
        if not pending:
            self.make_pending_moves()

    def cut_ground_string(self, coin, pending=False):
        """
        Cut a string connecting a coin to the ground.
        Setting pending=True will not make the move but will queue it up for later.
        It is assumed that all pending moves will be by the same player.
        """
        links = [l for l in self.links if l.is_link_to_ground(coin)]
        if not links:
            raise ValueError("Position contains no link from {0} to ground".format(coin))
        self.pending_moves.extend(links)
        if not pending:
            self.make_pending_moves()

    def add_to_layout(self):
        """Add elements to the given layout."""
        # Copy the elements so the layout can take ownership
        new_coins = dict(zip(self.coins, [copy.copy(c) for c in self.coins]))
        new_links = [copy.copy(l).replace_coins(new_coins) for l in self.links]
        for coin in new_coins.values():
            self.layout.add_coin(coin)
        for link in new_links:
            self.layout.add_link(link)

    def render(self, to=sys.stdout):
        """Shortcut method to render layout when it contains only one position"""
        self.add_to_layout()
        self.layout.render(to)
