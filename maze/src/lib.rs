#![no_std]
#![warn(missing_docs)]

//! 迷宫生成
//!
//! 生成的结果:0表示路,1表示墙

use alloc::vec::Vec;
use core::convert::TryInto;
use core::ops::{Add, Index, IndexMut, Mul};
use core::slice::{Iter, IterMut};
use rand::{prelude::SliceRandom, Rng};

#[macro_use]
extern crate alloc;

const TILE_FLOOR: u8 = 0;
const TILE_WALL: u8 = 1;

/// Error enum for maze generation
#[derive(Debug, PartialEq, Eq)]
pub enum MazeGenerationError {
    /// Maze dimensions must be odd and >= 5
    InvalidDimensions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
}

const ALL_DIRS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct TinyVec {
    x: isize,
    y: isize,
}

impl Add for TinyVec {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul<isize> for TinyVec {
    type Output = Self;

    fn mul(self, rhs: isize) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

/// Maze structure that contains all maze data
///
/// The maze's dimensions need to be odd numbers >= 5. After creation,
/// call [generate](#method.generate) to generate the data.
/// You can access the generated maze data by indexing (e.g. ```maze[x][y]```) or by
/// using [iter](#method.iter), [into_iter](#method.into_iter)
/// or [iter_mut](#method.iter_mut).
#[derive(Debug, Clone, PartialEq)]
pub struct Maze {
    /// Width of the maze. Must be an odd number >= 5.
    pub width: usize,
    /// Height of the maze. Must be an odd number >= 5.
    pub height: usize,
    /// map data
    pub data: Vec<Vec<u8>>,
}

impl Index<usize> for Maze {
    type Output = Vec<u8>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for Maze {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl IntoIterator for Maze {
    type Item = Vec<u8>;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl core::fmt::Display for Maze {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for d in &self.data {
            writeln!(f, "{:?}", d)?;
        }
        Ok(())
    }
}

impl Maze {
    /// Construct the maze. Only odd values >= 5 can be passed.
    pub fn new(width: usize, height: usize) -> Result<Self, MazeGenerationError> {
        // if width < 5 || width % 2 == 0 || height < 5 || height % 2 == 0 {
        //     return Err(MazeGenerationError::InvalidDimensions);
        // }
        Ok(Self {
            width,
            height,
            data: vec![vec![TILE_WALL; height]; width],
        })
    }

    /// Iterate over the maze data column-wise.
    pub fn iter(&self) -> Iter<Vec<u8>> {
        self.data.iter()
    }

    /// Mutably iterate over the maze data column-wise.
    pub fn iter_mut(&mut self) -> IterMut<Vec<u8>> {
        self.data.iter_mut()
    }

    /// Generate the maze data.
    pub fn generate<R>(mut self, rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        // This basically uses a recursive backtracking algorithm. However, it was
        // modified to be iterative. Since we model walls as tiles, we need
        // to carve two cells per iteration (carve through the walls).
        let mut stack = Vec::new();

        let start = TinyVec { x: 1, y: 1 };
        stack.push((start, start));

        while let Some((path, cell)) = stack.pop() {
            if self.data[cell.x as usize][cell.y as usize] == TILE_WALL {
                // clear the path up to the cell
                self.data[path.x as usize][path.y as usize] = TILE_FLOOR;
                self.data[cell.x as usize][cell.y as usize] = TILE_FLOOR;

                // go in random direction
                let mut dirs = ALL_DIRS;
                dirs.shuffle(rng);
                for dir in dirs {
                    let step = match dir {
                        Direction::North => TinyVec { x: 0, y: -1 },
                        Direction::East => TinyVec { x: 1, y: 0 },
                        Direction::South => TinyVec { x: 0, y: 1 },
                        Direction::West => TinyVec { x: -1, y: 0 },
                    };
                    let double = cell + step * 2;

                    // check, if this path is valid
                    if double.x >= 0
                        && double.x < self.width.try_into().unwrap()
                        && double.y >= 0
                        && double.y < self.height.try_into().unwrap()
                        && self.data[double.x as usize][double.y as usize] == TILE_WALL
                    {
                        // continue carving from there
                        stack.push((cell + step, double));
                    }
                }
            }
        }
        self
    }
}
