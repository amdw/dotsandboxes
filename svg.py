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

class StringsAndCoinsPosition:
    """Representation of a strings-and-coins position"""
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
        self.default_x = 10
        self.default_y = 10

    def next_line(self):
        """Move default y value to the next line"""
        self.default_y += self.default_gap

    def add_coin(self, coin):
        """Add a coin to the position"""
        self.coins.append(coin)

    def default_coin(self, x, y):
        """Create coin with default size, colour etc"""
        return Coin(x, y,
                    r=self.default_coin_r,
                    colour=self.default_fill_colour,
                    line_colour=self.default_line_colour,
                    thickness=self.default_thickness)

    def add_link(self, link):
        """Add a link to the position"""
        self.links.append(link)

    def add_default_2clink(self, coin1, coin2):
        """Add a two-coin link with default colour, thickness etc"""
        self.add_link(TwoCoinLink(coin1, coin2,
                                  colour=self.default_line_colour,
                                  thickness=self.default_thickness))

    def add_default_glink(self, coin, direction):
        """Add a ground link with default colour, thickness etc"""
        self.add_link(GroundLink(coin, direction,
                                 length=self.default_gap,
                                 colour=self.default_line_colour,
                                 thickness=self.default_thickness))

    def add_other_element(self, elem):
        """Add another renderable element"""
        self.other_elements.append(elem)

    def add_default_text(self, text, x=None, y=None):
        """Add text with default colour etc"""
        if x is None:
            x = self.default_x
        if y is None:
            y = self.default_y
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

    def add_horizontal_unlinked_row(self, num_coins, x_offset=0, y=None):
        """Add a horizontal row of unlinked coins"""
        if y is None:
            y = self.default_y
        coins = []
        for i in range(num_coins):
            coin = self.default_coin(self.default_x + (i * self.default_gap) + x_offset, y)
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

    def add_horizontal_row(self, num_coins, x_offset=0, y=None):
        """Add a horizontal row of linked coins"""
        coins = self.add_horizontal_unlinked_row(num_coins, x_offset=x_offset, y=y)
        self.add_horizontal_row_links(coins)
        return coins

    def add_horizontal_chain(self, num_coins):
        """Add a horizontal chain of a specified number of coins"""
        coins = self.add_horizontal_row(num_coins, self.default_gap)
        self.add_default_glink(coins[0], "left")
        self.add_default_glink(coins[-1], "right")
        return coins

    def add_horizontal_open_chain(self, num_coins):
        """Add a horizontal chain open at the right end"""
        coins = self.add_horizontal_row(num_coins, self.default_gap)
        self.add_default_glink(coins[0], "left")
        return coins

    def add_horizontal_loop(self, num_coins):
        """Add a horizontal loop with a specified number of coins"""
        if num_coins % 2 != 0:
            raise ValueError("Only even coin counts are currently supported: {0}".format(num_coins))
        row_length = int(num_coins / 2)
        row1_coins = self.add_horizontal_row(row_length, y=self.default_y)
        row2_coins = self.add_horizontal_row(row_length, y=self.default_y + self.default_gap)
        self.add_default_2clink(row1_coins[0], row2_coins[0])
        self.add_default_2clink(row1_coins[-1], row2_coins[-1])
