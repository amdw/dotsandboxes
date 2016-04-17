"""
Python functions to support the creation of SVG vector graphics
files representing strings-and-coins positions.
"""

import sys

def render_tag(tag, attribs, to):
    """Print an XML tag"""
    attrib_text = ' '.join(['{0}="{1}"'.format(k, attribs[k]) for k in sorted(attribs)])
    out = '<' + tag + ' ' + attrib_text + '/>'
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

class StringsAndCoinsPosition:
    """Representation of a strings-and-coins position"""
    def __init__(self, default_coin_r=10, default_thickness=1, default_gap=50,
                 default_line_colour="black", default_fill_colour="white"):
        self.coins = []
        self.links = []
        self.default_coin_r = default_coin_r
        self.default_thickness = default_thickness
        self.default_gap = default_gap
        self.default_line_colour = default_line_colour
        self.default_fill_colour = default_fill_colour
        self.default_x = 10
        self.default_y = 10

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

    def render(self, to=sys.stdout):
        """Render position as SVG"""
        # Render links first, so the coins go on top
        for link in self.links:
            link.render(to)
        for coin in self.coins:
            coin.render(to)

    def add_horizontal_row(self, num_coins, x_offset=0, y=None):
        """Add a horizontal row of linked coins"""
        if y is None:
            y = self.default_y
        coins = []
        for i in range(num_coins):
            coin = self.default_coin(self.default_x + (i * self.default_gap) + x_offset, y)
            coins.append(coin)
            self.add_coin(coin)
            if i > 0:
                self.add_default_2clink(coins[i-1], coin)
        return coins

    def add_horizontal_chain(self, num_coins):
        """Add a horizontal chain of a specified number of coins"""
        coins = self.add_horizontal_row(num_coins, self.default_gap)
        self.add_default_glink(coins[0], "left")
        self.add_default_glink(coins[-1], "right")

    def add_horizontal_loop(self, num_coins):
        """Add a horizontal loop with a specified number of coins"""
        if num_coins % 2 != 0:
            raise ValueError("Only even coin counts are currently supported: {0}".format(num_coins))
        row_length = int(num_coins / 2)
        row1_coins = self.add_horizontal_row(row_length, y=self.default_y)
        row2_coins = self.add_horizontal_row(row_length, y=self.default_y + self.default_gap)
        self.add_default_2clink(row1_coins[0], row2_coins[0])
        self.add_default_2clink(row1_coins[-1], row2_coins[-1])
