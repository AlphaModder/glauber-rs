use std::iter;
use rand::Rng;

use crate::config;

pub trait Cell: Sized {
    fn random(rng: &mut impl Rng) -> Self;
    fn mutate(&self, rng: &mut impl Rng) -> Self;
    fn interaction_energy(&self, neighbors: &Neighbors<Self>) -> f32;
    fn to_color(&self) -> Color;
}

pub struct Color(u8, u8, u8);

impl Color {
    pub fn to_rgba(&self) -> [u8; 4] { [self.0, self.1, self.2, 255] }
    pub fn correct_gamma(&self) -> Color {
        Color(
            ((self.0 as f32 / 255.0).powf(2.2) * 255.0) as u8,
            ((self.1 as f32 / 255.0).powf(2.2) * 255.0) as u8,
            ((self.2 as f32 / 255.0).powf(2.2) * 255.0) as u8,
        )
    }
}



pub struct CellGrid<C: Cell> {
    size: (u16, u16),
    cells: Vec<C>,
}

impl<C: Cell> CellGrid<C> {
    pub fn new_random(size: (u16, u16), random: &mut impl Rng) -> Self {
        assert!(size.0 < config::MAX_SIZE.0 && size.1 < config::MAX_SIZE.1);
        CellGrid {
            size: size,
            cells: iter::repeat_with(|| C::random(random)).take(size.0 as usize * size.1 as usize).collect(),
        }
    }

    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    pub fn neighbors(&self, x: u16, y: u16) -> Neighbors<C> {
        Neighbors { grid: self, cell: (x, y) }
    }

    pub fn get_cell(&self, x: u16, y: u16) -> &C {
        assert!(x < self.size.0 && y < self.size.1);
        self.cells.get(self.size.0 as usize * y as usize + x as usize).unwrap()
    }

    pub fn set_cell(&mut self, x: u16, y: u16, new: C) -> C {
        assert!(x < self.size.0 && y < self.size.1);
        let mut c = new;
        std::mem::swap(&mut c, self.cells.get_mut(self.size.0 as usize * y as usize + x as usize).unwrap());
        c
    }

    pub fn iter(&self) -> impl Iterator<Item=&C> {
        self.cells.iter()
    }
}

pub struct Neighbors<'a, C: Cell> {
    grid: &'a CellGrid<C>, 
    cell: (u16, u16),
}

impl<'a, C: Cell> Neighbors<'a, C> {
    pub fn get_cell(&self, x: i16, y: i16) -> &C {
        // println!("({}, {}) + ({}, {}) = ", self.cell.0, self.cell.1, x, y);
        let size = self.grid.size();
        let x = self.cell.0 + if x > 0 { x as u16 } else { (size.0 - ((-x as u16) % size.0)) };
        let y = self.cell.1 + if y > 0 { y as u16 } else { (size.1 - ((-y as u16) % size.1)) };
        // println!("({}, {})", x % size.0, y % size.1);
        self.grid.get_cell(x % size.0, y % size.1)
    }
}

#[derive(Copy, Clone)]
pub enum GlauberCell { Up, Down }

impl GlauberCell {
    fn value(&self) -> f32 {
        match self {
            GlauberCell::Up => 1.0,
            GlauberCell::Down => -1.0
        }
    }
}

impl Cell for GlauberCell {
    fn random(rng: &mut impl Rng) -> Self {
        match rng.gen_bool(0.5) {
            true => GlauberCell::Up,
            false => GlauberCell::Down,
        }
    }

    fn mutate(&self, rng: &mut impl Rng) -> Self {
        match (rng.gen_bool(0.5), self) {
            (true, GlauberCell::Up) => GlauberCell::Down,
            (true, GlauberCell::Down) => GlauberCell::Up,
            (false, val) => *val
        }
    }

    fn interaction_energy(&self, neighbors: &Neighbors<Self>) -> f32 {
        let sum = {
            neighbors.get_cell(0, 1).value() + neighbors.get_cell(0, -1).value() +
            neighbors.get_cell(1, 0).value() + neighbors.get_cell(-1, 0).value()
        };
        -sum * self.value() 
    }

    fn to_color(&self) -> Color {
        match self {
            GlauberCell::Up => Color(113, 58, 255),
            GlauberCell::Down => Color(65, 3, 68),
        }
    }
}