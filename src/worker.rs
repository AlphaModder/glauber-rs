use std::sync::mpsc;
use std::thread;

use crate::simulation::ErasedSimulation;

pub type SimulationResult = (Simulation, usize);
type Simulation = Box<dyn ErasedSimulation + Send>;

enum UIMessage { 
    Simulate(Simulation), 
    RequestSimulation 
}

pub fn create_worker_thread() -> NotSimulating {
    let (to_sim, from_ui) = mpsc::channel();
    let (to_ui, from_sim) = mpsc::channel();
    thread::spawn(move || worker_thread(to_ui, from_ui));
    NotSimulating { to_sim, from_sim }
}

fn worker_thread(to_ui: mpsc::Sender<SimulationResult>, from_ui: mpsc::Receiver<UIMessage>) {
    loop {
        let mut simulation = match from_ui.recv() {
            Ok(UIMessage::Simulate(simulation)) => simulation,
            Err(_) => return,
            Ok(UIMessage::RequestSimulation) => unreachable!(),
        };
        let mut iterations = 0;
        loop {
            match from_ui.try_recv() {
                Ok(UIMessage::RequestSimulation) => match to_ui.send((simulation, iterations)) {
                    Ok(_) => break,
                    Err(_) => return,
                }
                Err(mpsc::TryRecvError::Empty) => { 
                    simulation.step();
                    iterations += 1;
                }
                Err(mpsc::TryRecvError::Disconnected) => return,
                Ok(UIMessage::Simulate(_)) => unreachable!(),
            }
        }
    }
}

pub struct NotSimulating {
    to_sim: mpsc::Sender<UIMessage>,
    from_sim: mpsc::Receiver<SimulationResult>,
}

pub struct Simulating {
    to_sim: mpsc::Sender<UIMessage>,
    from_sim: mpsc::Receiver<SimulationResult>,
    paused_simulation: Option<Simulation>,
}

impl NotSimulating {
    pub fn start_simulation(self, simulation: Simulation) -> Simulating {
        self.to_sim.send(UIMessage::Simulate(simulation)).expect("Simulation thread terminated unexpectedly!");
        Simulating { to_sim: self.to_sim, from_sim: self.from_sim, paused_simulation: None }
    }

    pub fn pause_simulation(self, simulation: Simulation) -> Simulating {
        Simulating { to_sim: self.to_sim, from_sim: self.from_sim, paused_simulation: Some(simulation) }
    }
}

impl Simulating {
    pub fn stop_simulation(self) -> (NotSimulating, SimulationResult) {
        let res = match self.paused_simulation {
            Some(sim) => (sim, 0),
            None => {
                self.to_sim.send(UIMessage::RequestSimulation).expect("Simulation thread terminated unexpectedly!");
                self.from_sim.recv().expect("Simulation thread terminated unexpectedly!")
            }
        };
        (NotSimulating { to_sim: self.to_sim, from_sim: self.from_sim }, res)
    }
}