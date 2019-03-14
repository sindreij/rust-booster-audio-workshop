#[macro_use]
extern crate conrod;
extern crate conrod_derive;

extern crate audioengine;

mod event_loop;
mod types;
mod ui;

use std::sync::mpsc::channel;

use types::{Slider, SliderEvent, SliderEventType};

#[allow(unused_imports)]
use audioengine::types::KeyAction;

#[allow(unused_imports)]
use ui::Ui;

#[allow(unused_imports)]
use std::f64::consts::PI;

const X: f64 = 1.05946309436;

#[derive(Copy, Clone)]
enum AdsrState {
    Attack,
    Decay,
    Release,
}

struct Adsr {
    state: AdsrState,
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    value: f64,
}

impl Adsr {
    fn default() -> Adsr {
        Adsr {
            state: AdsrState::Release,
            attack: 10000.0,
            decay: 10000.0,
            sustain: 0.5,
            release: 10000.0,
            value: 0.0,
        }
    }

    fn next(&mut self, gate: f64) -> f64 {
        let next_val = match self.state {
            AdsrState::Attack => self.value + 1.0 / self.attack,
            AdsrState::Decay => self.value - (self.value - self.sustain) / self.decay,
            AdsrState::Release => self.value - self.value / self.release,
        };
        let next_state = match self.state {
            AdsrState::Attack if next_val >= 1.0 => AdsrState::Decay,
            AdsrState::Attack if gate < 0.5 => AdsrState::Release,
            AdsrState::Decay if gate < 0.5 => AdsrState::Release,
            AdsrState::Release if gate >= 0.5 => AdsrState::Attack,
            _ => self.state,
        };
        self.value = next_val;
        self.state = next_state;
        next_val
    }
}

struct Note {
    key: i32,
    adsr: Adsr,
}

#[allow(unused_variables)]
fn main() -> Result<(), Error> {
    let (sender, receiver) = channel::<Vec<f64>>();
    let (sliders_sender, sliders_receiver) = channel::<SliderEvent>();

    let audioengine = audioengine::EngineController::start();

    let sample_rate = audioengine.sample_rate;
    let time_per_sample = 1.0 / sample_rate;
    println!("Time per sample: {}", time_per_sample);

    let mut time = 0.0;
    let mut buffer = Vec::new();
    let mut last_val = 0.;

    let mut notes = Vec::new();
    for i in 0..15 {
        notes.push(Note {
            key: i,
            adsr: Adsr::default(),
        })
    }
    let synth = move |actions: &[i32]| {
        time += time_per_sample;

        let mut val: f64 = 0.0;

        for (slider_type, value) in sliders_receiver.try_iter() {
            match slider_type {
                SliderEventType::Attack => {
                    for note in &mut notes {
                        note.adsr.attack = value;
                    }
                }
                SliderEventType::Decay => {
                    for note in &mut notes {
                        note.adsr.decay = value;
                    }
                }
                SliderEventType::Release => {
                    for note in &mut notes {
                        note.adsr.release = value;
                    }
                }
                SliderEventType::Sustain => {
                    for note in &mut notes {
                        note.adsr.sustain = value;
                    }
                }
            }
        }

        for note in &mut notes {
            let gate = if actions.contains(&note.key) {
                1.0
            } else {
                0.0
            };

            let amp = note.adsr.next(gate);

            let res = {
                let freq = 261.63 * X.powf(note.key.into());

                let main = (2. * PI * freq * time + 0.).sin() * 0.5 * amp;
                let second = (2. * PI * freq * 2. * time + 0.).sin() * 0.5 * amp * 0.3;
                let third = (2. * PI * freq / 2. * time + 0.).sin() * 0.5 * amp * 0.3;

                // let main = (2. * PI * freq * time + 0.).sin().signum() * amp;
                // let second = (2. * PI * freq * 2. * time + 0.).sin().signum() * amp * 0.3;
                main + second + third
            };

            val += res;
        }

        // if let Some(action) = action.first() {
        //     gate = 1.0;
        //     if *action != current_key {
        //         current_key = *action;

        //         println!("{:?}", action);
        //     }
        // } else {
        //     gate = 0.0;
        // }

        // let amp = adsr.next(gate);

        if val > 0. && last_val <= 0. {
            sender.send(buffer.split_off(0)).expect("could not send")
        }
        // # square
        // let val = ;
        // # sin

        buffer.push(val);
        last_val = val;

        val
    };

    audioengine.set_processor_function(Box::new(synth));

    let sliders = vec![
        Slider {
            min: 1.,
            max: 20000.,
            default: 10000.,
            event_type: SliderEventType::Attack,
            label: "Attack".to_string(),
        },
        Slider {
            min: 1.,
            max: 20000.,
            default: 10000.,
            event_type: SliderEventType::Decay,
            label: "Decay".to_string(),
        },
        Slider {
            min: 0.,
            max: 1.,
            default: 0.5,
            event_type: SliderEventType::Sustain,
            label: "Sustain".to_string(),
        },
        Slider {
            min: 1.,
            max: 20000.,
            default: 10000.,
            event_type: SliderEventType::Release,
            label: "Release".to_string(),
        },
    ];

    let mut window = Ui::new(
        "Synthesizer",
        [1280.0, 800.0],
        audioengine,
        Some(&sliders),
        Some(sliders_sender),
        Some(receiver),
    );

    window.show();

    Ok(())
}

#[derive(Debug)]
enum Error {}
