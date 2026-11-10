/*
    Copyright 2017-2020, 2026 Andrew Medworth <github@medworth.org.uk>

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
use rand::{RngExt, SeedableRng};
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    // Indicate whether a given move is legal in the current position.
    fn is_legal_move(&self, m: M) -> bool;
    // Indicate how many coins a given move would capture (either 0, 1 or 2)
    fn would_capture(&self, m: M) -> usize;
    // Make a given move on the board, and indicate the outcome.
    fn make_move(&mut self, m: M) -> MoveOutcome;
    // Undo a given move by putting the line back on the board.
    // Behaviour if the move was never made in the first place is undefined.
    fn undo_move(&mut self, m: M);
    // Indicate whether the game is over (i.e. whether all strings have been cut).
    fn is_end_of_game(&self) -> bool;
    // Compute all possible legal moves in the position.
    fn legal_moves(&self) -> Vec<M>;
    // Indicate whether two moves are equivalent in the current position,
    // i.e. they refer to the same "logical" move on the board.
    // For example in a SimplePosition, (0,0) Right is equivalent to (1,0) Left
    // (provided the board is at least two columns wide).
    fn moves_equivalent(&self, m1: M, m2: M) -> bool;
    // Current Zobrist hash value for position.
    // This hash should be consistent across positions,
    // i.e. equal positions should have equal hashes.
    fn zhash(&self) -> u64;
}

// Plain-data description of a SimplePosition, for exchange with other programs
// such as the web interface. Strings are listed in terms of the lines of the
// equivalent dots-and-boxes position:
// * h_strings[y][x] is the string crossed by the horizontal line along the top
//   of square (x, y), for y in 0..=height (y == height is the bottom edge)
// * v_strings[y][x] is the string crossed by the vertical line along the left
//   of square (x, y), for x in 0..=width (x == width is the right edge)
// A value of true means the string is still present, i.e. the line has not yet
// been drawn.
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SimplePositionData {
    pub width: usize,
    pub height: usize,
    pub h_strings: Vec<Vec<bool>>,
    pub v_strings: Vec<Vec<bool>>,
}

// Plain-data description of a CompoundPosition: see SimplePositionData.
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompoundPositionData {
    pub parts: Vec<SimplePositionData>,
}

// The move which cuts the string crossed by the horizontal line at row y, column x
// (in the SimplePositionData sense), in the form used by legal_moves().
fn h_line_move(x: usize, y: usize) -> Move {
    if y == 0 { Move::new(x, 0, Side::Top) } else { Move::new(x, y - 1, Side::Bottom) }
}

// The move which cuts the string crossed by the vertical line at row y, column x
// (in the SimplePositionData sense), in the form used by legal_moves().
fn v_line_move(x: usize, y: usize) -> Move {
    if x == 0 { Move::new(0, y, Side::Left) } else { Move::new(x - 1, y, Side::Right) }
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
        let top_strings: Vec<bool> = iter::repeat(init_string).take(width).collect();
        let left_strings: Vec<bool> = iter::repeat(init_string).take(height).collect();
        let mut right_strings: Vec<Vec<bool>> = Vec::with_capacity(width);
        let mut down_strings: Vec<Vec<bool>> = Vec::with_capacity(width);
        for _ in 0..width {
            right_strings.push(iter::repeat(init_string).take(height).collect());
            down_strings.push(iter::repeat(init_string).take(height).collect());
        }
        let mut zhash = ZHash::new(width, height);
        if !init_string {
            zhash.toggle_cut_strings(&top_strings, &left_strings, &right_strings, &down_strings);
        }
        SimplePosition {
            top_strings: top_strings,
            left_strings: left_strings,
            down_strings: down_strings,
            right_strings: right_strings,
            zhash: zhash,
        }
    }

    // Every move on an empty board of the given dimensions, in the form used by legal_moves().
    pub fn all_moves(width: usize, height: usize) -> Vec<Move> {
        SimplePosition::new_game(width, height).legal_moves()
    }

    // Recalculate the Zobrist hash from scratch using the given seed. A fresh ZHash
    // represents a board with every string present, so toggle each string which has been cut.
    // This ensures that the hash depends only on the dimensions, the seed and the strings
    // present, however the position was constructed.
    fn reset_zhash(self: &mut SimplePosition, extra_seed: usize) {
        self.zhash = ZHash::new_seeded(self.width(), self.height(), extra_seed);
        self.zhash.toggle_cut_strings(&self.top_strings, &self.left_strings, &self.right_strings, &self.down_strings);
    }

    pub fn width(self: &SimplePosition) -> usize {
        self.top_strings.len()
    }

    pub fn height(self: &SimplePosition) -> usize {
        self.left_strings.len()
    }

    // Build a position from its plain-data description.
    pub fn from_data(data: &SimplePositionData) -> Result<SimplePosition, String> {
        let (width, height) = (data.width, data.height);
        if width == 0 || height == 0 {
            return Err(format!("Invalid dimensions {}x{}", width, height));
        }
        if data.h_strings.len() != height + 1 || data.h_strings.iter().any(|row| row.len() != width) {
            return Err(format!("Expected {} rows of {} horizontal strings", height + 1, width));
        }
        if data.v_strings.len() != height || data.v_strings.iter().any(|row| row.len() != width + 1) {
            return Err(format!("Expected {} rows of {} vertical strings", height, width + 1));
        }
        // Start from a new game and cut strings, so the Zobrist hash is consistent
        // with positions reached by play.
        let mut pos = SimplePosition::new_game(width, height);
        for (y, row) in data.h_strings.iter().enumerate() {
            for (x, &present) in row.iter().enumerate() {
                if !present {
                    pos.cut_string(h_line_move(x, y));
                }
            }
        }
        for (y, row) in data.v_strings.iter().enumerate() {
            for (x, &present) in row.iter().enumerate() {
                if !present {
                    pos.cut_string(v_line_move(x, y));
                }
            }
        }
        Ok(pos)
    }

    // Describe this position as plain data.
    pub fn to_data(self: &SimplePosition) -> SimplePositionData {
        let (width, height) = (self.width(), self.height());
        SimplePositionData {
            width: width,
            height: height,
            h_strings: (0..=height).map(
                |y| (0..width).map(|x| self.is_legal_move(h_line_move(x, y))).collect()).collect(),
            v_strings: (0..height).map(
                |y| (0..=width).map(|x| self.is_legal_move(v_line_move(x, y))).collect()).collect(),
        }
    }

    // Express a move in the canonical form used by legal_moves(), so that
    // for example (1, 0) Left becomes (0, 0) Right. Returns None if the move's
    // coordinates are off the board.
    pub fn normalise_move(self: &SimplePosition, m: Move) -> Option<Move> {
        if m.x >= self.width() || m.y >= self.height() {
            return None;
        }
        Some(match (m.x, m.y, m.side) {
            (x, y, Side::Top) if y > 0 => Move::new(x, y - 1, Side::Bottom),
            (x, y, Side::Left) if x > 0 => Move::new(x - 1, y, Side::Right),
            _ => m,
        })
    }

    // Cut a string without checking for captures (for setting up positions).
    fn cut_string(self: &mut SimplePosition, m: Move) {
        if self.is_legal_move(m) {
            self.set_string_value(m, false);
            self.zhash.toggle_element(m);
        }
    }

    // Indicate whether a given square has been captured.
    pub fn is_captured(self: &SimplePosition, x: usize, y: usize) -> bool {
        Side::all().iter().all(|&s| !self.is_legal_move(Move{x: x, y: y, side: s}))
    }

    // Valency or degree of coin at a given position
    pub fn valency(self: &SimplePosition, x: usize, y: usize) -> usize {
        Side::all().iter().filter(|&&s| self.is_legal_move(Move{x: x, y: y, side: s})).count()
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

    fn set_string_value(self: &mut SimplePosition, m: Move, val: bool) {
        match (m.x, m.y, m.side) {
            (0, y, Side::Left) => self.left_strings[y] = val,
            (x, 0, Side::Top) => self.top_strings[x] = val,
            (x, y, Side::Top) => self.down_strings[x][y-1] = val,
            (x, y, Side::Bottom) => self.down_strings[x][y] = val,
            (x, y, Side::Left) => self.right_strings[x-1][y] = val,
            (x, y, Side::Right) => self.right_strings[x][y] = val,
        }
    }

    fn legal_move_count(self: &SimplePosition) -> usize {
        self.left_strings.iter().filter(|&&b| b).count() +
        self.top_strings.iter().filter(|&&b| b).count() +
        self.down_strings.iter().flat_map(|r| r.iter().filter(|&&b| b)).count() +
        self.right_strings.iter().flat_map(|r| r.iter().filter(|&&b| b)).count()
    }
}

impl Position<Move> for SimplePosition {
    fn is_legal_move(self: &SimplePosition, m: Move) -> bool {
        if m.x >= self.width() {
            return false;
        }
        if m.y >= self.height() {
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

    fn make_move(self: &mut SimplePosition, m: Move) -> MoveOutcome {
        if !self.is_legal_move(m) {
            panic!("Illegal move {}, pos:\n{}", m, self);
        }
        self.set_string_value(m, false);
        let mut captures = if self.is_captured(m.x, m.y) { 1 } else { 0 };
        if m.side == Side::Left && m.x > 0 {
            if self.is_captured(m.x-1, m.y) {
                captures += 1
            }
        }
        if m.side == Side::Right && m.x < self.width()-1 {
            if self.is_captured(m.x+1, m.y) {
                captures += 1
            }
        }
        if m.side == Side::Top && m.y > 0 {
            if self.is_captured(m.x, m.y-1) {
                captures += 1
            }
        }
        if m.side == Side::Bottom && m.y < self.height()-1 {
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

    fn undo_move(self: &mut SimplePosition, m: Move) {
        self.set_string_value(m, true);
        self.zhash.toggle_element(m);
    }

    fn is_end_of_game(self: &SimplePosition) -> bool {
        if self.left_strings.iter().any(|&b| b) {
            return false;
        }
        if self.top_strings.iter().any(|&b| b) {
            return false;
        }
        if self.down_strings.iter().any(|col| col.iter().any(|&b| b)) {
            return false;
        }
        if self.right_strings.iter().any(|col| col.iter().any(|&b| b)) {
            return false;
        }
        true
    }

    fn legal_moves(self: &SimplePosition) -> Vec<Move> {
        let mut result: Vec<Move> = Vec::with_capacity(self.legal_move_count());
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
        for x in 0..self.width() {
            for y in 0..self.height() {
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

    fn moves_equivalent(self: &SimplePosition, move1: Move, move2: Move) -> bool {
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

    fn zhash(self: &SimplePosition) -> u64 {
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
        for j in 0..self.height() {
            write!(f, "{} {}", j % 10, if self.left_strings[j] { " " } else { "|" })?;
            for i in 0..self.width() {
                write!(f, " {}", if self.right_strings[i][j] { " " } else { "|" })?;
            }
            write!(f, "\n  ")?;
            for i in 0..self.width() {
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
    current_val: u64,
    top_strings: Vec<u64>,
    left_strings: Vec<u64>,
    right_strings: Vec<Vec<u64>>,
    down_strings: Vec<Vec<u64>>,
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
        let mut top_strings: Vec<u64> = Vec::with_capacity(width);
        let mut left_strings: Vec<u64> = Vec::with_capacity(height);
        let mut right_strings: Vec<Vec<u64>> = Vec::with_capacity(width);
        let mut down_strings: Vec<Vec<u64>> = Vec::with_capacity(width);

        for i in 0..width {
            top_strings.push(r.random());
            right_strings.push(Vec::with_capacity(height));
            down_strings.push(Vec::with_capacity(height));
            for _ in 0..height {
                right_strings[i].push(r.random());
                down_strings[i].push(r.random());
            }
        }
        for _ in 0..height {
            left_strings.push(r.random());
        }

        ZHash{
            current_val: r.random(),
            top_strings: top_strings,
            left_strings: left_strings,
            right_strings: right_strings,
            down_strings: down_strings,
        }
    }

    // Toggle the element for each string which has been cut (is false) in a position's
    // string arrays, which have the same shape as this hash's arrays
    fn toggle_cut_strings(self: &mut ZHash, top: &[bool], left: &[bool], right: &[Vec<bool>], down: &[Vec<bool>]) {
        fn xor_cut(vals: &[u64], present: &[bool]) -> u64 {
            vals.iter().zip(present).filter(|&(_, &p)| !p).fold(0, |acc, (&v, _)| acc ^ v)
        }
        self.current_val ^= xor_cut(&self.top_strings, top);
        self.current_val ^= xor_cut(&self.left_strings, left);
        for (vals, present) in self.right_strings.iter().zip(right) {
            self.current_val ^= xor_cut(vals, present);
        }
        for (vals, present) in self.down_strings.iter().zip(down) {
            self.current_val ^= xor_cut(vals, present);
        }
    }

    fn current_value(self: &ZHash) -> u64 {
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

// Representation of a position composed of multiple rectangular dots-and-boxes
// positions. This allows some additional strings-and-coins positions to be
// represented, such as the one-large-chain-multiple-3-chains positions from
// the paper.
#[derive(Clone)]
pub struct CompoundPosition {
    pub(crate) parts: Vec<SimplePosition>,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub fn new(mut parts: Vec<SimplePosition>) -> CompoundPosition {
        // We need different components to have different zhashes even if identical
        for idx in 0..parts.len() {
            parts[idx].reset_zhash(idx);
        }
        CompoundPosition{ parts: parts }
    }

    pub fn from_single(pos: SimplePosition) -> CompoundPosition {
        CompoundPosition{ parts: vec!(pos) }
    }

    pub fn parts(self: &CompoundPosition) -> &[SimplePosition] {
        &self.parts
    }

    // Build a position from its plain-data description.
    pub fn from_data(data: &CompoundPositionData) -> Result<CompoundPosition, String> {
        if data.parts.is_empty() {
            return Err("A position must have at least one part".to_string());
        }
        let parts = data.parts.iter().enumerate().map(
            |(i, p)| SimplePosition::from_data(p).map_err(|e| format!("Part {}: {}", i, e))
        ).collect::<Result<Vec<SimplePosition>, String>>()?;
        Ok(CompoundPosition::new(parts))
    }

    // Describe this position as plain data.
    pub fn to_data(self: &CompoundPosition) -> CompoundPositionData {
        CompoundPositionData{ parts: self.parts.iter().map(|p| p.to_data()).collect() }
    }

    // Express a move in the canonical form used by legal_moves():
    // see SimplePosition::normalise_move.
    pub fn normalise_move(self: &CompoundPosition, m: CPosMove) -> Option<CPosMove> {
        self.parts.get(m.part).and_then(|p| p.normalise_move(m.m)).map(|nm| CPosMove{ part: m.part, m: nm })
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
        let outcome = self.parts[m.part].make_move(m.m);
        // Finishing one part doesn't end the game (or the turn, after a capture)
        // unless every other part is finished too
        let end_of_game = outcome.end_of_game && self.is_end_of_game();
        MoveOutcome {
            coins_captured: outcome.coins_captured,
            end_of_turn: outcome.coins_captured == 0 || end_of_game,
            end_of_game: end_of_game,
        }
    }

    fn undo_move(self: &mut CompoundPosition, m: CPosMove) {
        self.parts[m.part].undo_move(m.m)
    }

    fn is_end_of_game(self: &CompoundPosition) -> bool {
        self.parts.iter().all(|p| p.is_end_of_game())
    }

    fn legal_moves(self: &CompoundPosition) -> Vec<CPosMove> {
        let sub_moves: Vec<Vec<Move>> = self.parts.iter().map(|p| p.legal_moves()).collect();
        let mut result: Vec<CPosMove> = Vec::with_capacity(sub_moves.iter().map(|ms| ms.len()).sum());
        for (i, moves) in sub_moves.iter().enumerate() {
            for &m in moves.iter() {
                result.push(CPosMove{ part: i, m: m });
            }
        }
        result
    }

    fn moves_equivalent(self: &CompoundPosition, move1: CPosMove, move2: CPosMove) -> bool {
        move1.part == move2.part && self.parts[move1.part].moves_equivalent(move1.m, move2.m)
    }

    fn zhash(self: &CompoundPosition) -> u64 {
        let mut result = 0u64;
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
    use crate::game::*;
    use crate::examples::*;
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
        assert!(lines[24].starts_with("1 "), "{}", lines[24].to_string());
    }

    #[test]
    fn legal_moves() {
        let mut pos = SimplePosition::new_game(2, 2);
        let moves = pos.legal_moves();
        assert_eq!(12, moves.len());
        assert_eq!(12, pos.legal_move_count());
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
    fn compound_move_equivalences() {
        let pos = CompoundPosition::new(vec!(make_chain(5), make_chain(5)));
        assert_eq!(true, pos.moves_equivalent(CPosMove::new(0, 0, 0, Side::Left),
                                              CPosMove::new(0, 0, 0, Side::Left)));
        assert_eq!(true, pos.moves_equivalent(CPosMove::new(0, 0, 0, Side::Right),
                                              CPosMove::new(0, 1, 0, Side::Left)));
        assert_eq!(false, pos.moves_equivalent(CPosMove::new(0, 0, 0, Side::Left),
                                               CPosMove::new(1, 0, 0, Side::Left)));
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
        let mut hashes: Vec<u64> = Vec::new();
        let mut moves: Vec<Move> = Vec::new();
        while !pos.is_end_of_game() {
            hashes.push(pos.zhash());
            let m = pos.legal_moves()[0];
            pos.make_move(m);
            moves.push(m);
        }
        hashes.push(pos.zhash());

        // Hashes should all be unique
        let unique_hashes: HashSet<u64> = hashes.iter().cloned().collect();
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

        assert_eq!(pos1.zhash(), CompoundPosition::from_single(pos1).zhash());

        assert_eq!(SimplePosition::new_end_game(width, height).zhash(),
                   SimplePosition::new_end_game(width, height).zhash());
    }

    fn all_hashes(pos: &mut SimplePosition) -> HashSet<u64> {
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
        let intersect: HashSet<u64> = hashes1.intersection(&hashes2).cloned().collect();
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
        let mut pos = CompoundPosition::new(
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

    #[test]
    fn data_round_trip() {
        for pos in [SimplePosition::new_game(3, 2), SimplePosition::new_end_game(2, 3),
                    p50(), ex3p12(), make_chain(4), double_loop(3)] {
            let data = pos.to_data();
            let rebuilt = SimplePosition::from_data(&data).unwrap();
            assert!(pos.eq(&rebuilt), "{} vs {}", pos, rebuilt);
            assert_eq!(data, rebuilt.to_data());
        }
    }

    #[test]
    fn data_layout() {
        let mut pos = SimplePosition::new_game(2, 1);
        pos.make_move(Move::new(1, 0, Side::Bottom));
        pos.make_move(Move::new(1, 0, Side::Left));
        let data = pos.to_data();
        assert_eq!(2, data.width);
        assert_eq!(1, data.height);
        assert_eq!(vec![vec![true, true], vec![true, false]], data.h_strings);
        assert_eq!(vec![vec![true, false, true]], data.v_strings);
    }

    #[test]
    fn data_zhash_consistent() {
        // A position built from data must hash the same as one reached by play,
        // or evaluation caches would treat equal positions as different
        let pos = p50();
        let rebuilt = SimplePosition::from_data(&pos.to_data()).unwrap();
        assert_eq!(pos.zhash(), rebuilt.zhash());
    }

    #[test]
    fn data_errors() {
        let good = SimplePosition::new_game(2, 2).to_data();
        let mut bad = good.clone();
        bad.width = 0;
        assert!(SimplePosition::from_data(&bad).is_err());
        let mut bad = good.clone();
        bad.h_strings.pop();
        assert!(SimplePosition::from_data(&bad).is_err());
        let mut bad = good.clone();
        bad.v_strings[1].push(true);
        assert!(SimplePosition::from_data(&bad).is_err());
        assert!(CompoundPosition::from_data(&CompoundPositionData{ parts: vec![] }).is_err());
        let err = CompoundPosition::from_data(&CompoundPositionData{ parts: vec![good.clone(), bad] }).err().unwrap();
        assert!(err.starts_with("Part 1:"), "{}", err);
    }

    #[test]
    fn compound_data_round_trip() {
        let pos = one_long_multi_three(3, 4);
        let rebuilt = CompoundPosition::from_data(&pos.to_data()).unwrap();
        assert!(pos.eq(&rebuilt));
        assert_eq!(pos.zhash(), rebuilt.zhash());
    }

    #[test]
    fn normalise_moves() {
        let pos = SimplePosition::new_game(3, 2);
        assert_eq!(Some(Move::new(0, 0, Side::Bottom)), pos.normalise_move(Move::new(0, 1, Side::Top)));
        assert_eq!(Some(Move::new(1, 1, Side::Right)), pos.normalise_move(Move::new(2, 1, Side::Left)));
        for m in [Move::new(1, 0, Side::Top), Move::new(0, 1, Side::Left),
                  Move::new(2, 1, Side::Bottom), Move::new(2, 1, Side::Right)] {
            assert_eq!(Some(m), pos.normalise_move(m));
        }
        assert_eq!(None, pos.normalise_move(Move::new(3, 0, Side::Top)));
        assert_eq!(None, pos.normalise_move(Move::new(0, 2, Side::Top)));
        // Every legal move is already in normal form
        for m in pos.legal_moves() {
            assert_eq!(Some(m), pos.normalise_move(m));
        }

        let cpos = CompoundPosition::new(vec![make_chain(2), make_chain(3)]);
        assert_eq!(Some(CPosMove::new(1, 1, 0, Side::Right)), cpos.normalise_move(CPosMove::new(1, 2, 0, Side::Left)));
        assert_eq!(None, cpos.normalise_move(CPosMove::new(2, 0, 0, Side::Left)));
        assert_eq!(None, cpos.normalise_move(CPosMove::new(0, 2, 0, Side::Left)));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_formats() {
        let m = CPosMove::new(1, 2, 3, Side::Left);
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(r#"{"part":1,"m":{"x":2,"y":3,"side":"Left"}}"#, json);
        assert_eq!(m, serde_json::from_str(&json).unwrap());

        let data = CompoundPosition::new(vec![make_chain(2)]).to_data();
        let json = serde_json::to_string(&data).unwrap();
        assert_eq!(
            r#"{"parts":[{"width":2,"height":1,"h_strings":[[false,false],[false,false]],"v_strings":[[true,true,true]]}]}"#,
            json);
        assert_eq!(data, serde_json::from_str(&json).unwrap());
    }

    #[test]
    fn zhash_independent_of_construction() {
        // Same position reached by play and built from an end-game position
        let mut played = SimplePosition::new_game(2, 2);
        for m in played.legal_moves() {
            played.make_move(m);
        }
        assert_eq!(played.zhash(), SimplePosition::new_end_game(2, 2).zhash());

        // An empty board and a finished one must not collide
        assert_ne!(SimplePosition::new_game(2, 2).zhash(), SimplePosition::new_end_game(2, 2).zhash());

        // Compound position built from parts with moves already made, versus moves made afterwards
        let prebuilt = CompoundPosition::new(vec![make_chain(3), make_chain(2)]);
        let mut played = CompoundPosition::new(vec![SimplePosition::new_game(3, 1), SimplePosition::new_game(2, 1)]);
        for (part, len) in [(0, 3), (1, 2)] {
            for x in 0..len {
                played.make_move(CPosMove::new(part, x, 0, Side::Top));
                played.make_move(CPosMove::new(part, x, 0, Side::Bottom));
            }
        }
        assert!(prebuilt.eq(&played));
        assert_eq!(prebuilt.zhash(), played.zhash());
    }

    #[test]
    fn compound_end_of_game() {
        // Two 1-chains: taking the first coin finishes one part but not the game
        let mut pos = CompoundPosition::new(vec!(make_chain(1), make_chain(1)));
        pos.make_move(CPosMove::new(0, 0, 0, Side::Left));
        let outcome = pos.make_move(CPosMove::new(0, 0, 0, Side::Right));
        assert_eq!(1, outcome.coins_captured);
        assert_eq!(false, outcome.end_of_game);
        assert_eq!(false, outcome.end_of_turn);

        pos.make_move(CPosMove::new(1, 0, 0, Side::Left));
        let outcome = pos.make_move(CPosMove::new(1, 0, 0, Side::Right));
        assert_eq!(1, outcome.coins_captured);
        assert_eq!(true, outcome.end_of_game);
        assert_eq!(true, outcome.end_of_turn);
    }
}
