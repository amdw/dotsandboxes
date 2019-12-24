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
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::fmt;
use std::iter;

#[derive(Clone)]
#[derive(Copy)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
#[derive(Ord)]
#[derive(Debug)]
pub enum Side {
    Top, Bottom, Left, Right
}

impl Side {
    pub fn all() -> Vec<Side> {
        vec!(Side::Top, Side::Bottom, Side::Left, Side::Right)
    }

    pub fn all_except(side: Side) -> Vec<Side> {
        let mut result = Vec::with_capacity(3);
        for s in Side::all() {
            if s != side {
                result.push(s);
            }
        }
        result
    }

    pub fn opposite(self: &Side) -> Side {
        match self {
            &Side::Left => Side::Right,
            &Side::Right => Side::Left,
            &Side::Top => Side::Bottom,
            &Side::Bottom => Side::Top,
        }
    }
}

impl fmt::Display for Side {
    fn fmt(self: &Side, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Side::Top => write!(f, "Top"),
            &Side::Bottom => write!(f, "Bottom"),
            &Side::Left => write!(f, "Left"),
            &Side::Right => write!(f, "Right"),
        }
    }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub struct MoveOutcome {
    pub coins_captured: usize,
    pub end_of_turn: bool,
    pub end_of_game: bool,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
#[derive(Debug)]
pub struct Move {
    pub x: usize,
    pub y: usize,
    pub side: Side,
}

impl Move {
    pub fn new(x: usize, y: usize, side: Side) -> Move {
        Move{x: x, y: y, side: side}
    }
}

impl fmt::Display for Move {
    fn fmt(self: &Move, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}) {}", self.x, self.y, self.side)
    }
}

pub trait Position<M> {
    fn is_legal_move(&self, m: M) -> bool;
    fn would_capture(&self, m: M) -> usize;
    fn make_move(&mut self, m: M) -> MoveOutcome;
    fn undo_move(&mut self, m: M);
    fn is_end_of_game(&self) -> bool;
    fn legal_moves(&self) -> Vec<M>;
    fn zhash(&self) -> usize;
}

// An m*n dots-and-boxes position is represented as:
// * a row of top ground links
// * a column of left ground links
// * an m*n array of downward-pointing links
// * an m*n array of rightward-pointing links
// Position coordinates originate at the top left and are 0-based,
// so x=1,y=2 is the second square in the third row.
#[derive(Clone)]
pub struct SimplePosition {
    top_strings: Vec<bool>,
    left_strings: Vec<bool>,
    down_strings: Vec<Vec<bool>>,
    right_strings: Vec<Vec<bool>>,
    zhash: ZHash,
}

impl SimplePosition {
    // Create a new dots-and-boxes position of a given size.
    pub fn new_game(width: usize, height: usize) -> SimplePosition {
        SimplePosition::make_position(width, height, true)
    }

    // Create a new dots-and-boxes position of a given size but with all moves completed.
    pub fn new_end_game(width: usize, height: usize) -> SimplePosition {
        SimplePosition::make_position(width, height, false)
    }

    fn make_position(width: usize, height: usize, init_string: bool) -> SimplePosition {
        let top_strings = iter::repeat(init_string).take(width).collect();
        let left_strings = iter::repeat(init_string).take(height).collect();
        let mut right_strings = Vec::with_capacity(width);
        let mut down_strings = Vec::with_capacity(width);
        for _ in 0..width {
            right_strings.push(iter::repeat(init_string).take(height).collect());
            down_strings.push(iter::repeat(init_string).take(height).collect());
        }
        SimplePosition {
            top_strings: top_strings,
            left_strings: left_strings,
            down_strings: down_strings,
            right_strings: right_strings,
            zhash: ZHash::new(width, height),
        }
    }

    pub fn width(self: &SimplePosition) -> usize {
        self.top_strings.len()
    }

    pub fn height(self: &SimplePosition) -> usize {
        self.left_strings.len()
    }

    // Indicate whether a given square has been captured.
    pub fn is_captured(self: &SimplePosition, x: usize, y: usize) -> bool {
        !self.is_legal_move(Move{x: x, y: y, side: Side::Left}) &&
            !self.is_legal_move(Move{x: x, y: y, side: Side::Right}) &&
            !self.is_legal_move(Move{x: x, y: y, side: Side::Top}) &&
            !self.is_legal_move(Move{x: x, y: y, side: Side::Bottom})
    }

    // Valency or degree of coin at a given position
    pub fn valency(self: &SimplePosition, x: usize, y: usize) -> usize {
        let mut result = 0;
        for s in Side::all() {
            if self.is_legal_move(Move{x: x, y: y, side: s}) {
                result += 1
            }
        }
        result
    }

