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
// Reading and writing positions in the text format used by the .pos files and the CLI:
// a first line giving the width and height of each part, followed by one move per line
// (with the part number first for positions of more than one part), and optionally
// undo moves (prefixed with "u"), comments (starting with "#") and blank lines.
use crate::game::{Move, Position, SimplePosition, CompoundPosition, Side, CPosMove};

use regex::Regex;
use std::io::{BufRead, Cursor};

pub fn parse_side(side_s: &str) -> Option<Side> {
    match side_s.to_lowercase().as_str() {
        "l" | "left" => Some(Side::Left),
        "r" | "right" => Some(Side::Right),
        "t" | "top" => Some(Side::Top),
        "b" | "bottom" => Some(Side::Bottom),
        _ => None,
    }
}

fn side_letter(side: Side) -> &'static str {
    match side {
        Side::Left => "l",
        Side::Right => "r",
        Side::Top => "t",
        Side::Bottom => "b",
    }
}

// Parse a move in a SimplePosition, in the form "x y side"
pub fn parse_simple_move(input: &str) -> Result<Move, String> {
    let move_re = Regex::new(r"^([0-9]+) ([0-9]+) ([a-zA-Z]+)$").unwrap();
    if let Some(caps) = move_re.captures(input) {
        let x = caps[1].parse::<usize>().map_err(|e| format!("Bad x coordinate: {}", e))?;
        let y = caps[2].parse::<usize>().map_err(|e| format!("Bad y coordinate: {}", e))?;
        let side_s = &caps[3];
        match parse_side(side_s) {
            Some(side) => Ok(Move{x: x, y: y, side: side}),
            None => Err(format!("Unrecognised side: [{}]", side_s))
        }
    } else {
        Err(format!("Could not extract move from [{}]", input))
    }
}

// Parse a move in a CompoundPosition, in the form "part x y side".
// If there is only one part, the form "x y side" is also accepted.
pub fn parse_compound_move(input: &str, pos: &CompoundPosition) -> Result<CPosMove, String> {
    if pos.parts().len() == 1 {
        if let Ok(m) = parse_simple_move(input) {
            return Ok(CPosMove{part: 0, m: m});
        }
    }
    let move_re = Regex::new(r"^([0-9]+) (.*)$").unwrap();
    if let Some(caps) = move_re.captures(input) {
        let p = caps[1].parse::<usize>().map_err(|e| format!("Bad part number: {}", e))?;
        let rest = &caps[2];
        if p >= pos.parts().len() {
            return Err(format!("Part {} out of bounds (count={})", p, pos.parts().len()));
        }
        if let Ok(m) = parse_simple_move(rest) {
            Ok(CPosMove{part: p, m: m})
        } else {
            Err(format!("Could not parse [{}] as SimplePosition move", rest))
        }
    } else {
        Err(format!("Could not extract move from [{}]", input))
    }
}

