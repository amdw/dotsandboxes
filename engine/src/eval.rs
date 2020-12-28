/*
    Copyright 2017-2020 Andrew Medworth <github@medworth.org.uk>

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
use game::{Move, Position, SimplePosition, CompoundPosition, Side, CPosMove};
use nimstring::{self, NimstringPosition};
use std::collections::HashMap;
use std::isize;

pub trait EvaluablePosition<M> : NimstringPosition<M> {
    // Given a loony position and the capture, find the corresponding double-dealing move.
    // Behaviour on a non-loony position is undefined.
    fn find_ddeal_move(&self, m: M) -> M;
}

impl EvaluablePosition<Move> for SimplePosition {
    fn find_ddeal_move(self: &SimplePosition, capture: Move) -> Move {
        // (capture.x, capture.y) might be the valency-1 coin or the valency-2 one
        let (v2_x, v2_y, excl_side) = if self.valency(capture.x, capture.y) == 1 {
            let (x, y) = self.offset(capture.x, capture.y, capture.side).unwrap();
            (x, y, capture.side.opposite())
        } else {
            (capture.x, capture.y, capture.side)
        };

        if self.valency(v2_x, v2_y) != 2 {
            panic!("Expected ({},{}) to have valency 2, found {} in {}",
                   v2_x, v2_y, self.valency(v2_x, v2_y), self);
        }

        for s in Side::all_except(excl_side) {
            if self.is_legal_move(Move{x: v2_x, y: v2_y, side: s}) {
                return Move{x: v2_x, y: v2_y, side: s};
            }
        }

        panic!("Could not find double-dealing move corresponding to {} in {}", capture, self);
    }
}

impl EvaluablePosition<CPosMove> for CompoundPosition {
    fn find_ddeal_move(self: &CompoundPosition, capture: CPosMove) -> CPosMove {
        CPosMove{part: capture.part, m: self.parts[capture.part].find_ddeal_move(capture.m)}
    }
}

// Evaluate a position given a set of moves to consider
fn eval_moves<M, P>(pos: &mut P, moves: &Vec<M>,
              cache: &mut HashMap<usize, (isize, M)>) -> (isize, Option<M>)
where M: Copy, P: EvaluablePosition<M> {
    if moves.is_empty() {
        return (0, None);
    }
    let mut value = isize::MIN;
    let mut best_move = moves[0];
    for &m in moves {
        let outcome = pos.make_move(m);
        let sign = if outcome.coins_captured > 0 { 1 } else { -1 };
        let (next_val, _) = eval_cache(pos, cache);
        let sub_val = (outcome.coins_captured as isize) + sign * next_val;
        pos.undo_move(m);
        if sub_val > value {
            value = sub_val;
            best_move = m;
        }
    }
    cache.insert(pos.zhash(), (value, best_move));
    (value, Some(best_move))
}

// Determine what moves deserve consideration in a given position
fn moves_to_consider<M, P>(pos: &mut P) -> Vec<M>
where M: Copy, P: EvaluablePosition<M> {
    let legal_moves = pos.legal_moves();

    // If there are any captures which don't affect looniness, just go ahead and make those
    let is_loony = pos.is_loony();
    let mut capture: Option<M> = None;
    for &m in &legal_moves {
        let captures = pos.would_capture(m);
        if captures == 0 {
            continue;
        }
        capture = Some(m);
        if !is_loony || nimstring::would_be_loony(pos, m) {
            return vec!(m);
        }
    }

    // Consider only capturing all and double-dealing in the loony case
    // TODO: Use other canonical play results to further reduce the set of moves considered
    if is_loony {
        let capture = capture.unwrap();
        let ddeal_move = pos.find_ddeal_move(capture);
        vec!(capture, ddeal_move)
    } else {
        legal_moves
    }
}

fn eval_cache<M, P>(pos: &mut P, cache: &mut HashMap<usize, (isize, M)>) -> (isize, Option<M>)
where M: Copy, P: EvaluablePosition<M> {
    if let Some(&(val, best_move)) = cache.get(&pos.zhash()) {
        return (val, Some(best_move));
    }

    let moves = moves_to_consider(pos);
    eval_moves(pos, &moves, cache)
}

// Calculate the value function of a given position and a move which achieves that value
pub fn eval<M, P>(pos: &P) -> (isize, Option<M>)
where M: Copy, P: EvaluablePosition<M> + Clone {
    let mut cache = HashMap::new();
    let mut pos = pos.clone();
    eval_cache(&mut pos, &mut cache)
}

#[cfg(test)]
mod test {
    use eval::*;
    use examples::*;

    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;
    use rand::seq::SliceRandom;
    use std::cmp;
    use time::Instant;

    #[test]
    fn eval_chain() {
        for i in 1..10 {
            let mut chain = make_chain(i);
            let (val, _) = eval(&chain);
            let expected_val = -(i as isize);
            assert_eq!(expected_val, val, "Closed {}-chain", i);
            chain.make_move(Move{x: 0, y: 0, side: Side::Left});
            let (val, best_move) = eval(&chain);
            assert_eq!(-expected_val, val, "Opened {}-chain", i);
            assert!(chain.moves_equivalent(best_move.unwrap(), Move{x: 0, y: 0, side: Side::Right}));
        }
    }

    #[test]
    fn eval_double_chain() {
        let (val, _) = eval(&double_chain(1));
        assert_eq!(0, val);
        for i in 2..10 {
            let mut pos = double_chain(i);
            let (val, _) = eval(&pos);
            let expected_val = 4 - 2*(i as isize);
            assert_eq!(expected_val, val, "Evaluation of double chain length {}", i);
            pos.make_move(Move{x: 0, y: 0, side: Side::Left});
            let (val, best_move) = eval(&pos);
            assert_eq!(-expected_val, val, "Evaluation of opened double chain length {}", i);
            assert!(pos.moves_equivalent(best_move.unwrap(), Move{x: 0, y: 0, side: Side::Right}));
        }
    }

    #[test]
    fn eval_multi_chains() {
        let mut pos = multi_chains(3, 4);

        let (val, _) = eval(&pos);
        assert_eq!(-2, val);
        pos.make_move(Move{x: 0, y: 0, side: Side::Left});

        let (val, best_move) = eval(&pos);
        assert_eq!(2, val);
        assert!(pos.moves_equivalent(best_move.unwrap(), Move{x: 0, y: 0, side: Side::Right}));
        pos.make_move(Move{x: 0, y: 0, side: Side::Right});

        let (val, best_move) = eval(&pos);
        assert_eq!(1, val);
        assert!(pos.moves_equivalent(best_move.unwrap(), Move{x: 1, y: 0, side: Side::Right}));
        pos.make_move(Move{x: 1, y: 0, side: Side::Right});

        let (val, best_move) = eval(&pos);
        assert_eq!(0, val);
        assert!(pos.moves_equivalent(best_move.unwrap(), Move{x: 2, y: 0, side: Side::Right}));
        pos.make_move(Move{x: 2, y: 0, side: Side::Right});

        let (val, _) = eval(&pos);
        assert_eq!(-1, val);
    }

    #[test]
    fn eval_double_loop() {
        for i in 2..8 {
            let mut pos = double_loop(i);
            let (val, _) = eval(&pos);
            let expected_val = 8 - 4*(i as isize);
            assert_eq!(expected_val, val, "Evaluation of double loop width {}", i);
            pos.make_move(Move{x: 0, y: 0, side: Side::Right});
            let (val, _) = eval(&pos);
            assert_eq!(-expected_val, val, "Evaluation of opened double loop width {}", i);
        }
    }

    #[test]
    fn eval_ex3p1() {
        let mut pos = ex3p1();
        let (val, best_move) = eval(&pos);
        let best_move = best_move.unwrap();
        assert_eq!(3, val);
        assert!(pos.moves_equivalent(best_move, Move{x: 2, y: 1, side: Side::Bottom}));

        pos.make_move(Move{x: 2, y: 1, side: Side::Bottom});
        let (val, best_move) = eval(&pos);
        let best_move = best_move.unwrap();
        assert_eq!(-3, val);
        assert!(pos.moves_equivalent(best_move, Move{x: 0, y: 0, side: Side::Bottom}));

        pos.make_move(Move{x: 0, y: 0, side: Side::Bottom});
        let (val, _) = eval(&pos);
        assert_eq!(3, val);

        pos.undo_move(Move{x: 0, y: 0, side: Side::Bottom});
        pos.make_move(Move{x: 0, y: 2, side: Side::Left});
        let (val, _) = eval(&pos);
        assert_eq!(5, val);
    }

    #[test]
    fn eval_ex3p12() {
        let mut pos = ex3p12();
        let (val, best_move) = eval(&pos);
        let best_move = best_move.unwrap();
        let expected_val = 9;
        assert_eq!(expected_val, val);
        assert!(pos.moves_equivalent(best_move, Move{x: 4, y: 0, side: Side::Bottom}));

        pos.make_move(Move{x: 4, y: 0, side: Side::Bottom});
        let (val, _) = eval(&pos);
        assert_eq!(-expected_val, val);
    }

    #[derive(Debug)]
    struct OLMTCase {
        i: usize,
        k: usize,
        exp_val: isize,
        taking_optimal: bool,
        ddeal_optimal: bool,
    }

    impl OLMTCase {
        fn new(i: usize, k: usize, exp_val: isize,
            taking_optimal: bool, ddeal_optimal: bool) -> OLMTCase {
            OLMTCase{i: i, k: k, exp_val: exp_val,
                taking_optimal: taking_optimal, ddeal_optimal: ddeal_optimal}
        }
    }

    #[test]
    fn eval_one_long_multi_three() {
        // Check the P_{i,4} table from the paper
        let cases = vec!(
            OLMTCase::new(0, 4, -4, true, false),
            OLMTCase::new(1, 4, -3, false, true),
            OLMTCase::new(2, 4, -2, false, true),
            OLMTCase::new(3, 4, -1, true, true),
            OLMTCase::new(4, 4, -2, true, false),
            OLMTCase::new(5, 4, -1, true, true),
            OLMTCase::new(6, 4, -2, true, false),
            OLMTCase::new(7, 4, -1, true, true),
            OLMTCase::new(8, 4, -2, true, false),
        );

        for case in cases.iter() {
            let mut pos = one_long_multi_three(case.i, case.k);
            let (val, _) = eval(&pos);
            assert_eq!(case.exp_val, val, "{:?}", case);

            let part = if case.i == 0 { 0 } else { 1 };
            let open = CPosMove::new(part, 0, 0, Side::Left);
            pos.make_move(open);
            // Once the first chain is opened, capturing the first coin
            // is the only move to consider
            assert_eq!(1, moves_to_consider(&mut pos).len());
            let (val, best_move) = eval(&pos);
            assert_eq!(-case.exp_val, val, "{:?}", case);
            assert_eq!(1, pos.make_move(best_move.unwrap()).coins_captured);

            let (val, best_move) = eval(&pos);
            if case.i > 0 {
                assert_eq!(2, moves_to_consider(&mut pos).len());
            }
            assert_eq!(-case.exp_val-1, val, "{:?}", case);

            let mut ok_moves: Vec<CPosMove> = Vec::with_capacity(2);
            if case.taking_optimal {
                ok_moves.push(CPosMove::new(part, 1, 0, Side::Right));
            }
            if case.ddeal_optimal {
                ok_moves.push(CPosMove::new(part, 2, 0, Side::Right));
            }
            assert!(ok_moves.iter().any(|&m| pos.moves_equivalent(m, best_move.unwrap())),
                "{:?} {:?} {}", case, ok_moves, best_move.unwrap());
        }
    }

    #[test]
    fn multiple_loony_parts() {
        let mut pos = CompoundPosition::new(vec!(make_chain(2), make_chain(2)));
        pos.make_move(CPosMove::new(0, 0, 0, Side::Left));
        pos.make_move(CPosMove::new(1, 0, 0, Side::Left));
        assert_eq!(1, moves_to_consider(&mut pos).len());
    }

    // For use in generative tests
    fn make_random_pos(r: &mut StdRng) -> SimplePosition {
        let width: usize = r.gen_range(1..4);
        let height: usize = r.gen_range(1..4);
        let mut pos = SimplePosition::new_game(width, height);
        let mut moves = pos.legal_moves();
        moves.as_mut_slice().shuffle(r);
        let max_remaining_moves = 9; // Limits running cost of naive minimax
        let min_move_count: usize = if moves.len() > max_remaining_moves {
            moves.len() - max_remaining_moves
        } else {
            0
        };
        let move_count = r.gen_range(min_move_count..moves.len() + 1);
        for i in 0..move_count {
            let m = moves[i];
            pos.make_move(m);
        }
        pos
    }

    // Dumb evaluation algorithm to compare to optimised one
    fn naive_minimax(pos: &mut SimplePosition) -> isize {
        let moves = pos.legal_moves();
        if moves.is_empty() {
            return 0;
        }
        let mut result = isize::MIN;
        for &m in &moves {
            let outcome = pos.make_move(m);
            let captures = outcome.coins_captured as isize;
            let m_val = if captures > 0 {
                captures + naive_minimax(pos)
            } else {
                -naive_minimax(pos)
            };
            pos.undo_move(m);
            result = cmp::max(result, m_val);
        }
        result
    }

    #[test]
    fn matches_naive_minimax() {
        let test_time_s = 10 as f64;
        let mut seed: [u8; 32] = [0; 32];
        seed[0] = 123;
        let mut r: StdRng = SeedableRng::from_seed(seed);
        let mut i = 0;
        let start_time = Instant::now();
        loop {
            let mut pos = make_random_pos(&mut r);
            let expected_val = naive_minimax(&mut pos);
            let (val, best_move) = eval(&pos);
            assert_eq!(expected_val, val, "SimplePosition {} value matches naive minimax", i);

            if !pos.is_end_of_game() {
                let best_move = best_move.unwrap();
                let outcome = pos.make_move(best_move);
                let captures = outcome.coins_captured as isize;
                let (next_val, _) = eval(&pos);
                let expected_next_val = if captures > 0 {
                    expected_val - captures
                } else {
                    -expected_val
                };
                assert_eq!(expected_next_val, next_val);
            }

            let elapsed = start_time.elapsed();
            if elapsed.as_seconds_f64() >= test_time_s {
                break;
            }
            i += 1;
        }
    }

    // TODO: Test that rotations and reflections do not affect the evaluation

//    #[test]
//    fn eval_p50() {
//        let mut pos = p50();
//        let (val, best_move) = eval(&pos);
//        let best_move = best_move.unwrap();
//        assert_eq!(4, val);
//        assert!(pos.moves_equivalent(best_move, Move{x: 0, y: 3, side: Side::Right})
//                || pos.moves_equivalent(best_move, Move{x: 0, y: 3, side: Side::Bottom})
//                || pos.moves_equivalent(best_move, Move{x: 0, y: 3, side: Side::Left}));
//    }
}