    // Move from the square indicated in the direction indicated by the side,
    // returning a result only if that square is still on the board
    pub fn offset(self: &SimplePosition, x: usize, y: usize, s: Side) -> Option<(usize, usize)> {
        match (x, y, s) {
            (0, _, Side::Left) => None,
            (x, _, Side::Right) if x == self.width()-1 => None,
            (_, 0, Side::Top) => None,
            (_, y, Side::Bottom) if y == self.height()-1 => None,
            (x, y, Side::Left) => Some((x-1, y)),
            (x, y, Side::Right) => Some((x+1, y)),
            (x, y, Side::Top) => Some((x, y-1)),
            (x, y, Side::Bottom) => Some((x, y+1)),
        }
    }

    // Indicate whether two moves are equivalent in the current position,
    // i.e. they refer to the same "logical" move on the board.
    // For example (0,0) Right is equivalent to (1,0) Left
    // (provided the board is at least two columns wide).
    pub fn moves_equivalent(self: &SimplePosition, move1: Move, move2: Move) -> bool {
        if move1 == move2 {
            true
        }
        else if let Some((n1x, n1y)) = self.offset(move1.x, move1.y, move1.side) {
            move2.x == n1x && move2.y == n1y && move2.side == move1.side.opposite()
        }
        else {
            false
        }
    }
}

impl Position<Move> for SimplePosition {
    // Indicate whether a given move is legal in the current position.
    fn is_legal_move(self: &SimplePosition, m: Move) -> bool {
        if m.x >= self.down_strings.len() {
            return false;
        }
        if m.y >= self.left_strings.len() {
            return false;
        }
        match (m.x, m.y, m.side) {
            (0, y, Side::Left) => self.left_strings[y],
            (x, 0, Side::Top) => self.top_strings[x],
            (x, y, Side::Top) => self.down_strings[x][y-1],
            (x, y, Side::Bottom) => self.down_strings[x][y],
            (x, y, Side::Left) => self.right_strings[x-1][y],
            (x, y, Side::Right) => self.right_strings[x][y],
        }
    }

    // Indicate how many coins a given move would capture (either 0, 1 or 2)
    fn would_capture(self: &SimplePosition, m: Move) -> usize {
        let mut result = 0;
        if self.valency(m.x, m.y) == 1 {
            result += 1;
        }
        if let Some((nx, ny)) = self.offset(m.x, m.y, m.side) {
            if self.valency(nx, ny) == 1 {
                result += 1;
            }
        }
        result
    }

    // Make a given move on the board, and indicate the outcome.
    fn make_move(self: &mut SimplePosition, m: Move) -> MoveOutcome {
        if !self.is_legal_move(m) {
            panic!(format!("Illegal move {}, pos:\n{}", m, self));
        }
        match (m.x, m.y, m.side) {
            (0, y, Side::Left) => self.left_strings[y] = false,
            (x, 0, Side::Top) => self.top_strings[x] = false,
            (x, y, Side::Top) => self.down_strings[x][y-1] = false,
            (x, y, Side::Bottom) => self.down_strings[x][y] = false,
            (x, y, Side::Left) => self.right_strings[x-1][y] = false,
            (x, y, Side::Right) => self.right_strings[x][y] = false,
        }
        let mut captures = if self.is_captured(m.x, m.y) { 1 } else { 0 };
        if m.side == Side::Left && m.x > 0 {
            if self.is_captured(m.x-1, m.y) {
                captures += 1
            }
        }
        if m.side == Side::Right && m.x < self.top_strings.len()-1 {
            if self.is_captured(m.x+1, m.y) {
                captures += 1
            }
        }
        if m.side == Side::Top && m.y > 0 {
            if self.is_captured(m.x, m.y-1) {
                captures += 1
            }
        }
        if m.side == Side::Bottom && m.y < self.left_strings.len()-1 {
            if self.is_captured(m.x, m.y+1) {
                captures += 1
            }
        }
        let end_of_game = self.is_end_of_game();
        self.zhash.toggle_element(m);
        MoveOutcome {
            coins_captured: captures,
            end_of_turn: captures == 0 || end_of_game,
            end_of_game: end_of_game,
        }
    }

    // Undo a given move by putting the line back on the board.
    // Behaviour if the move was never made in the first place is undefined.
    fn undo_move(self: &mut SimplePosition, m: Move) {
        match (m.x, m.y, m.side) {
            (0, y, Side::Left) => self.left_strings[y] = true,
            (x, 0, Side::Top) => self.top_strings[x] = true,
            (x, y, Side::Top) => self.down_strings[x][y-1] = true,
            (x, y, Side::Bottom) => self.down_strings[x][y] = true,
            (x, y, Side::Left) => self.right_strings[x-1][y] = true,
            (x, y, Side::Right) => self.right_strings[x][y] = true,
        }
        self.zhash.toggle_element(m);
    }

