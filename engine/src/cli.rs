/*
    Copyright 2017-2019 Andrew Medworth <github@medworth.org.uk>

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

use game::{Move, Position, SimplePosition, Side};
use nimstring;
use eval;

use std::io::{self, BufRead};
use std::fs::File;
use regex::{Regex, Captures};
use time;

#[derive(PartialEq)]
#[derive(Debug)]
enum Command {
    MakeMove(Move),
    UndoMove(Move),
    CalcNimstringValue,
    Evaluate,
    PrintHelp,
    Quit,
}

impl Command {
    fn execute(self: &Command, pos: &mut SimplePosition) {
        match self {
            &Command::MakeMove(m) => {
                if pos.is_legal_move(m) {
                    pos.make_move(m);
                }
                else {
                    println!("Not a legal move: {}", m);
                }
            },
            &Command::UndoMove(m) => { pos.undo_move(m); },
            &Command::CalcNimstringValue => {
                let (val, per_move) = nimstring::calc_value_with_moves(pos);
                println!("Position value is {}", val);
                let mut moves: Vec<&Move> = per_move.keys().collect();
                moves.sort_by(|a, b| a.y.cmp(&b.y).then(a.x.cmp(&b.x)).then(a.side.cmp(&b.side)));
                for &m in &moves {
                    println!("{} {}", m, per_move.get(m).unwrap());
                }
            },
            &Command::Evaluate => {
                let (val, best_move) = eval::eval(pos);
                if let Some(best_move) = best_move {
                    println!("V(P) = {}, best move {}", val, best_move);
                } else {
                    println!("V(P) = {}", val);
                }
            },
            &Command::PrintHelp => { print_help(); },
            &Command::Quit => { println!("Bye bye!"); },
        }
    }
}

fn print_help() {
    println!("Available commands:");
    println!("x y t/l/b/r - make move (x,y) top/left/bottom/right");
    println!("u x y t/l/b/r - undo move (x,y) top/left/bottom/right");
    println!("nv - calculate Nimstring value of current position");
    println!("eval - evaluate the current position");
    println!("help - print this help message");
    println!("quit/exit - exit program");
}

fn parse_side(side_s: &str) -> Option<Side> {
    if side_s == "l" || side_s == "left" { Some(Side::Left) }
    else if side_s == "r" || side_s == "right" { Some(Side::Right) }
    else if side_s == "t" || side_s == "top" { Some(Side::Top) }
    else if side_s == "b" || side_s == "bottom" { Some(Side::Bottom) }
    else { None }
}

fn parse_move(caps: Captures) -> Result<Move, String> {
    let x = caps[1].parse::<usize>().unwrap();
    let y = caps[2].parse::<usize>().unwrap();
    let side_s = caps[3].to_string();
    match parse_side(&side_s) {
        Some(side) => Ok(Move{x: x, y: y, side: side}),
        None => Err(format!("Unrecognised side: [{}]", side_s))
    }
}

fn parse_command(input: &str) -> Result<Command, String> {
    let input = input.to_lowercase();
    let move_re = Regex::new(r"^(\d+) (\d+) ([a-zA-Z]+)$").unwrap();
    if let Some(caps) = move_re.captures(&input) {
        match parse_move(caps) {
            Ok(m) => return Ok(Command::MakeMove(m)),
            Err(e) => return Err(format!("Cannot extract move from [{}]: {}", input, e))
        }
    }
    let undo_move_re = Regex::new(r"^[uU] (\d+) (\d+) ([a-zA-Z]+)$").unwrap();
    if let Some(caps) = undo_move_re.captures(&input) {
        match parse_move(caps) {
            Ok(m) => return Ok(Command::UndoMove(m)),
            Err(e) => return Err(format!("Cannot extract move from [{}]: {}", input, e))
        }
    }
    if "nv" == input {
        return Ok(Command::CalcNimstringValue);
    }
    if "eval" == input {
        return Ok(Command::Evaluate);
    }
    if "help" == input {
        return Ok(Command::PrintHelp);
    }
    if "exit" == input || "quit" == input {
        return Ok(Command::Quit);
    }
    Err("Unsupported command".to_string())
}

fn get_next_command() -> Command {
    loop {
        let mut input = String::new();
        if let Err(error) = io::stdin().read_line(&mut input) {
            println!("Error reading from standard input: {}", error);
            continue;
        }
        let input = input.trim();
        match parse_command(&input) {
            Ok(command) => return command,
            Err(error) => {
                println!("Cannot execute [{}]: {}", input, error);
                println!("For help, try 'help'");
            }
        }
    }
}

fn main_loop_from(pos: &mut SimplePosition) {
    loop {
        println!("{}", pos);
        let command = get_next_command();
        let start_time = time::precise_time_s();
        command.execute(pos);
        let end_time = time::precise_time_s();
        if command == Command::Quit {
            break;
        }
        let elapsed = end_time - start_time;
        if elapsed >= 0.1 {
            println!("({:.1} seconds)", end_time - start_time);
        }
        println!();
    }
}

// Enter the main loop of the CLI from the start of the game
pub fn main_loop_start(width: usize, height: usize) {
    main_loop_from(&mut SimplePosition::new_game(width, height));
}

// Execute a given file of commands (which must have the dimensions of the position on the first line)
// and then enter the CLI main loop.
pub fn main_loop_file(filename: &str) {
    let f = File::open(filename).expect(&format!("Could not open file [{}]", filename));
    let reader = io::BufReader::new(f);
    let mut lines = reader.lines();
    let size_spec = lines.next().expect("Empty file").expect("Could not read board size from first line");
    let size_re = Regex::new(r"^(\d+) (\d+)$").unwrap();
    let size_caps = size_re.captures(&size_spec).expect(&format!("Could not read board size from [{}]", size_spec));
    let width = size_caps[1].parse::<usize>().unwrap();
    let height = size_caps[2].parse::<usize>().unwrap();
    let mut pos = SimplePosition::new_game(width, height);
    for line in lines {
        let line = line.expect("Could not read line from file");
        if line.trim().len() == 0 {
            continue;
        }
        match parse_command(&line) {
            Ok(command) => command.execute(&mut pos),
            Err(error) => panic!("Cannot execute [{}]: {}", line, error),
        }
    }
    main_loop_from(&mut pos);
}

#[cfg(test)]
mod tests {
    use game::*;
    use cli::*;

    #[test]
    fn parse_make_move_cmd() {
        assert_eq!(Command::MakeMove(Move{x: 3, y: 5, side: Side::Bottom}), parse_command("3 5 b").unwrap());
        assert_eq!(Command::MakeMove(Move{x: 3, y: 5, side: Side::Bottom}), parse_command("3 5 Bottom").unwrap());
    }

    #[test]
    fn parse_undo_move_cmd() {
        assert_eq!(Command::UndoMove(Move{x: 8, y: 6, side: Side::Left}), parse_command("u 8 6 l").unwrap());
        assert_eq!(Command::UndoMove(Move{x: 8, y: 6, side: Side::Left}), parse_command("u 8 6 Left").unwrap());
    }

    #[test]
    fn parse_nimstring_value_cmd() {
        assert_eq!(Command::CalcNimstringValue, parse_command("nv").unwrap());
    }

    #[test]
    fn parse_evaluate_cmd() {
        assert_eq!(Command::Evaluate, parse_command("eval").unwrap());
    }

    #[test]
    fn parse_help_cmd() {
        assert_eq!(Command::PrintHelp, parse_command("help").unwrap());
    }

    #[test]
    fn parse_exit_cmd() {
        assert_eq!(Command::Quit, parse_command("quit").unwrap());
        assert_eq!(Command::Quit, parse_command("exit").unwrap());
    }
}
