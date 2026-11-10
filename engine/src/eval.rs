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
use crate::game::{Move, Position, SimplePosition, CompoundPosition, Side, CPosMove};
use crate::nimstring::{self, NimstringPosition};
use std::collections::HashMap;
use std::hash::Hash;

pub trait EvaluablePosition<M> : NimstringPosition<M> {
    // Given a loony position and the capture, find the corresponding double-dealing move.
    // Behaviour on a non-loony position is undefined.
    fn find_ddeal_move(&self, m: M) -> M;

    // Whether captures alone can take every remaining coin, ending the game. (Capturing only
    // makes other coins easier to capture, so the order the captures are made in doesn't matter.)
    fn can_capture_all(&self) -> bool;
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

    fn can_capture_all(self: &SimplePosition) -> bool {
        // Simulate the captures on the coins' valencies rather than making them, starting from
        // the coins capturable now. Capturing a coin cuts its last string, which reduces the
        // valency of the coin at its far end, if any, making that coin capturable at valency 1
        // (or capturing it too, at valency 0).
        let mut valency: Vec<Vec<usize>> = Vec::with_capacity(self.width());
        let mut capturable = Vec::new();
        for x in 0..self.width() {
            valency.push(Vec::with_capacity(self.height()));
            for y in 0..self.height() {
                let v = self.valency(x, y);
                valency[x].push(v);
                if v == 1 {
                    capturable.push((x, y));
                }
            }
        }
        let mut captured: Vec<Vec<bool>> = valency.iter()
            .map(|col| col.iter().map(|&v| v == 0).collect())
            .collect();
        while let Some((x, y)) = capturable.pop() {
            if captured[x][y] {
                // Captured along with a neighbour
                continue;
            }
            // The coin's last string is the one whose far end, if any, hasn't been captured
            // (capturing a coin cuts all its strings)
            let far_end = Side::ALL.into_iter()
                .filter(|&s| self.is_legal_move(Move{x: x, y: y, side: s}))
                .map(|s| self.offset(x, y, s))
                .find(|&far| far.map_or(true, |(fx, fy)| !captured[fx][fy]))
                .expect("Capturable coin should have a string left");
            captured[x][y] = true;
            if let Some((fx, fy)) = far_end {
                valency[fx][fy] -= 1;
                match valency[fx][fy] {
                    0 => captured[fx][fy] = true,
                    1 => capturable.push((fx, fy)),
                    _ => (),
                }
            }
        }
        captured.iter().all(|col| col.iter().all(|&c| c))
    }
}

impl EvaluablePosition<CPosMove> for CompoundPosition {
    fn find_ddeal_move(self: &CompoundPosition, capture: CPosMove) -> CPosMove {
        CPosMove{part: capture.part, m: self.parts[capture.part].find_ddeal_move(capture.m)}
    }

    fn can_capture_all(self: &CompoundPosition) -> bool {
        self.parts.iter().all(|p| p.can_capture_all())
    }
}

// The state of an evaluation: the cache of position values, and a limit on its size, beyond
// which the evaluation is abandoned (aborted). Once aborted, the search unwinds without
// caching anything more, so every cached value is still exact.
struct Search<'a, M> {
    cache: &'a mut HashMap<u64, (isize, M)>,
    limit: usize,
    aborted: bool,
}

// Evaluate a position given a set of moves to consider
fn eval_moves<M, P>(pos: &mut P, moves: &Vec<M>, search: &mut Search<M>) -> (isize, Option<M>)
where M: Copy, P: EvaluablePosition<M> {
    if moves.is_empty() {
        return (0, None);
    }
    let mut value = isize::MIN;
    let mut best_move = moves[0];
    for &m in moves {
        let sub_val = eval_move(pos, m, search);
        if search.aborted {
            return (0, None);
        }
        if sub_val > value {
            value = sub_val;
            best_move = m;
        }
    }
    search.cache.insert(pos.zhash(), (value, best_move));
    (value, Some(best_move))
}

