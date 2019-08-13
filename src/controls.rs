use imgui::{Ui, im_str, ImStr};
use rand::{SeedableRng, rngs::StdRng};

use crate::simulation::{Simulation, ErasedSimulation};
use crate::cells::{CellGrid, GlauberCell};
use crate::config;

const AUTO_SIZE: [f32; 2] = [0.0, 0.0];

pub struct Controls {
    pub paused: bool,
    pub update_interval: f32,
    current_sim_type: i32,
    next_sim_type: i32,
    next_sim_size: (u16, u16),
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            paused: true,
            update_interval: config::BASE_UPDATE_INTERVAL,
            current_sim_type: config::DEFAULT_TYPE_INDEX as i32,
            next_sim_type: config::DEFAULT_TYPE_INDEX as i32,
            next_sim_size: config::DEFAULT_SIZE,
        }
    }

    pub fn draw(&mut self, ui: &Ui, simulation: &mut Box<dyn ErasedSimulation + Send>, _iters: usize) {
        ui.window(im_str!("Controls"))
            .build(|| {
                let names = config::SIMULATION_TYPES.iter().map(SimulationType::name).collect::<Vec<_>>();
                ui.combo(im_str!("Simulation Type"), &mut self.current_sim_type, &names, -1);
                let mut size = [self.next_sim_size.0 as i32, self.next_sim_size.1 as i32];
                if ui.input_int2(im_str!("Size"), &mut size).build() {
                    let xrange = 0..config::MAX_SIZE.0 as i32 + 1;
                    let yrange = 0..config::MAX_SIZE.1 as i32 + 1;
                    if xrange.contains(&size[0]) && yrange.contains(&size[1]) {
                        self.next_sim_size = (size[0] as u16, size[1] as u16)
                    }
                }
                if ui.button(im_str!("New simulation"), AUTO_SIZE) {
                    *simulation = config::SIMULATION_TYPES[self.next_sim_type as usize].create(self.next_sim_size)
                }
                ui.separator();
                let toggle_label = if self.paused { "Play" } else { "Pause" };
                if ui.button(&im_str!("{}###TogglePlay", toggle_label), AUTO_SIZE) {
                    self.paused = !self.paused;
                }
                ui.same_line(0.0);
                if ui.button(im_str!("Step"), AUTO_SIZE) {
                    simulation.step()
                }
                ui.same_line(0.0);
                if ui.button(im_str!("Reset"), AUTO_SIZE) {
                    let size = simulation.size();
                    *simulation = config::SIMULATION_TYPES[self.current_sim_type as usize].create(size)
                }
                ui.slider_float(im_str!("Temperature"), simulation.temperature_mut(), 0.01, 5.0).build();

                // ui.text(format!("{}", ui.io().framerate));
                // ui.text(format!("{}", iters));
            });
    }
}

#[derive(Copy, Clone)]
pub enum SimulationType {
    Ising,
}

impl SimulationType {
    pub fn create(&self, size: (u16, u16)) -> Box<dyn ErasedSimulation + Send> {
        let mut rng = StdRng::from_rng(rand::thread_rng()).unwrap();
        match self {
            SimulationType::Ising => Box::new(Simulation::<GlauberCell, _> {
                cells: CellGrid::new_random(size, &mut rng),
                temperature: 1.0,
                rng: rng,
            })
        }
    }

    pub fn name(&self) -> &ImStr {
        match self {
            SimulationType::Ising => im_str!("Ising"),
        }
    }
}