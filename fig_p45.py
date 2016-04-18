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
