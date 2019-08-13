use lazy_static::*;
use rand::{Rng, distributions::{Distribution, Uniform}};

use crate::{support, canvas, cells::{Cell, CellGrid}};

lazy_static! {
    static ref UNIFORM_DISTR: Uniform<f32> = Uniform::new_inclusive(0.0, 1.0);
}

pub struct Simulation<C: Cell, R: Rng> {
    pub cells: CellGrid<C>,
    pub temperature: f32,
    pub rng: R,
}

impl<C: Cell, R: Rng> Simulation<C, R> {
    pub fn step(&mut self) {
        for _ in 0..self.cells.size().0 as usize * self.cells.size().1 as usize {
            let x = self.rng.gen_range(0, self.cells.size().0);
            let y = self.rng.gen_range(0, self.cells.size().1);
            
            let cell = self.cells.get_cell(x, y);
            let new_cell = cell.mutate(&mut self.rng);
            
            let neighbors = self.cells.neighbors(x, y);
            let energy_diff = new_cell.interaction_energy(&neighbors) - cell.interaction_energy(&neighbors);
            
            if energy_diff < 0.0 || (self.temperature > 0.0 && self.should_heat_flip(energy_diff)) {
                self.cells.set_cell(x, y, new_cell);
            }
        }
    }

    fn should_heat_flip(&mut self, energy_diff: f32) -> bool {
        UNIFORM_DISTR.sample(&mut self.rng) < f32::exp(-energy_diff / self.temperature)
    }
}

pub trait ErasedSimulation {
    fn temperature(&self) -> f32;
    fn temperature_mut(&mut self) -> &mut f32;
    fn size(&self) -> (u16, u16);
    fn step(&mut self);
    fn render_to(&self, encoder: &mut support::types::Encoder, texture: &mut canvas::SimTexture);
    fn set_temperature(&mut self, temp: f32) {
        *self.temperature_mut() = temp;
    }
}

impl<C: Cell, R: Rng> ErasedSimulation for Simulation<C, R> {
    fn temperature(&self) -> f32 { self.temperature }
    fn temperature_mut(&mut self) -> &mut f32 { &mut self.temperature }
    fn size(&self) -> (u16, u16) { self.cells.size() }
    fn step(&mut self) { self.step() }
    fn render_to(&self, encoder: &mut support::types::Encoder, texture: &mut canvas::SimTexture) {
        texture.update(encoder, self);
    }
}