// Read a position. Unlike the interactive CLI, illegal moves are errors.
pub fn parse_reader<R: BufRead>(reader: R) -> Result<CompoundPosition, String> {
    let mut lines = reader.lines();
    let size_spec = lines.next().map_or(
        Err("No lines found".to_string()),
        |l| l.map_err(|e| format!("Could not read first line: {}", e)))?;
    let mut size_spec_parts: Vec<usize> = Vec::with_capacity(2);
    for part in size_spec.split_whitespace() {
        let dim = part.parse::<usize>().map_err(
            |e| format!("Could not parse int from [{}]: {}", part, e))?;
        if dim == 0 {
            return Err("Dimensions must be positive".to_string());
        }
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
    let undo_re = Regex::new(r"^[uU] (.*)$").unwrap();
    for (i, line) in lines.enumerate() {
        let line_no = i + 2;
        let line = line.map_err(|e| format!("Could not read line {}: {}", line_no, e))?;
        let line = line.trim();
        if line.is_empty() || line.starts_with("#") {
            continue;
        }
        if let Some(caps) = undo_re.captures(line) {
            let m = parse_compound_move(&caps[1], &pos).map_err(|e| format!("Line {}: {}", line_no, e))?;
            let m = pos.normalise_move(m).ok_or(format!("Line {}: move off the board: {}", line_no, m))?;
            if pos.is_legal_move(m) {
                return Err(format!("Line {}: cannot undo move which has not been made: {}", line_no, m));
            }
            pos.undo_move(m);
        } else {
            let m = parse_compound_move(line, &pos).map_err(|e| format!("Line {}: {}", line_no, e))?;
            if !pos.is_legal_move(m) {
                return Err(format!("Line {}: not a legal move: {}", line_no, m));
            }
            pos.make_move(m);
        }
    }
    Ok(pos)
}

pub fn parse_str(input: &str) -> Result<CompoundPosition, String> {
    parse_reader(Cursor::new(input))
}

// Write a position in a form which parse_reader can read back
pub fn write(pos: &CompoundPosition) -> String {
    let parts = pos.parts();
    let dims: Vec<String> = parts.iter().map(|p| format!("{} {}", p.width(), p.height())).collect();
    let mut result = dims.join(" ");
    result.push('\n');
    for (i, part) in parts.iter().enumerate() {
        for m in SimplePosition::all_moves(part.width(), part.height()) {
            if part.is_legal_move(m) {
                continue;
            }
            if parts.len() > 1 {
                result.push_str(&format!("{} ", i));
            }
            result.push_str(&format!("{} {} {}\n", m.x, m.y, side_letter(m.side)));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::examples::*;
    use crate::game::*;
    use crate::posfile::*;

    #[test]
    fn parse_moves() {
        assert_eq!(Move::new(3, 5, Side::Bottom), parse_simple_move("3 5 b").unwrap());
        assert_eq!(Move::new(3, 5, Side::Bottom), parse_simple_move("3 5 Bottom").unwrap());
        assert!(parse_simple_move("3 5 x").is_err());

        let pos = CompoundPosition::from_single(SimplePosition::new_game(6, 6));
        assert_eq!(CPosMove::new(0, 3, 5, Side::Top), parse_compound_move("3 5 t", &pos).unwrap());

        let pos = CompoundPosition::new(vec!(make_chain(5), make_chain(5)));
        assert_eq!(CPosMove::new(1, 0, 1, Side::Left), parse_compound_move("1 0 1 l", &pos).unwrap());
        let parsed = parse_compound_move("2 0 0 l", &pos);
        assert!(parsed.err().unwrap().contains("Part 2 out of bounds"));
    }

    #[test]
    fn parse_simple_position() {
        let input_str = vec!(
            "3 1", "0 0 t", "0 0 b", "1 0 t", "1 0 b", "2 0 t", "2 0 b"
        ).join("\n");
        let expected = CompoundPosition::from_single(make_chain(3));
        let actual = parse_str(&input_str).unwrap();
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
        let actual = parse_str(&input_str).unwrap();
        assert_eq!(true, expected.eq(&actual), "{}", actual);
    }

    #[test]
    fn parse_with_undo() {
        let pos = parse_str("2 1\n0 0 r\n0 0 t\nu 1 0 l\n").unwrap();
        let mut expected = SimplePosition::new_game(2, 1);
        expected.make_move(Move::new(0, 0, Side::Top));
        assert!(CompoundPosition::from_single(expected).eq(&pos));
    }

    #[test]
    fn parse_position_errors() {
        let err = |s: &str| parse_str(s).err().unwrap();
        assert_eq!("No lines found", err(""));
        assert!(err("1").starts_with("Expected an even number of dimensions and at least 2"));
        assert!(err("1 2 3").starts_with("Expected an even number of dimensions and at least 2"));
        assert!(err("zxcv").contains("Could not parse int"));
        assert!(err("0 2").contains("positive"));
        assert_eq!("Line 3: not a legal move: Part 0: (0, 0) Top", err("2 2\n0 0 t\n0 0 t"));
        assert!(err("2 2\nu 0 0 t").contains("cannot undo"));
        assert!(err("2 2\nnv").starts_with("Line 2:"));
    }

    #[test]
    fn write_round_trip() {
        let positions = vec!(
            CompoundPosition::from_single(SimplePosition::new_game(3, 3)),
            CompoundPosition::from_single(p50()),
            CompoundPosition::from_single(ex3p12()),
            one_long_multi_three(3, 4),
        );
        for pos in positions {
            let written = write(&pos);
            let parsed = parse_str(&written).unwrap();
            assert!(pos.eq(&parsed), "{}", written);
            assert_eq!(pos.zhash(), parsed.zhash(), "{}", written);
        }
    }

    #[test]
    fn write_format() {
        assert_eq!("3 1\n0 0 t\n1 0 t\n2 0 t\n0 0 b\n1 0 b\n2 0 b\n",
                   write(&CompoundPosition::from_single(make_chain(3))));
        let pos = CompoundPosition::new(vec!(make_chain(1), SimplePosition::new_game(1, 1)));
        assert_eq!("1 1 1 1\n0 0 0 t\n0 0 0 b\n", write(&pos));
    }

    #[test]
    fn pos_files_match_examples() {
        let check = |text: &str, expected: CompoundPosition| {
            let parsed = parse_str(text).unwrap();
            assert!(expected.eq(&parsed), "{}", parsed);
        };
        check(include_str!("../ex5p1.pos"), CompoundPosition::from_single(ex3p1()));
        check(include_str!("../p50.pos"), CompoundPosition::from_single(p50()));
        check(include_str!("../p50bl.pos"), CompoundPosition::from_single(p50_bottomleft()));
        check(include_str!("../p3_4.pos"),
              CompoundPosition::new(vec!(make_chain(3), make_chain(3), make_chain(3), make_chain(4))));
    }
}
