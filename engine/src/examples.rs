/*
    Copyright 2017-2018 Andrew Medworth <github@medworth.org.uk>

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

pub fn icelandic_game(width: usize, height: usize) -> Position {
    let mut pos = Position::new_game(width, height);
    for y in 0..height {
        pos.make_move(0, y, Side::Left);
    }
    for x in 0..width {
        pos.make_move(x, 0, Side::Top);
    }
    pos
}

// Construct a position consisting of a given number of equally-sized chains.
pub fn multi_chains(chain_size: usize, chain_count: usize) -> Position {
    let mut pos = Position::new_game(chain_size, chain_count);
    for x in 0..chain_size {
        pos.make_move(x, 0, Side::Top);
        for y in 0..chain_count {
            pos.make_move(x, y, Side::Bottom);
        }
    }
    pos
}
// Create a position consisting of a single horizontal chain of a given length.
pub fn make_chain(length: usize) -> Position {
    multi_chains(length, 1)
}

// Create a position consisting of two horizontal chains of a given length.
pub fn double_chain(length: usize) -> Position {
    multi_chains(length, 2)
}

// Create a position consisting of two loops of a given width.
pub fn double_loop(width: usize) -> Position {
    let mut pos = Position::new_game(width, 4);
    for i in 0..width {
        pos.make_move(i, 0, Side::Top);
        pos.make_move(i, 1, Side::Bottom);
        pos.make_move(i, 3, Side::Bottom);
    }
    for i in 0..4 {
        pos.make_move(0, i, Side::Left);
        pos.make_move(width-1, i, Side::Right);
    }
    for i in 1..width-1 {
        pos.make_move(i, 0, Side::Bottom);
        pos.make_move(i, 2, Side::Bottom);
    }
    pos
}

// Create Exercise 3.1 from Berlekamp's book
pub fn ex3p1() -> Position {
    let mut pos = Position::new_game(3, 3);
    pos.make_move(0, 0, Side::Top);
    pos.make_move(0, 0, Side::Left);
    pos.make_move(1, 0, Side::Top);
    pos.make_move(1, 0, Side::Bottom);
    pos.make_move(2, 0, Side::Bottom);
    pos.make_move(0, 1, Side::Left);
    pos.make_move(0, 1, Side::Right);
    pos.make_move(0, 2, Side::Right);
    pos.make_move(1, 2, Side::Bottom);
    pos.make_move(2, 2, Side::Right);
    pos
}

// Create Exercise 3.12 from Berlekamp's book
pub fn ex3p12() -> Position {
    let mut pos = Position::new_game(5, 5);
    for i in 0..5 {
        pos.make_move(i, 0, Side::Top);
    }
    pos.make_move(0, 0, Side::Bottom);
    pos.make_move(2, 0, Side::Right);
    pos.make_move(0, 1, Side::Left);
    pos.make_move(1, 1, Side::Bottom);
    pos.make_move(1, 1, Side::Right);
    pos.make_move(2, 1, Side::Right);
    pos.make_move(3, 1, Side::Bottom);
    pos.make_move(4, 1, Side::Bottom);
    pos.make_move(0, 2, Side::Right);
    pos.make_move(0, 2, Side::Bottom);
    pos.make_move(2, 2, Side::Right);
    pos.make_move(2, 2, Side::Bottom);
    pos.make_move(4, 2, Side::Right);
    pos.make_move(0, 3, Side::Right);
    pos.make_move(2, 3, Side::Bottom);
    pos.make_move(3, 3, Side::Bottom);
    pos.make_move(3, 3, Side::Right);
    pos.make_move(4, 3, Side::Right);
    pos.make_move(0, 4, Side::Right);
    pos.make_move(1, 4, Side::Right);
    pos.make_move(3, 4, Side::Bottom);
    pos.make_move(4, 4, Side::Bottom);
    pos.make_move(4, 4, Side::Right);
    pos
}

// Create Exercise 6.2 from Berlekamp's book
pub fn ex6p2() -> Position {
    let mut pos = Position::new_game(3, 2);
    for x in 0..3 {
        pos.make_move(x, 0, Side::Top);
    }
    for y in 0..2 {
        pos.make_move(2, y, Side::Right);
    }
    pos.make_move(1, 0, Side::Bottom);
    pos
}

// Create Exercise 7.2 from Berlekamp's book
pub fn ex7p2() -> Position {
    let mut pos = Position::new_game(5, 5);
    pos.make_move(0, 0, Side::Right);
    pos.make_move(1, 0, Side::Right);
    pos.make_move(3, 0, Side::Right);
    pos.make_move(0, 1, Side::Left);
    pos.make_move(0, 1, Side::Right);
    pos.make_move(3, 1, Side::Right);
    pos.make_move(1, 1, Side::Bottom);
    pos.make_move(2, 1, Side::Bottom);
    pos.make_move(3, 1, Side::Bottom);
    pos.make_move(0, 2, Side::Bottom);
    pos.make_move(0, 2, Side::Right);
    pos.make_move(2, 2, Side::Bottom);
    pos.make_move(3, 2, Side::Right);
    pos.make_move(4, 2, Side::Right);
    pos.make_move(1, 3, Side::Right);
    pos.make_move(3, 3, Side::Right);
    pos.make_move(3, 3, Side::Bottom);
    pos.make_move(1, 4, Side::Right);
    pos.make_move(2, 4, Side::Right);
    pos
}