/*
    Copyright 2020-2026 Andrew Medworth <github@medworth.org.uk>

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
use criterion::{criterion_group, criterion_main, Criterion};
use dabengine::{game, eval, examples, nimstring};
use dabengine::game::{Move, Position, Side};
use std::time::Duration;

// A middle game position: a new game after the given moves. These are the moves
// examples/solve.rs plays for the same board and number of moves with the default seed.
fn middle_game(width: usize, height: usize, moves: &[(usize, usize, Side)]) -> game::SimplePosition {
    let mut pos = game::SimplePosition::new_game(width, height);
    for &(x, y, side) in moves {
        pos.make_move(Move { x, y, side });
    }
    pos
}

fn middle_game_3x4() -> game::SimplePosition {
    middle_game(3, 4, &[(2, 0, Side::Right), (0, 0, Side::Left), (0, 2, Side::Right), (2, 0, Side::Top),
                        (0, 2, Side::Bottom), (0, 3, Side::Bottom)])
}

fn middle_game_4x5() -> game::SimplePosition {
    middle_game(4, 5, &[(1, 1, Side::Right), (0, 0, Side::Bottom), (2, 4, Side::Right), (2, 0, Side::Top),
                        (0, 1, Side::Bottom), (2, 3, Side::Bottom), (0, 3, Side::Right), (3, 0, Side::Bottom),
                        (0, 0, Side::Right), (3, 2, Side::Bottom), (3, 1, Side::Right), (2, 2, Side::Right),
                        (0, 4, Side::Left), (2, 3, Side::Right)])
}

pub fn eval_benchmark(c: &mut Criterion) {
    let mut fast_group = c.benchmark_group("fast-evaluations");
    fast_group.measurement_time(Duration::from_secs(70));
    fast_group.bench_function("Eval simple 2x2", |b| b.iter(|| eval::eval(&game::SimplePosition::new_game(2, 2))));
    fast_group.bench_function("Eval composite OLMT", |b| b.iter(|| eval::eval(&examples::one_long_multi_three(5, 4))));
    fast_group.bench_function("Eval simple 2x4", |b| b.iter(|| eval::eval(&game::SimplePosition::new_game(2, 4))));
    let pos = middle_game_3x4();
    fast_group.bench_function("Eval middle game 3x4", |b| b.iter(|| eval::eval(&pos)));
    fast_group.bench_function("Eval nimber simple 2x2", |b| b.iter(|| nimstring::calc_value_with_moves(&game::SimplePosition::new_game(2, 2))));
    fast_group.bench_function("Eval nimber composite OLMT", |b| b.iter(|| nimstring::calc_value_with_moves(&examples::one_long_multi_three(5, 4))));
    fast_group.finish();

    let mut slow_group = c.benchmark_group("slow-evaluations");
    slow_group.sample_size(10);
    slow_group.measurement_time(Duration::from_secs(360));
    slow_group.bench_function("Eval simple 3x3", |b| b.iter(|| eval::eval(&game::SimplePosition::new_game(3, 3))));
    slow_group.bench_function("Eval simple 2x5", |b| b.iter(|| eval::eval(&game::SimplePosition::new_game(2, 5))));
    let pos = middle_game_4x5();
    slow_group.bench_function("Eval middle game 4x5", |b| b.iter(|| eval::eval(&pos)));
    slow_group.finish();
}

criterion_group!(eval_benches, eval_benchmark);
criterion_main!(eval_benches);
