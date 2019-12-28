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

use game::{Move, SimplePosition, CompoundPosition, Side, CPosMove};
use nimstring;
use eval::{self, EvaluablePosition};

use std::fmt::Display;
use std::hash::Hash;
use std::io::{self, BufRead};
use std::fs::File;
use regex::Regex;
use time;

#[derive(PartialEq)]
#[derive(Debug)]
enum Command<M> {
    MakeMove(M),
    UndoMove(M),
    CalcNimstringValue,
    Evaluate,
    PrintHelp,
    Quit,
}

impl <M: Copy + Display + Eq + Hash> Command<M> {
    fn execute<P>(self: &Command<M>, pos: &mut P)
    where P: CLIPosition<M> {
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
                let mut moves: Vec<&M> = per_move.keys().collect();
                pos.sort_moves(&mut moves);
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
            &Command::PrintHelp => { print_help(pos); },
            &Command::Quit => { println!("Bye bye!"); },
        }
    }
}

trait CLIPosition<M> : EvaluablePosition<M> + Display + Clone {
    fn parse_move(&self, input: &str) -> Result<M, String>;
    // Tell the user how to express a move
    fn move_cmd_help(&self, verb: &str) -> String;
    // Sort moves into the optimal order for display
    fn sort_moves(&self, moves: &mut Vec<&M>);
}

impl CLIPosition<Move> for SimplePosition {
    fn parse_move(self: &SimplePosition, input: &str) -> Result<Move, String> {
        let move_re = Regex::new(r"^(\d+) (\d+) ([a-zA-Z]+)$").unwrap();
        if let Some(caps) = move_re.captures(&input) {
            let x = caps[1].parse::<usize>().unwrap();
            let y = caps[2].parse::<usize>().unwrap();
            let side_s = caps[3].to_string();
            match parse_side(&side_s) {
                Some(side) => Ok(Move{x: x, y: y, side: side}),
                None => Err(format!("Unrecognised side: [{}]", side_s))
            }
        } else {
            Err(format!("Could not extract move from [{}]", input))
        }
    }

    fn move_cmd_help(self: &SimplePosition, verb: &str) -> String {
        format!("x y t/l/b/r - {} move (x,y) top/left/bottom/right", verb)
    }

    fn sort_moves(self: &SimplePosition, moves: &mut Vec<&Move>) {
        moves.sort_by(|a, b| a.y.cmp(&b.y).then(a.x.cmp(&b.x)).then(a.side.cmp(&b.side)));
    }
}

impl CLIPosition<CPosMove> for CompoundPosition {
    fn parse_move(self: &CompoundPosition, input: &str) -> Result<CPosMove, String> {
        // If there's only one part, let the user use the SimplePosition format for short
        if self.parts.len() == 1 {
            if let Ok(m) = self.parts[0].parse_move(&input) {
                return Ok(CPosMove{part: 0, m: m});
            }
        }
        let move_re = Regex::new(r"^(\d+) (.*)$").unwrap();
        if let Some(caps) = move_re.captures(&input) {
            let p = caps[1].parse::<usize>().unwrap();
            let rest = &caps[2];
            if p >= self.parts.len() {
                return Err(format!("Part {} out of bounds (count={})", p, self.parts.len()));
            }
            if let Ok(m) = self.parts[p].parse_move(rest) {
                Ok(CPosMove{part: p, m: m})
            } else {
                Err(format!("Could not parse [{}] as SimplePosition move", rest))
            }
        } else {
            Err(format!("Could not extract move from [{}]", input))
        }
    }

    fn move_cmd_help(self: &CompoundPosition, verb: &str) -> String {
        format!("[p] x y t/l/b/r - {} move (x,y) top/left/bottom/right in part p", verb)
    }

    fn sort_moves(self: &CompoundPosition, moves: &mut Vec<&CPosMove>) {
        moves.sort_by(|a, b| a.part.cmp(&b.part).then(
            a.m.x.cmp(&b.m.x)).then(
                a.m.y.cmp(&b.m.y)).then(
                    a.m.side.cmp(&b.m.side)));
    }
}

