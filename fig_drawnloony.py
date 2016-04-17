#!/usr/bin/env python3

"""Generate drawn loony endgame position"""

import svg

def main():
    """Entry point"""
    pos = svg.StringsAndCoinsPosition()
    pos.add_horizontal_loop(4)
    pos.default_x += 2 * pos.default_gap
    pos.add_horizontal_loop(4)
    print('<svg>')
    pos.render()
    print('</svg>')

if __name__ == "__main__":
    main()
