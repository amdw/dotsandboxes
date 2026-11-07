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
use crate::game::{Move, CPosMove, Position, CompoundPosition, SimplePosition, Side};
use std::cmp;
use std::iter;

pub trait SplittablePosition<M> : Position<M> {
    // Split a Position into its independent fragments.
    // If the position is fully connected, the result will consist of a single element
    // representing the whole position.
    fn split(&self) -> Vec<SimplePosition>;
}

// An independent fragment of a SimplePosition, together with the coordinates
// (in the original position) of the coins it consists of
pub struct Fragment {
    pub coins: Vec<(usize, usize)>,
    pub position: SimplePosition,
}

// Split a SimplePosition into its independent fragments, keeping track of
// which coins of the original position each fragment consists of.
// Coins which have already been captured are not part of any fragment.
pub fn fragments(pos: &SimplePosition) -> Vec<Fragment> {
    let mut visited: Vec<Vec<bool>> = Vec::with_capacity(pos.width());
    for _ in 0..pos.width() {
        visited.push(iter::repeat(false).take(pos.height()).collect());
    }
    let mut result = Vec::new();
    for x in 0..pos.width() {
        for y in 0..pos.height() {
            if !visited[x][y] && pos.valency(x, y) > 0 {
                let mut frag_coords = Vec::new();
                search(pos, x, y, &mut visited, &mut frag_coords);
                let position = make_fragment(pos, &frag_coords);
                frag_coords.sort();
                result.push(Fragment{ coins: frag_coords, position: position });
            }
        }
    }
    result
}

impl SplittablePosition<Move> for SimplePosition {
    fn split(self: &SimplePosition) -> Vec<SimplePosition> {
        fragments(self).into_iter().map(|f| f.position).collect()
    }
}

impl SplittablePosition<CPosMove> for CompoundPosition {
    fn split(self: &CompoundPosition) -> Vec<SimplePosition> {
        let mut new_parts = Vec::with_capacity(self.parts.len());
        for part in &self.parts {
            let mut split = part.split();
            new_parts.append(&mut split);
        }
        new_parts
    }
}

// Depth-first search to find all coordinates which are part of the fragment including (x, y)
fn search(pos: &SimplePosition, x: usize, y: usize,
          visited: &mut Vec<Vec<bool>>, frag_coords: &mut Vec<(usize, usize)>) {
    visited[x][y] = true;
    frag_coords.push((x, y));
    for side in Side::all() {
        if pos.is_legal_move(Move{x: x, y: y, side: side}) {
            if let Some((next_x, next_y)) = pos.offset(x, y, side) {
                if !visited[next_x][next_y] {
                    search(pos, next_x, next_y, visited, frag_coords);
                }
            }
        }
    }
}

// Build the sub-position from a list of coordinates
fn make_fragment(pos: &SimplePosition, coords: &Vec<(usize, usize)>) -> SimplePosition {
    let (x_left, x_right, y_top, y_bottom) = coords.iter().fold(
        (coords[0].0, coords[0].0, coords[0].1, coords[0].1),
        |(xl, xr, yt, yb), &(x, y)| (cmp::min(xl, x), cmp::max(xr, x), cmp::min(yt, y), cmp::max(yb, y))
    );
    let mut frag_pos = SimplePosition::new_end_game(x_right - x_left + 1, y_bottom - y_top + 1);
    for &(x, y) in coords {
        let (frag_x, frag_y) = (x - x_left, y - y_top);
        for side in Side::all() {
            let frag_move = Move{x: frag_x, y: frag_y, side: side};
            let legal_in_pos = pos.is_legal_move(Move{x: x, y: y, side: side});
            let legal_in_frag = frag_pos.is_legal_move(frag_move);
            if legal_in_pos && !legal_in_frag {
                frag_pos.undo_move(frag_move);
            }
        }
    }
    frag_pos
}

#[cfg(test)]
mod tests {
    use crate::splitter::*;
    use crate::examples::*;

    #[test]
    fn split_p50() {
        let pos = p50();
        let parts = pos.split();
        assert_eq!(3, parts.len());

        let top_parts: Vec<&SimplePosition> = parts.iter().filter(|&p| p.eq(&p50_top())).collect();
        assert_eq!(1, top_parts.len());

        let bl_parts: Vec<&SimplePosition> = parts.iter().filter(|&p| p.eq(&p50_bottomleft())).collect();
        assert_eq!(1, bl_parts.len());

        let br_parts: Vec<&SimplePosition> = parts.iter().filter(|&p| p.eq(&p50_bottomright())).collect();
        assert_eq!(1, br_parts.len());
    }

    #[test]
    fn split_unsplittable() {
        let pos = SimplePosition::new_game(3, 3);
        let parts = pos.split();
        assert_eq!(1, parts.len());
        let part = &parts[0];
        assert_eq!(true, part.eq(&pos));
    }

    #[test]
    fn fragment_coins() {
        let frags = fragments(&p50());
        assert_eq!(3, frags.len());
        let coin_count: usize = frags.iter().map(|f| f.coins.len()).sum();
        assert_eq!(20, coin_count);
        // Bottom-right corner of p50 is a 2x2 fragment
        let br: Vec<&Fragment> = frags.iter().filter(|f| f.position.eq(&p50_bottomright())).collect();
        assert_eq!(1, br.len());
        assert_eq!(vec![(3, 2), (3, 3), (4, 2), (4, 3)], br[0].coins);
    }

    #[test]
    fn fragments_skip_captured() {
        let mut pos = make_chain(3);
        pos.make_move(Move::new(0, 0, Side::Left));
        pos.make_move(Move::new(0, 0, Side::Right));
        let frags = fragments(&pos);
        assert_eq!(1, frags.len());
        assert_eq!(vec![(1, 0), (2, 0)], frags[0].coins);
    }

    #[test]
    fn split_compound() {
        let pos = CompoundPosition::new(vec!(
            multi_chains(5, 2),
            multi_chains(5, 2)
        ));
        let split = pos.split();
        assert_eq!(4, split.len());
        let chains: Vec<&SimplePosition> = split.iter().filter(|&p| p.eq(&make_chain(5))).collect();
        assert_eq!(4, chains.len());
    }
}