fn print_help<M, P>(pos: &P)
where P: CLIPosition<M> {
    println!("Available commands:");
    println!("{}", pos.move_cmd_help("make"));
    println!("u {}", pos.move_cmd_help("undo"));
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

fn parse_command<M, P: CLIPosition<M>>(input: &str, pos: &P) -> Result<Command<M>, String> {
    let input = input.to_lowercase();
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
    let undo_move_re = Regex::new(r"^[uU] (.*)$").unwrap();
    if let Some(caps) = undo_move_re.captures(&input) {
        match pos.parse_move(&caps[1]) {
            Ok(m) => return Ok(Command::UndoMove(m)),
            Err(e) => return Err(format!("Cannot extract move from [{}]: {}", input, e))
        }
    }
    match pos.parse_move(&input) {
        Ok(m) => return Ok(Command::MakeMove(m)),
        Err(e) => return Err(format!("Cannot extract move from [{}]: {}", input, e))
    }
}

fn get_next_command<M, P>(pos: &P) -> Command<M>
where P: CLIPosition<M> {
    loop {
        let mut input = String::new();
        if let Err(error) = io::stdin().read_line(&mut input) {
            println!("Error reading from standard input: {}", error);
            continue;
        }
        let input = input.trim();
        match parse_command(&input, pos) {
            Ok(command) => return command,
            Err(error) => {
                println!("Cannot execute [{}]: {}", input, error);
                println!("For help, try 'help'");
            }
        }
    }
}

fn main_loop_from<M, P>(pos: &mut P)
where M: Copy + Display + Eq + Hash, P: CLIPosition<M> {
    loop {
        println!("{}", pos);
        let command = get_next_command(pos);
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

fn parse_position<R: BufRead>(reader: R) -> Result<CompoundPosition, String> {
    let mut lines = reader.lines();
    let size_spec = lines.next().map_or(
        Err("No lines found".to_string()),
        |l| l.map_err(|e| format!("Could not read first line: {}", e)))?;
    let mut size_spec_parts: Vec<usize> = Vec::with_capacity(2);
    for part in size_spec.split(" ") {
        let dim = part.parse::<usize>().map_err(
            |e| format!("Could not parse int from [{}]: {}", part, e))?;
        size_spec_parts.push(dim);
    }
    if size_spec_parts.len() < 2 || size_spec_parts.len() % 2 != 0 {
        return Err(format!(
            "Expected an even number of dimensions and at least 2, found: {:?}", size_spec_parts));
    }
    let mut parts: Vec<SimplePosition> = Vec::with_capacity(size_spec_parts.len() / 2);
    for pair in size_spec_parts.chunks(2) {
        parts.push(SimplePosition::new_game(pair[0], pair[1]));
    }
    let mut pos = CompoundPosition::new(parts);
    for line in lines {
        let line = line.map_err(|e| format!("Could not read line: {}", e))?;
        if line.trim().len() == 0 || line.starts_with("#") {
            continue;
        }
        let command = parse_command(&line, &pos)?;
        command.execute(&mut pos);
    }
    Ok(pos)
}

// Enter the main loop of the CLI from the start of the game
pub fn main_loop_start(width: usize, height: usize) {
    let mut pos = SimplePosition::new_game(width, height);
    main_loop_from(&mut pos);
}

// Execute a given file of commands (which must have the dimensions of the position on the first line)
// and then enter the CLI main loop.
pub fn main_loop_file(filename: &str) {
    let f = File::open(filename).expect(
        format!("Could not open file [{}]", filename).as_str());
    let reader = io::BufReader::new(f);
    let mut pos = parse_position(reader).expect(
        format!("Could not read position from [{}]", filename).as_str());
    main_loop_from(&mut pos);
}

#[cfg(test)]
mod tests {
    use examples::*;
    use game::*;
    use cli::*;
    use std::io::Cursor;

    #[test]
    fn parse_make_move_cmd() {
        let pos = SimplePosition::new_game(6, 6);
        assert_eq!(Command::MakeMove(Move::new(3, 5, Side::Bottom)), parse_command("3 5 b", &pos).unwrap());
        assert_eq!(Command::MakeMove(Move::new(3, 5, Side::Bottom)), parse_command("3 5 Bottom", &pos).unwrap());

        let pos = CompoundPosition::from_single(SimplePosition::new_game(6, 6));
        assert_eq!(Command::MakeMove(CPosMove::new(0, 3, 5, Side::Top)), parse_command("3 5 t", &pos).unwrap());
        assert_eq!(Command::MakeMove(CPosMove::new(0, 3, 5, Side::Top)), parse_command("3 5 Top", &pos).unwrap());

        let pos = CompoundPosition::new(vec!(make_chain(5), make_chain(5)));
        assert_eq!(Command::MakeMove(CPosMove::new(1, 0, 1, Side::Left)), parse_command("1 0 1 l", &pos).unwrap());
        assert_eq!(Command::MakeMove(CPosMove::new(1, 0, 1, Side::Left)), parse_command("1 0 1 Left", &pos).unwrap());
    }

    #[test]
    fn parse_make_move_oob() {
        let pos = CompoundPosition::new(vec![make_chain(5), make_chain(5)]);
        let parsed = parse_command("2 0 0 l", &pos);
        assert!(parsed.is_err());
        assert!(parsed.err().unwrap().contains("Part 2 out of bounds"));
    }

    #[test]
    fn parse_undo_move_cmd() {
        let pos = SimplePosition::new_game(9, 9);
        assert_eq!(Command::UndoMove(Move::new(8, 6, Side::Left)), parse_command("u 8 6 l", &pos).unwrap());
        assert_eq!(Command::UndoMove(Move::new(8, 6, Side::Left)), parse_command("u 8 6 Left", &pos).unwrap());

        let pos = CompoundPosition::from_single(SimplePosition::new_game(9, 9));
        assert_eq!(Command::UndoMove(CPosMove::new(0, 8, 6, Side::Left)), parse_command("u 8 6 l", &pos).unwrap());
        assert_eq!(Command::UndoMove(CPosMove::new(0, 8, 6, Side::Left)), parse_command("u 8 6 Left", &pos).unwrap());

        let pos = CompoundPosition::new(vec!(make_chain(5), make_chain(5)));
        assert_eq!(Command::UndoMove(CPosMove::new(1, 3, 2, Side::Top)), parse_command("u 1 3 2 t", &pos).unwrap());
        assert_eq!(Command::UndoMove(CPosMove::new(1, 3, 2, Side::Top)), parse_command("u 1 3 2 Top", &pos).unwrap());
    }

    #[test]
    fn parse_nimstring_value_cmd() {
        let pos = SimplePosition::new_game(1, 1);
        assert_eq!(Command::CalcNimstringValue, parse_command("nv", &pos).unwrap());
    }

    #[test]
    fn parse_evaluate_cmd() {
        let pos = SimplePosition::new_game(1, 1);
        assert_eq!(Command::Evaluate, parse_command("eval", &pos).unwrap());
    }

    #[test]
    fn parse_help_cmd() {
        let pos = SimplePosition::new_game(1, 1);
        assert_eq!(Command::PrintHelp, parse_command("help", &pos).unwrap());
    }

    #[test]
    fn parse_exit_cmd() {
        let pos = SimplePosition::new_game(1, 1);
        assert_eq!(Command::Quit, parse_command("quit", &pos).unwrap());
        assert_eq!(Command::Quit, parse_command("exit", &pos).unwrap());
    }

    #[test]
    fn parse_simple_position() {
        let input_str = vec!(
            "3 1", "0 0 t", "0 0 b", "1 0 t", "1 0 b", "2 0 t", "2 0 b"
        ).join("\n");
        let expected = CompoundPosition::from_single(make_chain(3));
        let actual = parse_position(Cursor::new(input_str)).unwrap();
        assert_eq!(true, expected.eq(&actual), "{}", actual);
    }

    #[test]
    fn parse_compound_position() {
        let input_str = vec!(
            "3 1 4 1",
            "# A comment followed by a blank line", "",
            "0 0 0 t", "0 0 0 b", "0 1 0 t", "0 1 0 b", "0 2 0 t", "0 2 0 b",
            "1 0 0 t", "1 0 0 b", "1 1 0 t", "1 1 0 b", "1 2 0 t", "1 2 0 b", "1 3 0 t", "1 3 0 b"
        ).join("\n");
        let expected = CompoundPosition::new(vec!(make_chain(3), make_chain(4)));
        let actual = parse_position(Cursor::new(input_str)).unwrap();
        assert_eq!(true, expected.eq(&actual), "{}", actual);
    }

    #[test]
    fn parse_position_errors() {
        let parsed = parse_position(Cursor::new(""));
        assert!(parsed.is_err());
        assert_eq!("No lines found", parsed.err().unwrap());

        let parsed = parse_position(Cursor::new("1"));
        assert!(parsed.is_err());
        assert!(parsed.err().unwrap().starts_with(
            "Expected an even number of dimensions and at least 2"));

        let parsed = parse_position(Cursor::new("1 2 3"));
        assert!(parsed.is_err());
        assert!(parsed.err().unwrap().starts_with(
            "Expected an even number of dimensions and at least 2"));

        let parsed = parse_position(Cursor::new("zxcv"));
        assert!(parsed.is_err());
        assert!(parsed.err().unwrap().contains("Could not parse int"));
    }
}
