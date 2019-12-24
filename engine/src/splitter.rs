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
use std::cmp;
use std::iter;

// A fragment extracted from a larger position.
pub struct PositionFragment {
    pub pos: SimplePosition,
    pub x_offset: usize,
    pub y_offset: usize,
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

// Build a PositionFragment from a list of coordinates
fn make_fragment(pos: &SimplePosition, coords: &Vec<(usize, usize)>) -> PositionFragment {
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
    PositionFragment{pos: frag_pos, x_offset: x_left, y_offset: y_top}
}

// Split a position into its independent fragments.
// If the position is fully connected, the result will consist of a single element
// representing the whole position.
pub fn split(pos: &SimplePosition) -> Vec<PositionFragment> {
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
                result.push(make_fragment(pos, &frag_coords));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use splitter::*;
    use examples::*;

    #[test]
    fn split_p50() {
        let pos = p50();
        let parts = split(&pos);
        assert_eq!(3, parts.len());

        let top_parts: Vec<&PositionFragment> = parts.iter().filter(|f| f.x_offset == 0 && f.y_offset == 0).collect();
        assert_eq!(1, top_parts.len());
        assert_eq!(true, top_parts[0].pos.eq(&p50_top()));

        let bl_parts: Vec<&PositionFragment> = parts.iter().filter(|f| f.x_offset == 0 && f.y_offset == 2).collect();
        assert_eq!(1, bl_parts.len());
        assert_eq!(true, bl_parts[0].pos.eq(&p50_bottomleft()));

        let br_parts: Vec<&PositionFragment> = parts.iter().filter(|f| f.x_offset == 3 && f.y_offset == 2).collect();
        assert_eq!(1, br_parts.len());
        assert_eq!(true, br_parts[0].pos.eq(&p50_bottomright()));
    }

    #[test]
    fn split_unsplittable() {
        let pos = SimplePosition::new_game(3, 3);
        let parts = split(&pos);
        assert_eq!(1, parts.len());
        let frag = &parts[0];
        assert_eq!(0, frag.x_offset);
        assert_eq!(0, frag.y_offset);
        assert_eq!(true, frag.pos.eq(&pos));
    }
}
