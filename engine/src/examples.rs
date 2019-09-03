/*
    Copyright 2017-2019 Andrew Medworth <github@medworth.org.uk>

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
use game::{SimplePosition, Side, CompoundPosition};
use std::iter;

// Top of example from page 50 of Berlekamp's book
pub fn p50_top() -> SimplePosition {
    let mut pos = SimplePosition::new_game(5, 2);
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

// Bottom-left of example from page 50 of Berlekamp's book
pub fn p50_bottomleft() -> SimplePosition {
    let mut pos = SimplePosition::new_game(3, 2);
    pos.make_move(0, 0, Side::Top);
    pos.make_move(1, 0, Side::Top);
    pos.make_move(2, 0, Side::Top);
    pos.make_move(2, 0, Side::Right);
    pos.make_move(2, 1, Side::Right);
    pos.make_move(1, 0, Side::Bottom);
    pos
}

// Bottom-right of example from page 50 of Berlekamp's book
pub fn p50_bottomright() -> SimplePosition {
    let mut pos = SimplePosition::new_game(2, 2);
    pos.make_move(0, 0, Side::Top);
    pos.make_move(1, 0, Side::Top);
    pos.make_move(0, 0, Side::Left);
    pos.make_move(0, 1, Side::Left);
    pos
}

// Example from page 50 of Berlekamp's book
pub fn p50() -> SimplePosition {
    let mut pos = SimplePosition::new_game(5, 4);
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

pub fn icelandic_game(width: usize, height: usize) -> SimplePosition {
    let mut pos = SimplePosition::new_game(width, height);
    for y in 0..height {
        pos.make_move(0, y, Side::Left);
    }
    for x in 0..width {
        pos.make_move(x, 0, Side::Top);
    }
    pos
}

// Construct a position consisting of a given number of equally-sized chains.
pub fn multi_chains(chain_size: usize, chain_count: usize) -> SimplePosition {
    let mut pos = SimplePosition::new_game(chain_size, chain_count);
    for x in 0..chain_size {
        pos.make_move(x, 0, Side::Top);
        for y in 0..chain_count {
            pos.make_move(x, y, Side::Bottom);
        }
    }
    pos
}
// Create a position consisting of a single horizontal chain of a given length.
pub fn make_chain(length: usize) -> SimplePosition {
    multi_chains(length, 1)
}

// Create a position consisting of two horizontal chains of a given length.
pub fn double_chain(length: usize) -> SimplePosition {
    multi_chains(length, 2)
}

// Create a position consisting of two loops of a given width.
pub fn double_loop(width: usize) -> SimplePosition {
    let mut pos = SimplePosition::new_game(width, 4);
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
pub fn ex3p1() -> SimplePosition {
    let mut pos = SimplePosition::new_game(3, 3);
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
pub fn ex3p12() -> SimplePosition {
    let mut pos = SimplePosition::new_game(5, 5);
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
pub fn ex6p2() -> SimplePosition {
    let mut pos = SimplePosition::new_game(3, 2);
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
pub fn ex7p2() -> SimplePosition {
    let mut pos = SimplePosition::new_game(5, 5);
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

// Construct the one-long-chain multiple-three-chain positions from the paper.
// The long chain is the 0th part, the 3-chains parts 1-n.
pub fn one_long_multi_three(three_chain_count: usize, long_chain_size: usize) -> CompoundPosition {
    let mut parts: Vec<SimplePosition> = Vec::with_capacity(three_chain_count + 1);
    parts.push(make_chain(long_chain_size));
    parts.extend(iter::repeat(make_chain(3)).take(three_chain_count));
    CompoundPosition::new_game(parts)
}

#[cfg(test)]
mod tests {
    use examples::*;
    use game::*;

    #[test]
    fn test_olmt_indep() {
        let mut pos = one_long_multi_three(2, 5);
        assert_eq!(14, pos.legal_moves().len());
        // Just to assert that the repeated components are really cloned/independent
        pos.make_move(CPosMove::new(1, 0, 0, Side::Right));
        assert_eq!(13, pos.legal_moves().len());
    }
}