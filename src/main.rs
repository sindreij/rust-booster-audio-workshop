#[macro_use]
extern crate conrod;
extern crate conrod_derive;

extern crate audioengine;

mod event_loop;
mod types;
mod ui;

use std::sync::mpsc::channel;

#[allow(unused_imports)]
use audioengine::types::KeyAction;

#[allow(unused_imports)]
use ui::Ui;

#[allow(unused_imports)]
use std::f64::consts::PI;

const X: f64 = 1.05946309436;

#[allow(unused_variables)]
fn main() -> Result<(), Error> {
    let (sender, receiver) = channel::<Vec<f64>>();

    let audioengine = audioengine::EngineController::start();

    let sample_rate = audioengine.sample_rate;
    let time_per_sample = 1.0 / sample_rate;
    println!("Time per sample: {}", time_per_sample);

    let mut time = 0.0;
    let mut last_send_time = 0.;
    let mut buffer = Vec::new();

    let mut current_key = None;
    let synth = move |action: Option<i32>| {
        time += time_per_sample;
        if action != current_key {
            current_key = action;

            println!("{:?}", action);
        }

        if time - last_send_time > 0.1 {
            last_send_time = time;
            sender.send(buffer.split_off(0)).expect("could not send")
        }

        let res = match current_key {
            Some(key) => {
                let freq = 261.63 * X.powf(key.into());
                (2. * PI * freq * time + 0.).sin() * 0.5
            }
            None => 0.0,
        };
        // # square
        // let res = (2. * PI * 880. * time + 0.).sin().signum() * 0.5;
        // # sin

        buffer.push(res);

        res
    };

    audioengine.set_processor_function(Box::new(synth));

    let mut window = Ui::new(
        "Synthesizer",
        [1280.0, 800.0],
        audioengine,
        None,
        None,
        Some(receiver),
    );

    window.show();

    Ok(())
}

#[derive(Debug)]
enum Error {}
