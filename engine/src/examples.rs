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
use crate::game::{Move, Position, SimplePosition, Side, CompoundPosition, CPosMove};
use crate::posfile;
use std::iter;

// Top of example from page 50 of Berlekamp's book
pub fn p50_top() -> SimplePosition {
    let mut pos = SimplePosition::new_game(5, 2);
    pos.make_move(Move::new(0, 0, Side::Top));
    pos.make_move(Move::new(1, 0, Side::Top));
    pos.make_move(Move::new(0, 0, Side::Left));
    pos.make_move(Move::new(0, 1, Side::Left));
    pos.make_move(Move::new(1, 0, Side::Bottom));
    pos.make_move(Move::new(2, 0, Side::Bottom));
    pos.make_move(Move::new(3, 0, Side::Bottom));
    pos.make_move(Move::new(3, 0, Side::Right));
    for i in 0..5 {
        pos.make_move(Move::new(i, 1, Side::Bottom));
    }
    pos
}

// Bottom-left of example from page 50 of Berlekamp's book
pub fn p50_bottomleft() -> SimplePosition {
    let mut pos = SimplePosition::new_game(3, 2);
    pos.make_move(Move::new(0, 0, Side::Top));
    pos.make_move(Move::new(1, 0, Side::Top));
    pos.make_move(Move::new(2, 0, Side::Top));
    pos.make_move(Move::new(2, 0, Side::Right));
    pos.make_move(Move::new(2, 1, Side::Right));
    pos.make_move(Move::new(1, 0, Side::Bottom));
    pos
}

// Bottom-right of example from page 50 of Berlekamp's book
pub fn p50_bottomright() -> SimplePosition {
    let mut pos = SimplePosition::new_game(2, 2);
    pos.make_move(Move::new(0, 0, Side::Top));
    pos.make_move(Move::new(1, 0, Side::Top));
    pos.make_move(Move::new(0, 0, Side::Left));
    pos.make_move(Move::new(0, 1, Side::Left));
    pos
}

// Example from page 50 of Berlekamp's book
pub fn p50() -> SimplePosition {
    let mut pos = SimplePosition::new_game(5, 4);
    pos.make_move(Move::new(0, 0, Side::Top));
    pos.make_move(Move::new(0, 0, Side::Left));
    pos.make_move(Move::new(1, 0, Side::Top));
    pos.make_move(Move::new(1, 0, Side::Bottom));
    pos.make_move(Move::new(2, 0, Side::Bottom));
    pos.make_move(Move::new(3, 0, Side::Bottom));
    pos.make_move(Move::new(3, 0, Side::Right));
    pos.make_move(Move::new(0, 1, Side::Left));
    pos.make_move(Move::new(0, 1, Side::Bottom));
    pos.make_move(Move::new(1, 1, Side::Bottom));
    pos.make_move(Move::new(2, 1, Side::Bottom));
    pos.make_move(Move::new(3, 1, Side::Bottom));
    pos.make_move(Move::new(4, 1, Side::Bottom));
    pos.make_move(Move::new(1, 2, Side::Bottom));
    pos.make_move(Move::new(2, 2, Side::Right));
    pos.make_move(Move::new(2, 3, Side::Right));
    pos
}

pub fn icelandic_game(width: usize, height: usize) -> SimplePosition {
    let mut pos = SimplePosition::new_game(width, height);
    for y in 0..height {
        pos.make_move(Move::new(0, y, Side::Left));
    }
    for x in 0..width {
        pos.make_move(Move::new(x, 0, Side::Top));
    }
    pos
}

// Construct a position consisting of a given number of equally-sized chains.
pub fn multi_chains(chain_size: usize, chain_count: usize) -> SimplePosition {
    let mut pos = SimplePosition::new_game(chain_size, chain_count);
    for x in 0..chain_size {
        pos.make_move(Move::new(x, 0, Side::Top));
        for y in 0..chain_count {
            pos.make_move(Move::new(x, y, Side::Bottom));
        }
    }
    pos
}
// Create a position consisting of a single horizontal chain of a given length.
pub fn make_chain(length: usize) -> SimplePosition {
    multi_chains(length, 1)
}

// Create a position consisting of two horizontal chains of a given length.
pub fn double_chain(length: usize) -> SimplePosition {
    multi_chains(length, 2)
}

