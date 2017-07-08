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
    along with Foobar.  If not, see <http://www.gnu.org/licenses/>.
*/
use std::fmt;

#[derive(Clone)]
#[derive(Copy)]
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
    pub end_of_turn: bool
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
    pub fn new_game(m: usize, n: usize) -> Position {
        let mut top_strings = Vec::with_capacity(m);
        let mut left_strings = Vec::with_capacity(n);
        let mut down_strings = Vec::with_capacity(m);
        let mut right_strings = Vec::with_capacity(m);

        for i in 0..m {
            top_strings.push(true);
            down_strings.push(Vec::with_capacity(n));
            right_strings.push(Vec::with_capacity(n));
            for _ in 0..n {
                down_strings[i].push(true);
                right_strings[i].push(true);
            }
        }
        for _ in 0..n {
            left_strings.push(true);
        }
        Position {
            top_strings: top_strings,
            left_strings: left_strings,
            down_strings: down_strings,
            right_strings: right_strings,
        }
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
}

impl fmt::Display for Position {
    fn fmt(self: &Position, f: &mut fmt::Formatter) -> fmt::Result {
        for &b in self.top_strings.iter() {
            write!(f, "+{}", if b { " " } else { "-" })?;
        }
        write!(f, "+\n")?;
        for j in 0..self.left_strings.len() {
            write!(f, "{} ", if self.left_strings[j] { " " } else { "|" })?;
            for i in 0..self.top_strings.len() {
                write!(f, "{} ", if self.right_strings[i][j] { " " } else { "|" })?;
            }
            write!(f, "\n")?;
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

        let outcome = pos.make_move(1, 1, Side::Bottom);
        assert_eq!(0, outcome.coins_captured);
        assert_eq!(true, outcome.end_of_turn);
        assert_eq!(false, pos.is_legal_move(1, 1, Side::Bottom));
        assert_eq!(false, pos.is_legal_move(1, 2, Side::Top));
        assert_eq!(false, pos.is_captured(1, 1));

        let outcome = pos.make_move(1, 1, Side::Left);
        assert_eq!(0, outcome.coins_captured);
        assert_eq!(true, outcome.end_of_turn);
        assert_eq!(false, pos.is_legal_move(1, 1, Side::Left));
        assert_eq!(false, pos.is_legal_move(0, 1, Side::Right));
        assert_eq!(false, pos.is_captured(1, 1));

        let outcome = pos.make_move(1, 1, Side::Top);
        assert_eq!(1, outcome.coins_captured);
        assert_eq!(false, outcome.end_of_turn);
        assert_eq!(false, pos.is_legal_move(1, 1, Side::Top));
        assert_eq!(false, pos.is_legal_move(1, 0, Side::Bottom));
        assert_eq!(true, pos.is_captured(1, 1));

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
    fn display() {
        let mut pos = Position::new_game(3, 3);
        pos.make_move(1, 1, Side::Top);
        pos.make_move(0, 0, Side::Top);
        pos.make_move(0, 2, Side::Left);
        let display = format!("{}", pos);
        let expected = vec!("+-+ + +", "        ", "+ +-+ +", "        ", "+ + + +", "|       ", "+ + + +", "");
        let actual: Vec<&str> = display.split("\n").collect();
        assert_eq!(expected, actual);
    }
}