    // Indicate whether the game is over (i.e. whether all strings have been cut).
    fn is_end_of_game(self: &SimplePosition) -> bool {
        for &b in self.left_strings.iter() {
            if b {
                return false;
            }
        }
        for &b in self.top_strings.iter() {
            if b {
                return false;
            }
        }
        for row in self.down_strings.iter() {
            for &b in row {
                if b {
                    return false;
                }
            }
        }
        for col in self.right_strings.iter() {
            for &b in col {
                if b {
                    return false;
                }
            }
        }
        true
    }

    // Compute all possible legal moves in the position.
    fn legal_moves(self: &SimplePosition) -> Vec<Move> {
        let mut result: Vec<Move> = Vec::new();
        for (x, &b) in self.top_strings.iter().enumerate() {
            if b {
                result.push(Move{ x: x, y: 0, side: Side::Top });
            }
        }
        for (y, &b) in self.left_strings.iter().enumerate() {
            if b {
                result.push(Move{ x: 0, y: y, side: Side::Left });
            }
        }
        for x in 0..self.top_strings.len() {
            for y in 0..self.left_strings.len() {
                if self.down_strings[x][y] {
                    result.push(Move{ x: x, y: y, side: Side::Bottom });
                }
                if self.right_strings[x][y] {
                    result.push(Move{ x: x, y: y, side: Side::Right })
                }
            }
        }
        result
    }

    // Current Zobrist hash value for position.
    // This hash should be consistent across positions,
    // i.e. equal positions should have equal hashes.
    fn zhash(self: &SimplePosition) -> usize {
        self.zhash.current_value()
    }
}