// Create a position consisting of two loops of a given width.
pub fn double_loop(width: usize) -> SimplePosition {
    let mut pos = SimplePosition::new_game(width, 4);
    for i in 0..width {
        pos.make_move(Move::new(i, 0, Side::Top));
        pos.make_move(Move::new(i, 1, Side::Bottom));
        pos.make_move(Move::new(i, 3, Side::Bottom));
    }
    for i in 0..4 {
        pos.make_move(Move::new(0, i, Side::Left));
        pos.make_move(Move::new(width-1, i, Side::Right));
    }
    for i in 1..width-1 {
        pos.make_move(Move::new(i, 0, Side::Bottom));
        pos.make_move(Move::new(i, 2, Side::Bottom));
    }
    pos
}

// Create Exercise 3.1 from Berlekamp's book
pub fn ex3p1() -> SimplePosition {
    let mut pos = SimplePosition::new_game(3, 3);
    pos.make_move(Move::new(0, 0, Side::Top));
    pos.make_move(Move::new(0, 0, Side::Left));
    pos.make_move(Move::new(1, 0, Side::Top));
    pos.make_move(Move::new(1, 0, Side::Bottom));
    pos.make_move(Move::new(2, 0, Side::Bottom));
    pos.make_move(Move::new(0, 1, Side::Left));
    pos.make_move(Move::new(0, 1, Side::Right));
    pos.make_move(Move::new(0, 2, Side::Right));
    pos.make_move(Move::new(1, 2, Side::Bottom));
    pos.make_move(Move::new(2, 2, Side::Right));
    pos
}

// Create Exercise 3.12 from Berlekamp's book
pub fn ex3p12() -> SimplePosition {
    let mut pos = SimplePosition::new_game(5, 5);
    for i in 0..5 {
        pos.make_move(Move::new(i, 0, Side::Top));
    }
    pos.make_move(Move::new(0, 0, Side::Bottom));
    pos.make_move(Move::new(2, 0, Side::Right));
    pos.make_move(Move::new(0, 1, Side::Left));
    pos.make_move(Move::new(1, 1, Side::Bottom));
    pos.make_move(Move::new(1, 1, Side::Right));
    pos.make_move(Move::new(2, 1, Side::Right));
    pos.make_move(Move::new(3, 1, Side::Bottom));
    pos.make_move(Move::new(4, 1, Side::Bottom));
    pos.make_move(Move::new(0, 2, Side::Right));
    pos.make_move(Move::new(0, 2, Side::Bottom));
    pos.make_move(Move::new(2, 2, Side::Right));
    pos.make_move(Move::new(2, 2, Side::Bottom));
    pos.make_move(Move::new(4, 2, Side::Right));
    pos.make_move(Move::new(0, 3, Side::Right));
    pos.make_move(Move::new(2, 3, Side::Bottom));
    pos.make_move(Move::new(3, 3, Side::Bottom));
    pos.make_move(Move::new(3, 3, Side::Right));
    pos.make_move(Move::new(4, 3, Side::Right));
    pos.make_move(Move::new(0, 4, Side::Right));
    pos.make_move(Move::new(1, 4, Side::Right));
    pos.make_move(Move::new(3, 4, Side::Bottom));
    pos.make_move(Move::new(4, 4, Side::Bottom));
    pos.make_move(Move::new(4, 4, Side::Right));
    pos
}

// Create Exercise 6.2 from Berlekamp's book
pub fn ex6p2() -> SimplePosition {
    let mut pos = SimplePosition::new_game(3, 2);
    for x in 0..3 {
        pos.make_move(Move::new(x, 0, Side::Top));
    }
    for y in 0..2 {
        pos.make_move(Move::new(2, y, Side::Right));
    }
    pos.make_move(Move::new(1, 0, Side::Bottom));
    pos
}

// Create Exercise 7.2 from Berlekamp's book
pub fn ex7p2() -> SimplePosition {
    let mut pos = SimplePosition::new_game(5, 5);
    pos.make_move(Move::new(0, 0, Side::Right));
    pos.make_move(Move::new(1, 0, Side::Right));
    pos.make_move(Move::new(3, 0, Side::Right));
    pos.make_move(Move::new(0, 1, Side::Left));
    pos.make_move(Move::new(0, 1, Side::Right));
    pos.make_move(Move::new(3, 1, Side::Right));
    pos.make_move(Move::new(1, 1, Side::Bottom));
    pos.make_move(Move::new(2, 1, Side::Bottom));
    pos.make_move(Move::new(3, 1, Side::Bottom));
    pos.make_move(Move::new(0, 2, Side::Bottom));
    pos.make_move(Move::new(0, 2, Side::Right));
    pos.make_move(Move::new(2, 2, Side::Bottom));
    pos.make_move(Move::new(3, 2, Side::Right));
    pos.make_move(Move::new(4, 2, Side::Right));
    pos.make_move(Move::new(1, 3, Side::Right));
    pos.make_move(Move::new(3, 3, Side::Right));
    pos.make_move(Move::new(3, 3, Side::Bottom));
    pos.make_move(Move::new(1, 4, Side::Right));
    pos.make_move(Move::new(2, 4, Side::Right));
    pos
}

