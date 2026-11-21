/*
    Copyright 2026 Andrew Medworth <github@medworth.org.uk>

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

// Time the evaluation of a game: solve <width> <height> [<max cached positions> [<moves> [<seed>]]]
// The game is a new one, or one in which the given number of moves have been played at
// random (never offering a coin if that can be avoided), to give a middle game position.
// No limit on the cached positions is given as 0.
//
// This is for investigating the evaluator's performance, which the benchmarks
// (benches/benchmarks.rs) don't suit: they track the time of a fixed set of evaluations
// across changes. This evaluates once, so it can be used on positions which take minutes
// (a new 4x4 game) or would take Criterion's minimum ten samples too long. It also takes
// the board, cache limit and position from the command line, for exploring how the search
// scales and how it behaves when the cache is too small. And it reports the number of
// positions evaluated and forgotten as well as the time, which shows whether a change
// saves search or only makes each step cheaper.
use dabengine::eval::Evaluator;
use dabengine::game::{Move, Position, SimplePosition};
use std::time::Instant;

fn offers_coin(pos: &SimplePosition, m: Move) -> bool {
    pos.valency(m.x, m.y) <= 2 || pos.offset(m.x, m.y, m.side).is_some_and(|(x, y)| pos.valency(x, y) <= 2)
}

fn main() {
    let a: Vec<usize> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let mut pos = SimplePosition::new_game(a[0], a[1]);
    let mut seed = a.get(4).copied().unwrap_or(1) as u64;
    for _ in 0..a.get(3).copied().unwrap_or(0) {
        let legal = pos.legal_moves();
        let safe: Vec<Move> = legal.iter().copied().filter(|&m| !offers_coin(&pos, m)).collect();
        let moves = if safe.is_empty() { legal } else { safe };
        if moves.is_empty() {
            break;
        }
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        pos.make_move(moves[(seed >> 33) as usize % moves.len()]);
    }
    let mut e: Evaluator<Move> = match a.get(2) {
        Some(&max_cached) if max_cached > 0 => Evaluator::with_max_cached(max_cached),
        _ => Evaluator::new(),
    };
    let t = Instant::now();
    let (v, m) = e.eval(&pos);
    println!("{}x{}: value {} move {:?} time {:.2?} positions {} forgotten {}", a[0], a[1], v, m.map(|m| m.to_string()), t.elapsed(), e.positions_evaluated(), e.positions_forgotten());
}
