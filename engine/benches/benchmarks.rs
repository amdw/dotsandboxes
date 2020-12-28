/*
    Copyright 2020 Andrew Medworth <github@medworth.org.uk>

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
use dabengine::{game, eval};

pub fn eval_benchmark(c: &mut Criterion) {
    c.bench_function("2x2", |b| b.iter(|| eval::eval(&game::SimplePosition::new_game(2, 2))));
}

criterion_group!(eval_benches, eval_benchmark);
criterion_main!(eval_benches);
