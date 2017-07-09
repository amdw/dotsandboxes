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
use std::io;
use regex::{Regex, Captures};

#[derive(PartialEq)]
#[derive(Debug)]
enum Command {
    MakeMove(Move),
    UndoMove(Move),
    CalcNimstringValue,
    PrintHelp,
}

impl Command {
    fn execute(self: &Command, pos: &mut Position) {
        match self {
            &Command::MakeMove(m) => { pos.make_move(m.x, m.y, m.side); },
            &Command::UndoMove(m) => { pos.undo_move(m.x, m.y, m.side); },
            &Command::CalcNimstringValue => {
                let (val, per_move) = nimstring::calc_value_with_moves(pos);
                println!("Position value is {}", val);
                for (m, v) in &per_move {
                    println!("{} {}", m, v);
                }
            },
            &Command::PrintHelp => { print_help(); },
        }
    }
}

fn print_help() {
    println!("Available commands:");
    println!("x y t/l/b/r - make move (x,y) top/left/bottom/right");
    println!("u x y t/l/b/r - undo move (x,y) top/left/bottom/right");
    println!("nv - calculate Nimstring value of current position");
    println!("help - print this help message");
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
    if "help" == input {
        return Ok(Command::PrintHelp);
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

// Enter the main loop of the CLI
pub fn main_loop(width: usize, height: usize) {
    let mut pos = Position::new_game(width, height);
    loop {
        println!("{}", pos);
        let command = get_next_command();
        command.execute(&mut pos);
        println!();
    }
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
    fn parse_help_cmd() {
        assert_eq!(Command::PrintHelp, parse_command("help").unwrap());
    }
}
