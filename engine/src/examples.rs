/*
    Copyright 2017 Andrew Medworth <github@medworth.org.uk>

    This file is part of Dots-and-Boxes Engine.

    Dots-and-Boxes Engine is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Dots-and-Boxes Engine is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
along with Dots-and-Boxes Engine.  If not, see <http://www.gnu.org/licenses/>.
*/
use game::{Position, Side};

pub fn p50_top() -> Position {
    let mut pos = Position::new_game(5, 2);
    pos.make_move(0, 0, Side::Top);
    pos.make_move(1, 0, Side::Top);
    pos.make_move(0, 0, Side::Left);
    pos.make_move(0, 1, Side::Left);
    pos.make_move(1, 0, Side::Bottom);
    pos.make_move(2, 0, Side::Bottom);
    pos.make_move(3, 0, Side::Bottom);
    pos.make_move(3, 0, Side::Right);
    for i in 0..5 {
        pos.make_move(i, 1, Side::Bottom);
    }
    pos
}

pub fn p50_bottomleft() -> Position {
    let mut pos = Position::new_game(3, 2);
    pos.make_move(0, 0, Side::Top);
    pos.make_move(1, 0, Side::Top);
    pos.make_move(2, 0, Side::Top);
    pos.make_move(2, 0, Side::Right);
    pos.make_move(2, 1, Side::Right);
    pos.make_move(1, 0, Side::Bottom);
    pos
}

pub fn p50_bottomright() -> Position {
    let mut pos = Position::new_game(2, 2);
    pos.make_move(0, 0, Side::Top);
    pos.make_move(1, 0, Side::Top);
    pos.make_move(0, 0, Side::Left);
    pos.make_move(0, 1, Side::Left);
    pos
}

pub fn p50() -> Position {
    let mut pos = Position::new_game(5, 4);
    pos.make_move(0, 0, Side::Top);
    pos.make_move(0, 0, Side::Left);
    pos.make_move(1, 0, Side::Top);
    pos.make_move(1, 0, Side::Bottom);
    pos.make_move(2, 0, Side::Bottom);
    pos.make_move(3, 0, Side::Bottom);
    pos.make_move(3, 0, Side::Right);
    pos.make_move(0, 1, Side::Left);
    pos.make_move(0, 1, Side::Bottom);
    pos.make_move(1, 1, Side::Bottom);
    pos.make_move(2, 1, Side::Bottom);
    pos.make_move(3, 1, Side::Bottom);
    pos.make_move(4, 1, Side::Bottom);
    pos.make_move(1, 2, Side::Bottom);
    pos.make_move(2, 2, Side::Right);
    pos.make_move(2, 3, Side::Right);
    pos
}

// Create a position consisting of a single horizontal chain of a given length.
pub fn make_chain(length: usize) -> Position {
    let mut pos = Position::new_game(length, 1);
    for i in 0..length {
        pos.make_move(i, 0, Side::Top);
        pos.make_move(i, 0, Side::Bottom);
    }
    pos
}
