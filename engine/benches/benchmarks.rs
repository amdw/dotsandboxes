/*
    Copyright 2020-2022 Andrew Medworth <github@medworth.org.uk>

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
use dabengine::{game, eval, examples};
use std::time::Duration;

pub fn eval_benchmark(c: &mut Criterion) {
    let mut fast_group = c.benchmark_group("fast-evaluations");
    fast_group.measurement_time(Duration::from_secs(60));
    fast_group.bench_function("Eval simple 2x2", |b| b.iter(|| eval::eval(&game::SimplePosition::new_game(2, 2))));
    fast_group.bench_function("Eval composite OLMT", |b| b.iter(|| eval::eval(&examples::one_long_multi_three(5, 4))));
    fast_group.finish();

    let mut slow_group = c.benchmark_group("slow-evaluations");
    slow_group.sample_size(10);
    slow_group.measurement_time(Duration::from_secs(360));
    slow_group.bench_function("Eval simple 3x3", |b| b.iter(|| eval::eval(&game::SimplePosition::new_game(3, 3))));
    slow_group.finish();
}

criterion_group!(eval_benches, eval_benchmark);
criterion_main!(eval_benches);
