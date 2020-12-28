/*
    Copyright 2017, 2020 Andrew Medworth <github@medworth.org.uk>

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
use dabengine::cli;
use std::env;
use std::process;

fn usage(name: &str) {
    println!("Usage:");
    println!("{} x y - start a new game of width x, height y", name);
    println!("{} cmd_file - read commands from cmd_file and start CLI from there", name);
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 3 {
        let width = args[1].parse::<usize>().unwrap();
        let height = args[2].parse::<usize>().unwrap();
        cli::main_loop_start(width, height);
    }
    else if args.len() == 2 {
        let filename = &args[1];
        cli::main_loop_file(filename);
    }
    else {
        usage(&args[0]);
        process::exit(1);
    }
}
