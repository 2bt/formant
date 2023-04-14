use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use std::time::Duration;

const MIXRATE: f32 = 44100.0;
const WIN_W: u32 = 600;
const WIN_H: u32 = 600;


struct VoiceSynth {
    phase: f32,
    phase_inc: f32,
    phase_f1: f32,
    phase_f1_inc: f32,
    phase_f2: f32,
    phase_f2_inc: f32,
    volume: f32,
}

fn lerp(a: f32, b: f32, x: f32) -> f32 {
    a * (1.0 - x) + b * x
}
fn osc(x: f32) -> f32 {
    // (x * 4.0 - 2.0).abs() - 1.0
    (x * 2.0 * std::f32::consts::PI).sin()
}

impl VoiceSynth {
    fn new() -> VoiceSynth {
        VoiceSynth {
            phase: 0.0,
            phase_inc: 110.0 / MIXRATE,
            phase_f1: 0.0,
            phase_f1_inc: 0.0,
            phase_f2: 0.0,
            phase_f2_inc: 0.0,
            volume: 0.0,
        }
    }
    fn update(&mut self, p: &Point) {
        self.volume = 0.25;
        self.phase_f1_inc = lerp(250.0, 800.0, p.x as f32 / (WIN_W - 1) as f32) / MIXRATE;
        self.phase_f2_inc = lerp(500.0, 2500.0, 1.0 - p.y as f32 / (WIN_H - 1) as f32) / MIXRATE;
    }
}

impl AudioCallback for VoiceSynth {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {

            self.phase += self.phase_inc;
            if self.phase >= 1.0 {
                self.phase = 0.0;
                self.phase_f1 = 0.0;
                self.phase_f2 = 0.0;
            }

            self.phase_f1 = (self.phase_f1 + self.phase_f1_inc) % 1.0;
            self.phase_f2 = (self.phase_f2 + self.phase_f2_inc) % 1.0;

            *x = osc(self.phase_f1);
            *x += osc(self.phase_f2) * 0.8;
            *x *= self.volume;
            self.volume = (self.volume - 0.00001).max(0.0);
        }
    }
}

struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let sdl_context = sdl2::init().unwrap();

    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(MIXRATE as i32),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };
    let mut device = audio_subsystem
        .open_playback(None, &desired_spec, |_| VoiceSynth::new())
        .unwrap();
    device.resume();

    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("formant", WIN_W, WIN_H)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let mut p = Point { x: -100, y: 0 };

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Escape => break 'running,
                    _ => {}
                },
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    p.x = x;
                    p.y = y;
                    device.lock().update(&p);
                }
                Event::MouseMotion {
                    mousestate, x, y, ..
                } => {
                    if mousestate.left() {
                        p.x = x;
                        p.y = y;
                        device.lock().update(&p);
                    }
                }
                _ => {}
            }
        }

        canvas.set_draw_color(sdl2::pixels::Color::BLACK);
        canvas.clear();

        canvas.set_draw_color(sdl2::pixels::Color::RED);
        canvas
            .fill_rect(sdl2::rect::Rect::new(p.x - 10, p.y - 10, 20, 20))
            .unwrap();

        canvas.present();

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    // // Play for 2 seconds
    // std::thread::sleep(Duration::from_millis(2000));
}