// Evaluate a single move: the coins it captures, plus or minus the value of the resulting
// position depending on whether the same player moves again
fn eval_move<M, P>(pos: &mut P, m: M, search: &mut Search<M>) -> isize
where M: Copy, P: EvaluablePosition<M> {
    let outcome = pos.make_move(m);
    let sign = if outcome.coins_captured > 0 { 1 } else { -1 };
    let (next_val, _) = eval_cache(pos, search);
    pos.undo_move(m);
    (outcome.coins_captured as isize) + sign * next_val
}

// A canonical capture is one which can be made without loss, so there's no need to consider
// alternatives. In a position which isn't loony, any capture is canonical (taking a free coin
// never costs anything); in a loony position, a capture is canonical if the position stays
// loony (so it doesn't affect the option of double-dealing), or if after it every remaining
// coin can be captured (so there's nothing left to gain by double-dealing, e.g. when taking
// the last chain of the game).
//
// Captures only make other coins easier to capture, so every coin can be captured after one
// capture iff it can before it. So that condition is checked on the position itself, at most
// once per position, and only if a capture fails the cheaper conditions.
//
// This returns all the canonical captures available in a position.
pub fn canonical_moves<M, P>(pos: &P) -> Vec<M>
where M: Copy, P: EvaluablePosition<M> + Clone {
    let mut pos = pos.clone();
    let captures: Vec<M> = pos.legal_moves().into_iter().filter(|&m| pos.would_capture(m) > 0).collect();
    if !pos.is_loony() {
        return captures;
    }
    let keeping_loony: Vec<M> = captures.iter().copied().filter(|&m| nimstring::would_be_loony(&mut pos, m)).collect();
    if keeping_loony.len() < captures.len() && pos.can_capture_all() {
        return captures;
    }
    keeping_loony
}

// Determine what moves deserve consideration in a given position
fn moves_to_consider<M, P>(pos: &mut P) -> Vec<M>
where M: Copy, P: EvaluablePosition<M> {
    let legal_moves = pos.legal_moves();
    let captures: Vec<M> = legal_moves.iter().copied().filter(|&m| pos.would_capture(m) > 0).collect();
    let Some(&capture) = captures.first() else {
        // A position with no captures can't be loony
        return legal_moves;
    };

    // If there are any canonical captures, just go ahead and make one
    if !pos.is_loony() {
        return vec!(capture);
    }
    if let Some(&m) = captures.iter().find(|&&m| nimstring::would_be_loony(pos, m)) {
        return vec!(m);
    }
    if pos.can_capture_all() {
        return vec!(capture);
    }

    // Consider only capturing all and double-dealing in the loony case
    // TODO: Use other canonical play results to further reduce the set of moves considered
    let ddeal_move = pos.find_ddeal_move(capture);
    vec!(capture, ddeal_move)
}

fn eval_cache<M, P>(pos: &mut P, search: &mut Search<M>) -> (isize, Option<M>)
where M: Copy, P: EvaluablePosition<M> {
    if let Some(&(val, best_move)) = search.cache.get(&pos.zhash()) {
        return (val, Some(best_move));
    }

    let moves = moves_to_consider(pos);
    if !moves.is_empty() && search.cache.len() >= search.limit {
        search.aborted = true;
        return (0, None);
    }
    eval_moves(pos, &moves, search)
}

// An evaluator which keeps the values of the positions it has evaluated, so that later
// evaluations can reuse them: for example, as a game goes on, most of the positions
// reachable from the next position were already explored when evaluating the previous one.
// The value of a position depends only on the position, so the cache never goes stale.
pub struct Evaluator<M> {
    cache: HashMap<u64, (isize, M)>,
}

impl<M: Copy> Evaluator<M> {
    pub fn new() -> Evaluator<M> {
        Evaluator{ cache: HashMap::new() }
    }

    // Number of positions whose values are cached
    pub fn cached_positions(&self) -> usize {
        self.cache.len()
    }

    // Forget all cached values, freeing their memory
    pub fn clear(&mut self) {
        self.cache = HashMap::new();
    }

