#!/usr/bin/env python3

"""Generate SVG diagram for P_{4,5}"""

import svg

def main():
    """Entry point"""
    pos = svg.StringsAndCoinsPosition()
    for _ in range(4):
        pos.add_horizontal_chain(3)
        pos.next_line()
    pos.add_horizontal_chain(5)
    print('<svg>')
    pos.render()
    print('</svg>')

if __name__ == '__main__':
    main()