// Construct the one-long-chain multiple-three-chain positions from the paper.
// The long chain is the 0th part, the 3-chains parts 1-n.
pub fn one_long_multi_three(three_chain_count: usize, long_chain_size: usize) -> CompoundPosition {
    let mut parts: Vec<SimplePosition> = Vec::with_capacity(three_chain_count + 1);
    parts.push(make_chain(long_chain_size));
    parts.extend(iter::repeat(make_chain(3)).take(three_chain_count));
    CompoundPosition::new(parts)
}

// Build a single-coin-row position with the given strings present, for the
// small Nimstring examples: `ground_left`/`ground_right` are the ground strings at
// either end, and the coins are always linked to each other.
fn coin_row(length: usize, ground_left: bool, ground_right: bool) -> SimplePosition {
    let mut pos = make_chain(length);
    if !ground_left {
        pos.make_move(Move::new(0, 0, Side::Left));
    }
    if !ground_right {
        pos.make_move(Move::new(length - 1, 0, Side::Right));
    }
    pos
}

// The small positions illustrating Nimstring values in the paper, as separate parts
pub fn nimstring_examples() -> CompoundPosition {
    CompoundPosition::new(vec!(
        coin_row(1, true, false),
        coin_row(2, false, false),
        coin_row(2, true, false),
        coin_row(3, false, false),
        make_chain(2),
        make_chain(3),
    ))
}

// A named example position, optionally with a sequence of moves to be played from it
// (e.g. a sample game from the paper)
pub struct Example {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub position: CompoundPosition,
    pub moves: Vec<CPosMove>,
}

impl Example {
    fn new(id: &'static str, title: &'static str, description: &'static str, position: CompoundPosition) -> Example {
        Example{ id: id, title: title, description: description, position: position, moves: Vec::new() }
    }

    fn simple(id: &'static str, title: &'static str, description: &'static str, position: SimplePosition) -> Example {
        Example::new(id, title, description, CompoundPosition::from_single(position))
    }

    // An example consisting of a game played from the start on a board of the given size,
    // with moves given as a comma-separated list in the .pos format, e.g. "0 0 r, 1 0 b"
    fn game(id: &'static str, title: &'static str, description: &'static str,
            width: usize, height: usize, moves: &str) -> Example {
        let position = CompoundPosition::from_single(SimplePosition::new_game(width, height));
        let moves = moves.split(',').map(|m| {
            let m = posfile::parse_simple_move(m.trim()).unwrap();
            position.normalise_move(CPosMove{ part: 0, m: m }).unwrap()
        }).collect();
        Example{ id: id, title: title, description: description, position: position, moves: moves }
    }
}