    // Calculate the value function of a given position and a move which achieves that value
    pub fn eval<P>(&mut self, pos: &P) -> (isize, Option<M>)
    where P: EvaluablePosition<M> + Clone {
        let mut pos = pos.clone();
        let mut search = Search{ cache: &mut self.cache, limit: usize::MAX, aborted: false };
        eval_cache(&mut pos, &mut search)
    }

    // Calculate the value function of a given position, along with the value achieved by
    // each legal move (i.e. the net number of coins the player to move gains from the rest
    // of the game by making that move, assuming optimal play thereafter).
    pub fn eval_with_moves<P>(&mut self, pos: &P) -> (isize, HashMap<M, isize>)
    where M: Hash + Eq, P: EvaluablePosition<M> + Clone {
        self.eval_with_moves_limited(pos, usize::MAX).unwrap()
    }

    // As eval_with_moves, but giving up (returning None) if it would need to evaluate more
    // than max_new positions which aren't already cached: i.e. evaluate it only if that's
    // cheap. The values of positions evaluated before giving up stay cached.
    pub fn eval_with_moves_limited<P>(&mut self, pos: &P, max_new: usize) -> Option<(isize, HashMap<M, isize>)>
    where M: Hash + Eq, P: EvaluablePosition<M> + Clone {
        let mut pos = pos.clone();
        let limit = self.cache.len().saturating_add(max_new);
        let mut search = Search{ cache: &mut self.cache, limit: limit, aborted: false };
        let (value, _) = eval_cache(&mut pos, &mut search);
        let mut per_move = HashMap::new();
        for m in pos.legal_moves() {
            if search.aborted {
                break;
            }
            per_move.insert(m, eval_move(&mut pos, m, &mut search));
        }
        if search.aborted { None } else { Some((value, per_move)) }
    }
}

// Calculate the value function of a given position and a move which achieves that value
pub fn eval<M, P>(pos: &P) -> (isize, Option<M>)
where M: Copy, P: EvaluablePosition<M> + Clone {
    Evaluator::new().eval(pos)
}

// Calculate the value function of a given position, along with the value achieved by
// each legal move (see Evaluator::eval_with_moves).
pub fn eval_with_moves<M, P>(pos: &P) -> (isize, HashMap<M, isize>)
where M: Copy + Hash + Eq, P: EvaluablePosition<M> + Clone {
    Evaluator::new().eval_with_moves(pos)
}

#[cfg(test)]
mod test {
    use crate::eval::*;
    use crate::examples::*;

