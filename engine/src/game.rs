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
use std::fmt;

#[derive(Clone)]
#[derive(Copy)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
#[derive(Debug)]
pub enum Side {
    Top, Bottom, Left, Right
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub struct MoveOutcome {
    pub coins_captured: usize,
    pub end_of_turn: bool,
    // TODO: Add end_of_game
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

// An m*n dots-and-boxes position is represented as:
// * a row of top ground links
// * a column of left ground links
// * an m*n array of downward-pointing links
// * an m*n array of rightward-pointing links
// Position coordinates originate at the top left and are 0-based,
// so x=1,y=2 is the second square in the third row.
pub struct Position {
    // TODO: Make this a generic type and change the vectors to arrays
    top_strings: Vec<bool>,
    left_strings: Vec<bool>,
    down_strings: Vec<Vec<bool>>,
    right_strings: Vec<Vec<bool>>,
}

impl Position {
    // Create a new dots-and-boxes position of a given size.
    pub fn new_game(width: usize, height: usize) -> Position {
        let mut top_strings = Vec::with_capacity(width);
        let mut left_strings = Vec::with_capacity(height);
        let mut down_strings = Vec::with_capacity(width);
        let mut right_strings = Vec::with_capacity(width);

        for i in 0..width {
            top_strings.push(true);
            down_strings.push(Vec::with_capacity(height));
            right_strings.push(Vec::with_capacity(height));
            for _ in 0..height {
                down_strings[i].push(true);
                right_strings[i].push(true);
            }
        }
        for _ in 0..height {
            left_strings.push(true);
        }
        Position {
            top_strings: top_strings,
            left_strings: left_strings,
            down_strings: down_strings,
            right_strings: right_strings,
        }
    }

    pub fn width(self: &Position) -> usize {
        self.top_strings.len()
    }

    pub fn height(self: &Position) -> usize {
        self.left_strings.len()
    }

    // Indicate whether a given move is legal in the current position.
    pub fn is_legal_move(self: &Position, x: usize, y: usize, s: Side) -> bool {
        if x >= self.down_strings.len() {
            return false;
        }
        if y >= self.left_strings.len() {
            return false;
        }
        match (x, y, s) {
            (0, y, Side::Left) => self.left_strings[y],
            (x, 0, Side::Top) => self.top_strings[x],
            (x, y, Side::Top) => self.down_strings[x][y-1],
            (x, y, Side::Bottom) => self.down_strings[x][y],
            (x, y, Side::Left) => self.right_strings[x-1][y],
            (x, y, Side::Right) => self.right_strings[x][y],
        }
    }

    // Indicate whether a given square has been captured.
    pub fn is_captured(self: &Position, x: usize, y: usize) -> bool {
        !self.is_legal_move(x, y, Side::Left) &&
            !self.is_legal_move(x, y, Side::Right) &&
            !self.is_legal_move(x, y, Side::Top) &&
            !self.is_legal_move(x, y, Side::Bottom)
    }

    // Indicate whether a given move would be a capture
    pub fn would_capture(self: &Position, x: usize, y: usize, s: Side) -> bool {
        if self.valency(x, y) == 1 {
            return true;
        }
        if let Some((nx, ny)) = self.offset(x, y, s) {
            self.valency(nx, ny) == 1
        }
        else {
            false
        }
    }

    // Make a given move on the board, and indicate the outcome.
    pub fn make_move(self: &mut Position, x: usize, y: usize, s: Side) -> MoveOutcome {
        if !self.is_legal_move(x, y, s) {
            panic!(format!("Illegal move x = {}, y = {}, s = {:?}", x, y, s));
        }
        match (x, y, s) {
            (0, y, Side::Left) => self.left_strings[y] = false,
            (x, 0, Side::Top) => self.top_strings[x] = false,
            (x, y, Side::Top) => self.down_strings[x][y-1] = false,
            (x, y, Side::Bottom) => self.down_strings[x][y] = false,
            (x, y, Side::Left) => self.right_strings[x-1][y] = false,
            (x, y, Side::Right) => self.right_strings[x][y] = false,
        }
        let mut captures = if self.is_captured(x, y) { 1 } else { 0 };
        if s == Side::Left && x > 0 {
            if self.is_captured(x-1, y) {
                captures += 1
            }
        }
        if s == Side::Right && x < self.top_strings.len()-1 {
            if self.is_captured(x+1, y) {
                captures += 1
            }
        }
        if s == Side::Top && y > 0 {
            if self.is_captured(x, y-1) {
                captures += 1
            }
        }
        if s == Side::Bottom && y < self.left_strings.len()-1 {
            if self.is_captured(x, y+1) {
                captures += 1
            }
        }
        MoveOutcome { coins_captured: captures, end_of_turn: captures == 0 || self.is_end_of_game() }
    }

    // Undo a given move by putting the line back on the board.
    // Behaviour if the move was never made in the first place is undefined.
    pub fn undo_move(self: &mut Position, x: usize, y: usize, s: Side) {
        match (x, y, s) {
            (0, y, Side::Left) => self.left_strings[y] = true,
            (x, 0, Side::Top) => self.top_strings[x] = true,
            (x, y, Side::Top) => self.down_strings[x][y-1] = true,
            (x, y, Side::Bottom) => self.down_strings[x][y] = true,
            (x, y, Side::Left) => self.right_strings[x-1][y] = true,
            (x, y, Side::Right) => self.right_strings[x][y] = true,
        }
    }

    // Indicate whether the game is over (i.e. whether all strings have been cut).
    pub fn is_end_of_game(self: &Position) -> bool {
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
    pub fn legal_moves(self: &Position) -> Vec<Move> {
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

    // Valency or degree of coin at a given position
    pub fn valency(self: &Position, x: usize, y: usize) -> usize {
        let mut result = 0;
        for &s in [Side::Top, Side::Bottom, Side::Left, Side::Right].iter() {
            if self.is_legal_move(x, y, s) {
                result += 1
            }
        }
        result
    }

    // Move from the square indicated in the direction indicated by the side,
    // returning a result only if that square is still on the board
    pub fn offset(self: &Position, x: usize, y: usize, s: Side) -> Option<(usize, usize)> {
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
}

impl fmt::Display for Position {
    fn fmt(self: &Position, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "  ")?;
        for i in 0..self.width() {
            write!(f, " {}", i)?;
        }
        write!(f, "\n  ")?;
        for &b in self.top_strings.iter() {
            write!(f, "+{}", if b { " " } else { "-" })?;
        }
        write!(f, "+\n")?;
        for j in 0..self.left_strings.len() {
            write!(f, "{} {}", j, if self.left_strings[j] { " " } else { "|" })?;
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

#[cfg(test)]
mod tests {
    use game::*;

    #[test]
    fn simple_capture() {
        let mut pos = Position::new_game(3, 3);
        for i in 0..3 {
            for j in 0..3 {
                assert_eq!(true, pos.is_legal_move(i, j, Side::Left));
                assert_eq!(true, pos.is_legal_move(i, j, Side::Right));
                assert_eq!(true, pos.is_legal_move(i, j, Side::Bottom));
                assert_eq!(true, pos.is_legal_move(i, j, Side::Top));
                assert_eq!(4, pos.valency(i, j));
            }
        }
        // Out of bounds
        assert_eq!(false, pos.is_legal_move(2, 3, Side::Left));
        assert_eq!(false, pos.is_legal_move(3, 2, Side::Left));

        let outcome = pos.make_move(1, 1, Side::Right);
        assert_eq!(0, outcome.coins_captured);
        assert_eq!(true, outcome.end_of_turn);
        assert_eq!(false, pos.is_legal_move(1, 1, Side::Right));
        assert_eq!(false, pos.is_legal_move(2, 1, Side::Left));
        assert_eq!(false, pos.is_captured(1, 1));
        assert_eq!(3, pos.valency(1, 1));
        assert_eq!(3, pos.valency(2, 1));

        let outcome = pos.make_move(1, 1, Side::Bottom);
        assert_eq!(0, outcome.coins_captured);
        assert_eq!(true, outcome.end_of_turn);
        assert_eq!(false, pos.is_legal_move(1, 1, Side::Bottom));
        assert_eq!(false, pos.is_legal_move(1, 2, Side::Top));
        assert_eq!(false, pos.is_captured(1, 1));
        assert_eq!(2, pos.valency(1, 1));
        assert_eq!(3, pos.valency(1, 2));

        assert_eq!(false, pos.would_capture(1, 1, Side::Left));
        let outcome = pos.make_move(1, 1, Side::Left);
        assert_eq!(0, outcome.coins_captured);
        assert_eq!(true, outcome.end_of_turn);
        assert_eq!(false, pos.is_legal_move(1, 1, Side::Left));
        assert_eq!(false, pos.is_legal_move(0, 1, Side::Right));
        assert_eq!(false, pos.is_captured(1, 1));
        assert_eq!(1, pos.valency(1, 1));
        assert_eq!(3, pos.valency(0, 1));

        assert_eq!(true, pos.would_capture(1, 1, Side::Top));
        let outcome = pos.make_move(1, 1, Side::Top);
        assert_eq!(1, outcome.coins_captured);
        assert_eq!(false, outcome.end_of_turn);
        assert_eq!(false, pos.is_legal_move(1, 1, Side::Top));
        assert_eq!(false, pos.is_legal_move(1, 0, Side::Bottom));
        assert_eq!(true, pos.is_captured(1, 1));
        assert_eq!(0, pos.valency(1, 1));
        assert_eq!(3, pos.valency(1, 0));

        assert_eq!(false, pos.is_end_of_game());
    }

    #[test]
    fn corners() {
        let mut pos = Position::new_game(3, 3);
        for &(x, y) in [(0, 0), (0, 2), (2, 0), (2, 2)].iter() {
            let mut sides_captured = 0;
            for &s in [Side::Left, Side::Right, Side::Top, Side::Bottom].iter() {
                assert_eq!(true, pos.is_legal_move(x, y, s));
                let outcome = pos.make_move(x, y, s);
                assert_eq!(false, pos.is_legal_move(x, y, s));
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
        let mut pos = Position::new_game(2, 1);
        assert_eq!(2, pos.width());
        assert_eq!(1, pos.height());
        let moves = [(0, 0, Side::Top), (0, 0, Side::Bottom),
                     (1, 0, Side::Top), (1, 0, Side::Bottom),
                     (0, 0, Side::Left), (1, 0, Side::Right)];
        for &(x, y, s) in moves.iter() {
            assert_eq!(true, pos.is_legal_move(x, y, s));
            let outcome = pos.make_move(x, y, s);
            assert_eq!(false, pos.is_legal_move(x, y, s));
            assert_eq!(0, outcome.coins_captured);
            assert_eq!(true, outcome.end_of_turn);
            assert_eq!(false, pos.is_end_of_game());
        }
        assert_eq!(true, pos.is_legal_move(0, 0, Side::Right));
        let outcome = pos.make_move(0, 0, Side::Right);
        assert_eq!(2, outcome.coins_captured);
        assert_eq!(true, outcome.end_of_turn); // End of game
        assert_eq!(true, pos.is_end_of_game());
    }

    #[test]
    fn undo() {
        let mut pos = Position::new_game(3, 3);
        pos.make_move(1, 1, Side::Top);
        pos.make_move(1, 1, Side::Left);
        pos.undo_move(1, 1, Side::Top);
        pos.undo_move(0, 1, Side::Right);
        for i in 0..2 {
            for j in 0..2 {
                for &s in [Side::Top, Side::Bottom, Side::Left, Side::Right].iter() {
                    assert_eq!(true, pos.is_legal_move(i, j, s));
                }
            }
        }
    }

    #[test]
    fn display() {
        let mut pos = Position::new_game(3, 3);
        pos.make_move(1, 1, Side::Top);
        pos.make_move(0, 0, Side::Top);
        pos.make_move(0, 2, Side::Left);
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
    fn legal_moves() {
        let mut pos = Position::new_game(2, 2);
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

        pos.make_move(0, 0, Side::Bottom);
        let moves = pos.legal_moves();
        assert!(!moves.contains(&Move{ x: 0, y: 0, side: Side::Bottom }) &&
                !moves.contains(&Move{ x: 0, y: 1, side: Side::Top }));
    }

    #[test]
    fn offsets() {
        let pos = Position::new_game(2, 2);

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
}