// All the named example positions, from Berlekamp's book "The Dots-and-Boxes Game"
// and from the paper in this repository
pub fn catalogue() -> Vec<Example> {
    vec!(
        Example::game("sample-game-2x2", "Sample 2×2 game",
            "The sample game from the introduction of the paper, in both its dots-and-boxes and strings-and-coins figures",
            2, 2, "0 0 r, 1 0 r, 0 0 b, 0 1 r, 1 1 b, 0 0 l, 0 0 t, 0 1 l, 0 1 b, 1 1 r, 1 1 t, 1 0 t"),
        Example::new("p45", "P(4,5)",
            "Four 3-chains and a 5-chain, from the paper's analysis of one long chain with multiple 3-chains",
            one_long_multi_three(4, 5)),
        Example::new("p34", "P(3,4)",
            "Three 3-chains and a 4-chain (also in p3_4.pos)",
            one_long_multi_three(3, 4)),
        Example::simple("double-deal", "Double-dealing on a chain",
            "Two 5-chains: the player to move must open one, and the opponent double-deals to win 8-2 (paper figure)",
            double_chain(5)),
        Example::simple("loop-double-deal", "Double-dealing on a loop",
            "Two 10-loops: the player to move must open one, and the opponent double-deals to win 16-4 (paper figure)",
            double_loop(5)),
        Example::simple("drawn-loony", "Drawn loony endgame",
            "Two 4-loops, a drawn endgame (paper figure)",
            double_loop(2)),
        Example::new("nimstring-examples", "Nimstring value examples",
            "The small positions used to illustrate Nimstring values in the paper, each as a separate part",
            nimstring_examples()),
        Example::simple("3x3-strategy", "Berlekamp's 3×3 strategy",
            "The opening position of Berlekamp's winning 3×3 strategy (paper figure)",
            {
                let mut pos = SimplePosition::new_game(3, 3);
                for m in [Move::new(0, 0, Side::Bottom), Move::new(0, 2, Side::Right),
                          Move::new(2, 2, Side::Top), Move::new(2, 0, Side::Left)] {
                    pos.make_move(m);
                }
                pos
            }),
        Example::game("3x3-sample-game", "Sample 3×3 game",
            "A sample game played according to Berlekamp's winning 3×3 strategy (paper figure)",
            3, 3, "1 2 b, 1 2 l, 1 1 t, 0 1 t, 2 1 t, 1 1 r, 1 0 t, 0 1 l, 0 0 t, 0 0 r, \
                   0 0 l, 1 0 r, 0 2 l, 2 0 t, 2 0 r, 2 2 b, 2 2 r, 0 2 b, 0 2 t, 0 1 r"),
        Example::simple("berlekamp-p50", "Berlekamp p. 50",
            "Figure 25 from page 50 of Berlekamp's book, used in the paper to motivate Nimstring (also in p50.pos)",
            p50()),
        Example::simple("berlekamp-p50-top", "Berlekamp p. 50 (top)",
            "The top component of the page 50 example", p50_top()),
        Example::simple("berlekamp-p50-bottom-left", "Berlekamp p. 50 (bottom left)",
            "The bottom-left component of the page 50 example (also in p50bl.pos)", p50_bottomleft()),
        Example::simple("berlekamp-p50-bottom-right", "Berlekamp p. 50 (bottom right)",
            "The bottom-right component of the page 50 example", p50_bottomright()),
        Example::simple("berlekamp-ex3.1", "Berlekamp exercise 3.1",
            "Exercise 3.1 from Berlekamp's book (paper figure; also in ex5p1.pos)", ex3p1()),
        Example::simple("berlekamp-ex3.12", "Berlekamp exercise 3.12",
            "Exercise 3.12 from Berlekamp's book (paper figure)", ex3p12()),
        Example::simple("berlekamp-ex6.2", "Berlekamp exercise 6.2",
            "Exercise 6.2 from Berlekamp's book", ex6p2()),
        Example::simple("berlekamp-ex7.2", "Berlekamp exercise 7.2",
            "Exercise 7.2 from Berlekamp's book", ex7p2()),
        Example::simple("icelandic-2x2", "Icelandic 2×2",
            "A 2×2 board with the top and left edges already drawn", icelandic_game(2, 2)),
        Example::simple("icelandic-3x3", "Icelandic 3×3",
            "A 3×3 board with the top and left edges already drawn", icelandic_game(3, 3)),
    )
}

#[cfg(test)]
mod tests {
    use crate::examples::*;
    use crate::game::*;

    #[test]
    fn test_olmt_indep() {
        let mut pos = one_long_multi_three(2, 5);
        assert_eq!(14, pos.legal_moves().len());
        // Just to assert that the repeated components are really cloned/independent
        pos.make_move(CPosMove::new(1, 0, 0, Side::Right));
        assert_eq!(13, pos.legal_moves().len());
    }

    #[test]
    fn catalogue_valid() {
        let examples = catalogue();
        let mut ids: Vec<&str> = examples.iter().map(|e| e.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(examples.len(), ids.len(), "Example IDs must be unique");
        for example in examples {
            // Every move in a game record must be legal when it is played
            let mut pos = example.position.clone();
            for &m in &example.moves {
                assert!(pos.is_legal_move(m), "{}: {}", example.id, m);
                pos.make_move(m);
            }
        }
    }

    #[test]
    fn sample_games_finish() {
        for id in ["sample-game-2x2"] {
            let example = catalogue().into_iter().find(|e| e.id == id).unwrap();
            let mut pos = example.position.clone();
            for &m in &example.moves {
                pos.make_move(m);
            }
            assert!(pos.is_end_of_game(), "{}", id);
        }
    }

    #[test]
    fn paper_figure_values() {
        use crate::eval::eval;
        use crate::nimstring::{component_values, Value};
        let find = |id: &str| catalogue().into_iter().find(|e| e.id == id).unwrap().position;
        assert_eq!(-6, eval(&find("double-deal")).0);
        assert_eq!(-12, eval(&find("loop-double-deal")).0);
        assert_eq!(0, eval(&find("drawn-loony")).0);

        // Values as given in the Nimstring examples figure
        let values: Vec<Value> = component_values(&nimstring_examples()).iter().map(|c| c.value).collect();
        assert_eq!(vec!(Value::Nimber(0), Value::Nimber(0), Value::Loony, Value::Nimber(0),
                        Value::Nimber(1), Value::Nimber(0)), values);
    }
}
