mod support;
mod controls;
mod cells;
mod simulation;
mod canvas;
mod worker;
mod config;

use canvas::Canvas;
use controls::Controls;

fn main() {
    let system = support::init("glauber-rs");
    let simulation = config::DEFAULT_SIMULATION_TYPE.create(config::DEFAULT_SIZE);
    let mut controls = Controls::new();
    let mut canvas = Canvas::new();
    let mut worker = Some(worker::create_worker_thread().pause_simulation(simulation));
    system.main_loop(|_, mut frame_data| {
        let (empty_worker, (mut simulation, iters)) = worker.take().unwrap().stop_simulation();
        canvas.draw(&mut frame_data, &mut *simulation);
        controls.draw(frame_data.ui, &mut simulation, iters);
        worker = Some(match controls.paused {
            true => empty_worker.pause_simulation(simulation),
            false => empty_worker.start_simulation(simulation),
        })
    });
}