    use rand::{RngExt, SeedableRng};
    use rand::rngs::StdRng;
    use rand::seq::SliceRandom;
    use std::cmp;
    use std::time::Instant;

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
        let width: usize = r.random_range(1..4);
        let height: usize = r.random_range(1..4);
        let mut pos = SimplePosition::new_game(width, height);
        let mut moves = pos.legal_moves();
        moves.as_mut_slice().shuffle(r);
        let max_remaining_moves = 9; // Limits running cost of naive minimax
        let min_move_count: usize = if moves.len() > max_remaining_moves {
            moves.len() - max_remaining_moves
        } else {
            0
        };
        let move_count = r.random_range(min_move_count..moves.len() + 1);
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
            if elapsed.as_secs_f64() >= test_time_s {
                break;
            }
            i += 1;
        }
    }

    // Naive value of a single move, for comparison with eval_with_moves
    fn naive_move_value(pos: &mut SimplePosition, m: Move) -> isize {
        let captures = pos.make_move(m).coins_captured as isize;
        let next_val = naive_minimax(pos);
        pos.undo_move(m);
        if captures > 0 { captures + next_val } else { -next_val }
    }

    #[test]
    fn eval_with_moves_matches_naive_minimax() {
        let test_time_s = 5 as f64;
        let mut seed: [u8; 32] = [0; 32];
        seed[0] = 77;
        let mut r: StdRng = SeedableRng::from_seed(seed);
        let start_time = Instant::now();
        let mut i = 0;
        while start_time.elapsed().as_secs_f64() < test_time_s {
            let mut pos = make_random_pos(&mut r);
            let (val, per_move) = eval_with_moves(&pos);
            assert_eq!(naive_minimax(&mut pos), val, "Position {}: {}", i, pos);
            let legal_moves = pos.legal_moves();
            assert_eq!(legal_moves.len(), per_move.len());
            for m in legal_moves {
                assert_eq!(naive_move_value(&mut pos, m), per_move[&m], "Position {}, move {}: {}", i, m, pos);
            }
            if let Some(&best) = per_move.values().max() {
                assert_eq!(val, best);
            }
            i += 1;
        }
    }

    #[test]
    fn eval_with_moves_compound() {
        // By the double-chain formula (4 - 2n), two closed 3-chains are worth -2 to the
        // player to move, and since every move opens a chain, every move has that value.
        let pos = CompoundPosition::new(vec![make_chain(3), make_chain(3)]);
        let (val, per_move) = eval_with_moves(&pos);
        assert_eq!(-2, val);
        assert_eq!(pos.legal_moves().len(), per_move.len());
        assert!(per_move.values().all(|&v| v == -2), "{:?}", per_move);
    }

    #[test]
    fn eval_with_moves_end_of_game() {
        let (val, per_move) = eval_with_moves(&SimplePosition::new_end_game(2, 2));
        assert_eq!(0, val);
        assert!(per_move.is_empty());
    }

    #[test]
    fn canonical_moves_in_chains() {
        // Closed chain: no captures at all
        assert!(canonical_moves(&make_chain(3)).is_empty());

        // Opened 3-chain: taking the first coin leaves it loony, so is canonical
        let mut chain = make_chain(3);
        chain.make_move(Move::new(0, 0, Side::Left));
        assert_eq!(vec![Move::new(0, 0, Side::Right)], canonical_moves(&chain));

        // Opened 2-chain with another chain left: taking the first coin gives up the
        // double-dealing option, so isn't canonical
        let mut chains = double_chain(2);
        chains.make_move(Move::new(0, 0, Side::Left));
        assert!(canonical_moves(&chains).is_empty());

        // But if it's the last chain, there's nothing to gain by double-dealing, so it is
        let mut chain = make_chain(2);
        chain.make_move(Move::new(0, 0, Side::Left));
        assert_eq!(vec![Move::new(0, 0, Side::Right)], canonical_moves(&chain));

        // Likewise the last two coins of a longer last chain
        let mut chain = make_chain(4);
        for x in 0..3 {
            chain.make_move(Move::new(x, 0, Side::Left));
        }
        assert_eq!(vec![Move::new(2, 0, Side::Right)], canonical_moves(&chain));

        // A single capturable coin in a position which isn't loony is free to take
        let mut chain = make_chain(1);
        chain.make_move(Move::new(0, 0, Side::Left));
        assert_eq!(vec![Move::new(0, 0, Side::Right)], canonical_moves(&chain));
    }

    // Whether captures alone can take every remaining coin, found by making them: an
    // independent check of EvaluablePosition::can_capture_all
    fn can_capture_all_by_play<M, P>(pos: &mut P) -> bool
    where M: Copy, P: EvaluablePosition<M> {
        // Making a move never makes another one illegal, so list the legal moves once, and
        // make and remove the captures among them, pass by pass, until a pass finds none.
        let mut remaining = pos.legal_moves();
        let mut made = Vec::new();
        loop {
            let made_before = made.len();
            remaining.retain(|&m| {
                if pos.would_capture(m) == 0 {
                    return true;
                }
                pos.make_move(m);
                made.push(m);
                false
            });
            if remaining.is_empty() || made.len() == made_before {
                break;
            }
        }
        let result = remaining.is_empty();
        for &m in made.iter().rev() {
            pos.undo_move(m);
        }
        result
    }

    #[test]
    fn can_capture_all_matches_play() {
        let mut seed: [u8; 32] = [0; 32];
        seed[0] = 7;
        let mut r: StdRng = SeedableRng::from_seed(seed);
        let (mut can, mut cannot) = (0, 0);
        for _ in 0..1000 {
            let mut pos = make_random_pos(&mut r);
            let expected = can_capture_all_by_play(&mut pos);
            assert_eq!(expected, pos.can_capture_all(), "{}", pos);
            if expected { can += 1 } else { cannot += 1 }
        }
        assert!(can > 0 && cannot > 0, "can {}, cannot {}", can, cannot);
    }

    #[test]
    fn canonical_moves_are_optimal() {
        // Every canonical move achieves the value of the position
        let mut seed: [u8; 32] = [0; 32];
        seed[0] = 42;
        let mut r: StdRng = SeedableRng::from_seed(seed);
        let mut checked = 0;
        for _ in 0..300 {
            let pos = make_random_pos(&mut r);
            let canonical = canonical_moves(&pos);
            if canonical.is_empty() {
                continue;
            }
            let (val, per_move) = eval_with_moves(&pos);
            for m in canonical {
                assert_eq!(val, per_move[&m], "Canonical move {} in {}", m, pos);
                checked += 1;
            }
        }
        assert!(checked > 0);
    }

    #[test]
    fn evaluator_reuse_matches_fresh_evaluation() {
        // One evaluator shared across many positions (including games played on from them)
        // must give exactly the same results as evaluating each from scratch
        let mut seed: [u8; 32] = [0; 32];
        seed[0] = 99;
        let mut r: StdRng = SeedableRng::from_seed(seed);
        let mut evaluator = Evaluator::new();
        for _ in 0..200 {
            let mut pos = CompoundPosition::from_single(make_random_pos(&mut r));
            while !pos.is_end_of_game() {
                let (val, per_move) = evaluator.eval_with_moves(&pos);
                assert_eq!(eval_with_moves(&pos), (val, per_move.clone()), "{}", pos);
                let best = per_move.iter().find(|&(_, &v)| v == val).map(|(&m, _)| m).unwrap();
                pos.make_move(best);
            }
        }
        assert!(evaluator.cached_positions() > 0);
        evaluator.clear();
        assert_eq!(0, evaluator.cached_positions());
    }

    #[test]
    fn limited_evaluation() {
        let pos = CompoundPosition::from_single(ex3p12());
        // Not enough budget: gives up, but whatever it cached is still correct
        let mut evaluator = Evaluator::new();
        assert_eq!(None, evaluator.eval_with_moves_limited(&pos, 10));
        assert!(evaluator.cached_positions() > 0);
        assert_eq!(eval_with_moves(&pos), evaluator.eval_with_moves(&pos));
        // With the position already evaluated, no new positions are needed
        assert_eq!(Some(eval_with_moves(&pos)), evaluator.eval_with_moves_limited(&pos, 0));
        // Positions whose values are already known to cost nothing extra: end of game
        let end = CompoundPosition::from_single(SimplePosition::new_end_game(2, 2));
        assert_eq!(Some((0, HashMap::new())), Evaluator::new().eval_with_moves_limited(&end, 0));
    }

    #[test]
    fn limited_evaluation_after_aborts_matches_fresh() {
        // Repeatedly giving up partway, on random positions, never leaves wrong values cached
        let mut seed: [u8; 32] = [0; 32];
        seed[0] = 7;
        let mut r: StdRng = SeedableRng::from_seed(seed);
        let mut evaluator = Evaluator::new();
        for i in 0..300 {
            let pos = CompoundPosition::from_single(make_random_pos(&mut r));
            let _ = evaluator.eval_with_moves_limited(&pos, i % 7);
            assert_eq!(eval_with_moves(&pos), evaluator.eval_with_moves(&pos), "{}", pos);
        }
    }

    #[test]
    fn evaluator_reuse_saves_work() {
        // After evaluating a position, evaluating the position after the best move
        // finds almost everything it needs already cached
        let mut pos = CompoundPosition::from_single(ex3p12());
        let mut evaluator = Evaluator::new();
        let (_, best) = evaluator.eval(&pos);
        let after_first = evaluator.cached_positions();
        pos.make_move(best.unwrap());
        evaluator.eval_with_moves(&pos);
        let added = evaluator.cached_positions() - after_first;
        let mut fresh = Evaluator::new();
        fresh.eval_with_moves(&pos);
        assert!(added * 2 < fresh.cached_positions(), "added {} vs fresh {}", added, fresh.cached_positions());
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