impl PartialEq for SimplePosition {
    fn eq(self: &SimplePosition, other: &SimplePosition) -> bool {
        if self.width() != other.width() || self.height() != other.height() {
            return false;
        }
        for x in 0..self.width() {
            for y in 0..self.height() {
                for s in Side::all() {
                    let legal_here = self.is_legal_move(Move{x: x, y: y, side: s});
                    let legal_there = other.is_legal_move(Move{x: x, y: y, side: s});
                    if legal_here != legal_there {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl Eq for SimplePosition {}

impl fmt::Display for SimplePosition {
    fn fmt(self: &SimplePosition, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "  ")?;
        for i in 0..self.width() {
            write!(f, " {}", i % 10)?;
        }
        write!(f, "\n  ")?;
        for &b in self.top_strings.iter() {
            write!(f, "+{}", if b { " " } else { "-" })?;
        }
        write!(f, "+\n")?;
        for j in 0..self.left_strings.len() {
            write!(f, "{} {}", j % 10, if self.left_strings[j] { " " } else { "|" })?;
            for i in 0..self.top_strings.len() {
                write!(f, " {}", if self.right_strings[i][j] { " " } else { "|" })?;
            }
            write!(f, "\n  ")?;
            for i in 0..self.top_strings.len() {
                write!(f, "+{}", if self.down_strings[i][j] { " " } else { "-" })?;
            }
            write!(f, "+\n")?;
        }
        Ok(())
    }
}

// Struct to encapsulate Zobrist hash for positions
// It has an internal structure mirroring the position, one integer per element
#[derive(Clone)]
struct ZHash {
    current_val: usize,
    top_strings: Vec<usize>,
    left_strings: Vec<usize>,
    right_strings: Vec<Vec<usize>>,
    down_strings: Vec<Vec<usize>>,
}

impl ZHash {
    fn new(width: usize, height: usize) -> ZHash {
        ZHash::new_seeded(width, height, 0)
    }

    fn new_seeded(width: usize, height: usize, extra_seed: usize) -> ZHash {
        let mut seed: [u8; 32] = [0; 32];
        seed[0] = width as u8;
        seed[1] = height as u8;
        seed[2] = extra_seed as u8;
        let mut r: StdRng = SeedableRng::from_seed(seed);
        let mut top_strings: Vec<usize> = Vec::with_capacity(width);
        let mut left_strings: Vec<usize> = Vec::with_capacity(height);
        let mut right_strings: Vec<Vec<usize>> = Vec::with_capacity(width);
        let mut down_strings: Vec<Vec<usize>> = Vec::with_capacity(width);

        for i in 0..width {
            top_strings.push(r.gen());
            right_strings.push(Vec::with_capacity(height));
            down_strings.push(Vec::with_capacity(height));
            for _ in 0..height {
                right_strings[i].push(r.gen());
                down_strings[i].push(r.gen());
            }
        }
        for _ in 0..height {
            left_strings.push(r.gen());
        }

        ZHash{
            current_val: r.gen(),
            top_strings: top_strings,
            left_strings: left_strings,
            right_strings: right_strings,
            down_strings: down_strings,
        }
    }

    fn current_value(self: &ZHash) -> usize {
        self.current_val
    }

    fn toggle_element(self: &mut ZHash, m: Move) {
        match (m.x, m.y, m.side) {
            (0, y, Side::Left) => self.current_val ^= self.left_strings[y],
            (x, 0, Side::Top) => self.current_val ^= self.top_strings[x],
            (x, y, Side::Left) => self.current_val ^= self.right_strings[x-1][y],
            (x, y, Side::Right) => self.current_val ^= self.right_strings[x][y],
            (x, y, Side::Bottom) => self.current_val ^= self.down_strings[x][y],
            (x, y, Side::Top) => self.current_val ^= self.down_strings[x][y-1],
        }
    }
}

// TODO: Extract trait from the simple and compound positions to ensure consistent interface

// Representation of a position composed of multiple rectangular dots-and-boxes
// positions. This allows some additional strings-and-coins positions to be
// represented, such as the one-large-chain-multiple-3-chains positions from
// the paper.
#[derive(Clone)]
pub struct CompoundPosition {
    parts: Vec<SimplePosition>,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub struct CPosMove {
    pub part: usize,
    pub m: Move,
}

impl CPosMove {
    pub fn new(part: usize, x: usize, y: usize, side: Side) -> CPosMove {
        CPosMove{ part: part, m: Move{ x: x, y: y, side: side }}
    }
}

impl fmt::Display for CPosMove {
    fn fmt(self: &CPosMove, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Part {}: {}", self.part, self.m)
    }
}

impl CompoundPosition {
    pub fn new_game(mut parts: Vec<SimplePosition>) -> CompoundPosition {
        // We need different components to have different zhashes even if identical
        for idx in 0..parts.len() {
            parts[idx].zhash = ZHash::new_seeded(parts[idx].width(), parts[idx].height(), idx);
        }
        CompoundPosition{ parts: parts }
    }

    pub fn from_single(pos: SimplePosition) -> CompoundPosition {
        CompoundPosition{ parts: vec!(pos) }
    }
}

impl Position<CPosMove> for CompoundPosition {
    fn is_legal_move(self: &CompoundPosition, m: CPosMove) -> bool {
        if let Some(p) = self.parts.get(m.part) {
            p.is_legal_move(m.m)
        } else {
            false
        }
    }

    fn would_capture(self: &CompoundPosition, m: CPosMove) -> usize {
        self.parts[m.part].would_capture(m.m)
    }

    fn make_move(self: &mut CompoundPosition, m: CPosMove) -> MoveOutcome {
        self.parts[m.part].make_move(m.m)
    }

    fn undo_move(self: &mut CompoundPosition, m: CPosMove) {
        self.parts[m.part].undo_move(m.m)
    }

    fn is_end_of_game(self: &CompoundPosition) -> bool {
        for part in self.parts.iter() {
            if !part.is_end_of_game() {
                return false;
            }
        }
        true
    }

    fn legal_moves(self: &CompoundPosition) -> Vec<CPosMove> {
        let mut result: Vec<CPosMove> = Vec::new();
        for (i, part) in self.parts.iter().enumerate() {
            for &m in part.legal_moves().iter() {
                result.push(CPosMove{ part: i, m: m });
            }
        }
        result
    }

    fn zhash(self: &CompoundPosition) -> usize {
        let mut result = 0;
        for part in self.parts.iter() {
            result ^= part.zhash();
        }
        result
    }
}

impl PartialEq for CompoundPosition {
    fn eq(self: &CompoundPosition, other: &CompoundPosition) -> bool {
        if self.parts.len() == other.parts.len() {
            for (i, part) in self.parts.iter().enumerate() {
                if !part.eq(&other.parts[i]) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

impl Eq for CompoundPosition {}

impl fmt::Display for CompoundPosition {
    fn fmt(self: &CompoundPosition, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, part) in self.parts.iter().enumerate() {
            write!(f, "Component {}:\n", i)?;
            write!(f, "{}", part)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use game::*;
    use examples::*;
    use std::collections::HashSet;

    #[test]
    fn simple_capture() {
        let mut pos = SimplePosition::new_game(3, 3);
        for i in 0..3 {
            for j in 0..3 {
                assert_eq!(true, pos.is_legal_move(Move{x: i, y: j, side: Side::Left}));
                assert_eq!(true, pos.is_legal_move(Move{x: i, y: j, side: Side::Right}));
                assert_eq!(true, pos.is_legal_move(Move{x: i, y: j, side: Side::Bottom}));
                assert_eq!(true, pos.is_legal_move(Move{x: i, y: j, side: Side::Top}));
                assert_eq!(4, pos.valency(i, j));
            }
        }
        // Out of bounds
        assert_eq!(false, pos.is_legal_move(Move{x: 2, y: 3, side: Side::Left}));
        assert_eq!(false, pos.is_legal_move(Move{x: 3, y: 2, side: Side::Left}));

        let outcome = pos.make_move(Move{x: 1, y: 1, side: Side::Right});
        assert_eq!(0, outcome.coins_captured);
        assert_eq!(true, outcome.end_of_turn);
        assert_eq!(false, outcome.end_of_game);
        assert_eq!(false, pos.is_legal_move(Move{x: 1, y: 1, side: Side::Right}));
        assert_eq!(false, pos.is_legal_move(Move{x: 2, y: 1, side: Side::Left}));
        assert_eq!(false, pos.is_captured(1, 1));
        assert_eq!(3, pos.valency(1, 1));
        assert_eq!(3, pos.valency(2, 1));

        let outcome = pos.make_move(Move{x: 1, y: 1, side: Side::Bottom});
        assert_eq!(0, outcome.coins_captured);
        assert_eq!(true, outcome.end_of_turn);
        assert_eq!(false, outcome.end_of_game);
        assert_eq!(false, pos.is_legal_move(Move{x: 1, y: 1, side: Side::Bottom}));
        assert_eq!(false, pos.is_legal_move(Move{x: 1, y: 2, side: Side::Top}));
        assert_eq!(false, pos.is_captured(1, 1));
        assert_eq!(2, pos.valency(1, 1));
        assert_eq!(3, pos.valency(1, 2));

        assert_eq!(0, pos.would_capture(Move{x: 1, y: 1, side: Side::Left}));
        let outcome = pos.make_move(Move{x: 1, y: 1, side: Side::Left});
        assert_eq!(0, outcome.coins_captured);
        assert_eq!(true, outcome.end_of_turn);
        assert_eq!(false, outcome.end_of_game);
        assert_eq!(false, pos.is_legal_move(Move{x: 1, y: 1, side: Side::Left}));
        assert_eq!(false, pos.is_legal_move(Move{x: 0, y: 1, side: Side::Right}));
        assert_eq!(false, pos.is_captured(1, 1));
        assert_eq!(1, pos.valency(1, 1));
        assert_eq!(3, pos.valency(0, 1));

        assert_eq!(1, pos.would_capture(Move{x: 1, y: 1, side: Side::Top}));
        let outcome = pos.make_move(Move{x: 1, y: 1, side: Side::Top});
        assert_eq!(1, outcome.coins_captured);
        assert_eq!(false, outcome.end_of_turn);
        assert_eq!(false, outcome.end_of_game);
        assert_eq!(false, pos.is_legal_move(Move{x: 1, y: 1, side: Side::Top}));
        assert_eq!(false, pos.is_legal_move(Move{x: 1, y: 0, side: Side::Bottom}));
        assert_eq!(true, pos.is_captured(1, 1));
        assert_eq!(0, pos.valency(1, 1));
        assert_eq!(3, pos.valency(1, 0));

        assert_eq!(false, pos.is_end_of_game());
    }

    #[test]
    fn corners() {
        let mut pos = SimplePosition::new_game(3, 3);
        for &(x, y) in [(0, 0), (0, 2), (2, 0), (2, 2)].iter() {
            let mut sides_captured = 0;
            for s in Side::all() {
                let m = Move{x: x, y: y, side: s};
                assert_eq!(true, pos.is_legal_move(m));
                let outcome = pos.make_move(m);
                assert_eq!(false, pos.is_legal_move(m));
                sides_captured += 1;
                assert_eq!(sides_captured == 4, pos.is_captured(x, y));
                assert_eq!(if sides_captured == 4 { 1 } else { 0 }, outcome.coins_captured);
                assert_eq!(sides_captured < 4, outcome.end_of_turn);
            }
        }

        assert_eq!(false, pos.is_end_of_game());
    }

    #[test]
    fn double_cross() {
        let mut pos = SimplePosition::new_game(2, 1);
        assert_eq!(2, pos.width());
        assert_eq!(1, pos.height());
        let moves = [
            Move{x: 0, y: 0, side: Side::Top},
            Move{x: 0, y: 0, side: Side::Bottom},
            Move{x: 1, y: 0, side: Side::Top},
            Move{x: 1, y: 0, side: Side::Bottom},
            Move{x: 0, y: 0, side: Side::Left},
            Move{x: 1, y: 0, side: Side::Right},
        ];
        for &m in moves.iter() {
            assert_eq!(true, pos.is_legal_move(m));
            assert_eq!(0, pos.would_capture(m));
            let outcome = pos.make_move(m);
            assert_eq!(false, pos.is_legal_move(m));
            assert_eq!(0, outcome.coins_captured);
            assert_eq!(true, outcome.end_of_turn);
            assert_eq!(false, outcome.end_of_game);
            assert_eq!(false, pos.is_end_of_game());
        }
        let dc = Move{x: 0, y: 0, side: Side::Right};
        assert_eq!(true, pos.is_legal_move(dc));
        assert_eq!(2, pos.would_capture(dc));
        let outcome = pos.make_move(dc);
        assert_eq!(2, outcome.coins_captured);
        assert_eq!(true, outcome.end_of_game);
        assert_eq!(true, outcome.end_of_turn);
        assert_eq!(true, pos.is_end_of_game());
        assert_eq!(0, pos.legal_moves().len());
    }

    #[test]
    fn undo() {
        let mut pos = SimplePosition::new_game(3, 3);
        let t = Move{x: 1, y: 1, side: Side::Top};
        pos.make_move(t);
        pos.make_move(Move{x: 1, y: 1, side: Side::Left});
        pos.undo_move(t);
        pos.undo_move(Move{x: 0, y: 1, side: Side::Right});
        for i in 0..2 {
            for j in 0..2 {
                for s in Side::all() {
                    assert_eq!(true, pos.is_legal_move(Move{x: i, y: j, side: s}));
                }
            }
        }
    }

    #[test]
    fn move_display() {
        assert_eq!("(0, 0) Top", format!("{}", Move{x: 0, y: 0, side: Side::Top}));
        assert_eq!("(5, 3) Bottom", format!("{}", Move{x: 5, y: 3, side: Side::Bottom}));
    }

    #[test]
    fn pos_display() {
        let mut pos = SimplePosition::new_game(3, 3);
        pos.make_move(Move{x: 1, y: 1, side: Side::Top});
        pos.make_move(Move{x: 0, y: 0, side: Side::Top});
        pos.make_move(Move{x: 0, y: 2, side: Side::Left});
        let display = format!("{}", pos);
        let expected = vec!("   0 1 2",
                            "  +-+ + +",
                            "0        ",
                            "  + +-+ +",
                            "1        ",
                            "  + + + +",
                            "2 |      ",
                            "  + + + +",
                            "");
        let actual: Vec<&str> = display.split("\n").collect();
        assert_eq!(expected.len(), actual.len());
        for i in 0..expected.len() {
            assert_eq!(expected[i], actual[i], "strings mismatch at position {}", i);
        }
    }

    #[test]
    fn big_pos_display() {
        let pos = SimplePosition::new_game(12, 12);
        let actual = format!("{}", pos);
        let lines: Vec<&str> = actual.split("\n").collect();
        assert_eq!("   0 1 2 3 4 5 6 7 8 9 0 1", lines[0]);
        assert!(lines[24].starts_with("1 "), lines[24].to_string());
    }

    #[test]
    fn legal_moves() {
        let mut pos = SimplePosition::new_game(2, 2);
        let moves = pos.legal_moves();
        assert_eq!(12, moves.len());
        // Edge moves, for which there is only one representation
        for i in 0..2 {
            assert!(moves.contains(&Move{ x: i, y: 0, side: Side::Top }));
            assert!(moves.contains(&Move{ x: i, y: 1, side: Side::Bottom }));
        }
        for j in 0..2 {
            assert!(moves.contains(&Move{ x: 0, y: j, side: Side::Left }));
            assert!(moves.contains(&Move{ x: 1, y: j, side: Side::Right }));
        }
        // Interior moves, for which there are two representations
        for i in 0..2 {
            assert!(moves.contains(&Move{ x: i, y: 0, side: Side::Bottom }) ||
                    moves.contains(&Move{ x: i, y: 1, side: Side::Top }));
        }
        for j in 0..2 {
            assert!(moves.contains(&Move{ x: 0, y: j, side: Side::Right }) ||
                    moves.contains(&Move{ x: 1, y: j, side: Side::Left }));
        }

        pos.make_move(Move{x: 0, y: 0, side: Side::Bottom});
        let moves = pos.legal_moves();
        assert!(!moves.contains(&Move{ x: 0, y: 0, side: Side::Bottom }) &&
                !moves.contains(&Move{ x: 0, y: 1, side: Side::Top }));
    }

    #[test]
    fn offsets() {
        let pos = SimplePosition::new_game(2, 2);

        assert_eq!(None, pos.offset(0, 0, Side::Top));
        assert_eq!(None, pos.offset(0, 0, Side::Left));
        assert_eq!(Some((0, 1)), pos.offset(0, 0, Side::Bottom));
        assert_eq!(Some((1, 0)), pos.offset(0, 0, Side::Right));

        assert_eq!(Some((0, 0)), pos.offset(0, 1, Side::Top));
        assert_eq!(None, pos.offset(0, 1, Side::Left));
        assert_eq!(None, pos.offset(0, 1, Side::Bottom));
        assert_eq!(Some((1, 1)), pos.offset(0, 1, Side::Right));

        assert_eq!(None, pos.offset(1, 0, Side::Top));
        assert_eq!(Some((0, 0)), pos.offset(1, 0, Side::Left));
        assert_eq!(Some((1, 1)), pos.offset(1, 0, Side::Bottom));
        assert_eq!(None, pos.offset(1, 0, Side::Right));

        assert_eq!(Some((1, 0)), pos.offset(1, 1, Side::Top));
        assert_eq!(Some((0, 1)), pos.offset(1, 1, Side::Left));
        assert_eq!(None, pos.offset(1, 1, Side::Bottom));
        assert_eq!(None, pos.offset(1, 1, Side::Right));
    }

    #[test]
    fn move_equivalences() {
        let pos = SimplePosition::new_game(2, 2);

        // A move is always equivalent to itself
        assert!(pos.moves_equivalent(Move{x: 0, y: 0, side: Side::Left},
                                     Move{x: 0, y: 0, side: Side::Left}));

        assert!(pos.moves_equivalent(Move{x: 0, y: 0, side: Side::Right},
                                     Move{x: 1, y: 0, side: Side::Left}));
        assert!(pos.moves_equivalent(Move{x: 0, y: 0, side: Side::Bottom},
                                     Move{x: 0, y: 1, side: Side::Top}));

        assert!(!pos.moves_equivalent(Move{x: 0, y: 0, side: Side::Left},
                                      Move{x: 0, y: 0, side: Side::Right}));

        // Out of bounds
        assert!(!pos.moves_equivalent(Move{x: 1, y: 0, side: Side::Right},
                                      Move{x: 2, y: 0, side: Side::Left}));
    }

    #[test]
    fn equality() {
        let pos1 = p50();
        let mut pos2 = p50();
        assert_eq!(true, pos1.eq(&pos2));
        let m = Move{x: 0, y: 3, side: Side::Bottom};
        pos2.make_move(m);
        assert_eq!(false, pos1.eq(&pos2));
        pos2.undo_move(m);
        assert_eq!(true, pos1.eq(&pos2));

        assert_eq!(false, pos1.eq(&p50_top()));
    }

    #[test]
    fn zhashes() {
        let mut pos = SimplePosition::new_game(3, 3);
        let mut hashes: Vec<usize> = Vec::new();
        let mut moves: Vec<Move> = Vec::new();
        while !pos.is_end_of_game() {
            hashes.push(pos.zhash());
            let m = pos.legal_moves()[0];
            pos.make_move(m);
            moves.push(m);
        }
        hashes.push(pos.zhash());

        // Hashes should all be unique
        let unique_hashes: HashSet<usize> = hashes.iter().cloned().collect();
        assert_eq!(hashes.len(), unique_hashes.len());

        // Undoing moves all the way back should give the same sequence of hashes in reverse
        while !moves.is_empty() {
            let hash = hashes.pop().unwrap();
            let m = moves.pop().unwrap();
            assert_eq!(hash, pos.zhash());
            pos.undo_move(m);
        }
        assert_eq!(1, hashes.len());
        assert_eq!(hashes.pop().unwrap(), pos.zhash());
    }

    #[test]
    fn zhashes_across_position() {
        let (width, height) = (3, 4);
        let mut pos1 = SimplePosition::new_game(width, height);
        let mut pos2 = SimplePosition::new_game(width, height);
        assert_eq!(pos1.zhash(), pos2.zhash());
        let m = Move{x: 1, y: 1, side: Side::Top};
        pos1.make_move(m);
        pos2.make_move(m);
        assert_eq!(pos1.zhash(), pos2.zhash());

        assert_eq!(SimplePosition::new_end_game(width, height).zhash(),
                   SimplePosition::new_end_game(width, height).zhash());
    }

    fn all_hashes(pos: &mut SimplePosition) -> HashSet<usize> {
        let mut hashes = HashSet::new();
        loop {
            hashes.insert(pos.zhash());
            let legal_moves = pos.legal_moves();
            if legal_moves.is_empty() {
                hashes.insert(pos.zhash());
                break;
            }
            let m = legal_moves[0];
            pos.make_move(m);
        }
        hashes
    }

    #[test]
    fn zhashes_across_games() {
        let (width, height) = (3, 4);
        let mut pos1 = SimplePosition::new_game(width, height);
        let mut pos2 = SimplePosition::new_game(height, width);
        let hashes1 = all_hashes(&mut pos1);
        let hashes2 = all_hashes(&mut pos2);
        let intersect: HashSet<usize> = hashes1.intersection(&hashes2).cloned().collect();
        assert!(intersect.is_empty());
    }

    #[test]
    fn end_position() {
        let (width, height) = (3, 4);
        let pos = SimplePosition::new_end_game(width, height);
        for i in 0..width {
            for j in 0..height {
                for s in Side::all() {
                    assert_eq!(false, pos.is_legal_move(Move{x: i, y: j, side: s}));
                }
            }
        }
    }

    #[test]
    fn all_sides() {
        let sides = Side::all();
        assert_eq!(4, sides.len());
        assert!(sides.contains(&Side::Top));
        assert!(sides.contains(&Side::Bottom));
        assert!(sides.contains(&Side::Left));
        assert!(sides.contains(&Side::Right));
    }

    #[test]
    fn opposites() {
        assert_eq!(Side::Left, Side::Right.opposite());
        assert_eq!(Side::Right, Side::Left.opposite());
        assert_eq!(Side::Top, Side::Bottom.opposite());
        assert_eq!(Side::Bottom, Side::Top.opposite());
    }

    #[test]
    fn all_sides_except() {
        for side in Side::all() {
            let sides = Side::all_except(side);
            assert_eq!(3, sides.len());
            for other_side in Side::all() {
                assert_eq!(side != other_side, sides.contains(&other_side));
            }
        }
    }

    #[test]
    fn compound_move() {
        assert_eq!("Part 1: (2, 3) Right", format!("{}", CPosMove::new(1, 2, 3, Side::Right)));
    }

    #[test]
    fn compound_position() {
        let mut pos = one_long_multi_three(3, 4);
        let init_hash = pos.zhash();
        assert_eq!(true, pos.is_legal_move(CPosMove::new(0, 3, 0, Side::Right)));
        assert_eq!(false, pos.is_legal_move(CPosMove::new(0, 4, 0, Side::Right)));
        assert_eq!(false, pos.is_legal_move(CPosMove::new(4, 0, 0, Side::Left)));
        let legal_moves = pos.legal_moves();
        // 5 legal moves for the 4-chain, 4 for each of the 3 3-chains
        assert_eq!(17, legal_moves.len());
        for &m in legal_moves.iter() {
            assert_eq!(false, pos.is_end_of_game());
            let wc = pos.would_capture(m);
            let outcome = pos.make_move(m);
            assert_eq!(wc, outcome.coins_captured);
            assert!(init_hash != pos.zhash());
        }
        assert_eq!(true, pos.is_end_of_game());
        let final_hash = pos.zhash();
        assert_ne!(0, final_hash);
        for &m in legal_moves.iter() {
            pos.undo_move(m);
            assert!(final_hash != pos.zhash());
            assert_eq!(false, pos.is_end_of_game());
        }
        assert_eq!(init_hash, pos.zhash());
    }

    #[test]
    fn compound_pos_eq() {
        let pos = one_long_multi_three(3, 4);
        assert_eq!(true, pos.eq(&one_long_multi_three(3, 4)));
        assert_eq!(false, pos.eq(&one_long_multi_three(4, 4)));
        assert_eq!(false, pos.eq(&one_long_multi_three(3, 5)));
    }

    #[test]
    fn compound_pos_display() {
        let pos = one_long_multi_three(2, 4);
        let expected = vec!("Component 0:",
                            "   0 1 2 3",
                            "  +-+-+-+-+",
                            "0          ",
                            "  +-+-+-+-+",
                            "Component 1:",
                            "   0 1 2",
                            "  +-+-+-+",
                            "0        ",
                            "  +-+-+-+",
                            "Component 2:",
                            "   0 1 2",
                            "  +-+-+-+",
                            "0        ",
                            "  +-+-+-+",
                            "");
        let display = format!("{}", pos);
        let actual: Vec<&str> = display.split("\n").collect();
        assert_eq!(expected.len(), actual.len());
        for i in 0..expected.len() {
            assert_eq!(expected[i], actual[i], "strings mismatch at position {}", i);
        }
    }

    #[test]
    fn compound_pos_zhash() {
        let simple = SimplePosition::new_game(3, 1);
        let legals = simple.legal_moves();

        // We don't want the zhash to be zero just because the two sub-positions
        // have equal and cancelling hashes
        let mut pos = CompoundPosition::new_game(
            vec!(SimplePosition::new_game(3, 1), SimplePosition::new_game(3, 1)));
        let mut hashes = HashSet::new();
        hashes.insert(pos.zhash());
        for &m in &legals {
            for p in 0..2 {
                pos.make_move(CPosMove{part: p, m: m});
                hashes.insert(pos.zhash());
            }
        }
        assert_eq!(legals.len() * 2 + 1, hashes.len());
        assert_eq!(false, hashes.contains(&0));
    }
}
