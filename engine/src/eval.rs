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
use game::{Move, Position, Side};
use nimstring;
use std::cmp;
use std::collections::HashMap;

// Evaluate a position given a set of moves to consider
fn eval_moves(pos: &mut Position, moves: &Vec<Move>, cache: &mut HashMap<usize, isize>) -> isize {
    if moves.is_empty() {
        return 0;
    }
    let mut sub_vals = Vec::with_capacity(moves.len());
    for m in moves {
        let outcome = pos.make_move(m.x, m.y, m.side);
        let sign = if outcome.coins_captured > 0 { 1 } else { -1 };
        sub_vals.push((outcome.coins_captured as isize) + sign * eval_cache(pos, cache));
        pos.undo_move(m.x, m.y, m.side);
    }
    let result = sub_vals.iter().fold(sub_vals[0], |a, &v| cmp::max(a, v));
    cache.insert(pos.zhash(), result);
    result
}

// Given a loony position and the capture, find the corresponding double-dealing move
fn find_ddeal_move(pos: &Position, capture: Move) -> Move {
    // (capture.x, capture.y) might be the valency-1 coin or the valency-2 one
    let (v2_x, v2_y, excl_side) = if pos.valency(capture.x, capture.y) == 1 {
        let (x, y) = pos.offset(capture.x, capture.y, capture.side).unwrap();
        (x, y, capture.side.opposite())
    } else {
        (capture.x, capture.y, capture.side)
    };

    for s in Side::all_except(excl_side) {
        if pos.is_legal_move(v2_x, v2_y, s) {
            return Move{x: v2_x, y: v2_y, side: s};
        }
    }

    panic!("Could not find double-dealing move corresponding to {} in {}", capture, pos);
}

// Determine what moves deserve consideration in a given position
fn moves_to_consider(pos: &mut Position) -> Vec<Move> {
    let legal_moves = pos.legal_moves();

    // If there are any captures which don't affect looniness, just go ahead and make those
    let is_loony = nimstring::is_loony(pos);
    let mut capture: Option<Move> = None;
    for &m in &legal_moves {
        let captures = pos.would_capture(m.x, m.y, m.side);
        if captures == 0 {
            continue;
        }
        capture = Some(m);
        if !is_loony || nimstring::would_be_loony(pos, m.x, m.y, m.side) {
            return vec!(m);
        }
    }

    // Consider only capturing all and double-dealing in the loony case
    // TODO: Use other canonical play results to further reduce the set of moves considered
    if is_loony {
        let capture = capture.unwrap();
        let ddeal_move = find_ddeal_move(pos, capture);
        vec!(capture, ddeal_move)
    } else {
        legal_moves
    }
}

fn eval_cache(pos: &mut Position, cache: &mut HashMap<usize, isize>) -> isize {
    // TODO: Use alpha-beta pruning
    if let Some(&cached) = cache.get(&pos.zhash()) {
        return cached;
    }

    let moves = moves_to_consider(pos);
    eval_moves(pos, &moves, cache)
}

// Calculate the value function of a given position
pub fn eval(pos: &Position) -> isize {
    let mut cache = HashMap::new();
    let mut pos = pos.clone();
    eval_cache(&mut pos, &mut cache)
}

#[cfg(test)]
mod test {
    use eval::*;
    use examples::*;

    #[test]
    fn eval_chain() {
        for i in 1..10 {
            let mut chain = make_chain(i);
            assert_eq!(-(i as isize), eval(&mut chain));
        }
    }

    #[test]
    fn eval_double_chain() {
        assert_eq!(0, eval(&mut double_chain(1)));
        for i in 2..10 {
            let mut pos = double_chain(i);
            assert_eq!(4 - 2*(i as isize), eval(&mut pos), "Evaluation of double chain length {}", i);
        }
    }

    #[test]
    fn eval_double_loop() {
        for i in 2..8 {
            let mut pos = double_loop(i);
            assert_eq!(8 - 4*(i as isize), eval(&mut pos), "Evaluation of double loop width {}", i);
        }
    }
}
