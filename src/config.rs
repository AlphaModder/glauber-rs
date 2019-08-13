use crate::controls::SimulationType;

pub const BASE_UPDATE_INTERVAL: f32 = 0.001;
pub const SIMULATION_TYPES: [SimulationType; 1] = [SimulationType::Ising];
pub const MAX_SIZE: (u16, u16) = (65535, 65535);
pub const DEFAULT_SIZE: (u16, u16) = (100, 100);
pub const DEFAULT_TYPE_INDEX: usize = 0;
pub const DEFAULT_SIMULATION_TYPE: SimulationType = SIMULATION_TYPES[DEFAULT_TYPE_INDEX];
