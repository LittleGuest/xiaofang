//! 迷宫生成器
//!
//! 生成的结果用0和1表示,0为路,1为墙

#![no_std]

use rand::Rng;

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Wall,
    Path,
}

struct Maze {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Maze {
    fn new(width: usize, height: usize) -> Self {
        let cells = vec![Cell::Wall; width * height];
        Maze {
            width,
            height,
            cells,
        }
    }

    fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn get_cell(&self, x: usize, y: usize) -> Cell {
        self.cells[self.get_index(x, y)]
    }

    fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[self.get_index(x, y)] = cell;
    }

    fn is_valid(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && x < self.width as isize && y < self.height as isize
    }

    fn generate(&mut self, start_x: usize, start_y: usize) {
        self.set_cell(start_x, start_y, Cell::Path);
        self.recursive_backtrack(start_x as isize, start_y as isize);
    }

    fn recursive_backtrack(&mut self, x: isize, y: isize) {
        let directions = [(0, -2), (-2, 0), (0, 2), (2, 0)];

        let mut rng = rand::thread_rng();
        let mut directions = directions.to_vec();
        rng.shuffle(&mut directions);

        for (dx, dy) in directions.iter() {
            let new_x = x + *dx;
            let new_y = y + *dy;

            if self.is_valid(new_x, new_y)
                && self.get_cell(new_x as usize, new_y as usize) == Cell::Wall
            {
                self.set_cell(new_x as usize, new_y as usize, Cell::Path);
                self.set_cell(
                    (x + new_x) as usize / 2,
                    (y + new_y) as usize / 2,
                    Cell::Path,
                );
                self.recursive_backtrack(new_x, new_y);
            }
        }
    }

    fn display(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = match self.get_cell(x, y) {
                    Cell::Wall => '#',
                    Cell::Path => ' ',
                };
                print!("{}", cell);
            }
            println!();
        }
    }
}

fn main() {
    let width = 21;
    let height = 21;

    let mut maze = Maze::new(width, height);
    maze.generate(1, 1);
    maze.display();
}
