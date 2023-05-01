use std::{thread, time::Duration};

use sdl2::{
    event::Event,
    gfx::primitives::DrawRenderer,
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    render::WindowCanvas,
};

const PARTICLE_RADIUS: i32 = 10;
const CHARGE_STEP: f64 = 1.602176634e-19;
const COULUMBS_CONST: f64 = 8.9875517923e9;
const MAX_LINE_ITERS: usize = 4096;

fn draw_particle(canvas: &mut WindowCanvas, particle: Particle, x: i16, y: i16) {
    match particle {
        // Positive charged particle, draw red circle with plus in it
        Particle::Positive => {
            canvas
                .filled_circle(x, y, PARTICLE_RADIUS as i16, Color::RGB(255, 0, 0))
                .unwrap();
            canvas.set_draw_color(Color::WHITE);
            canvas
                .fill_rect(Rect::new(
                    x as i32 - PARTICLE_RADIUS / 10,
                    y as i32 - PARTICLE_RADIUS / 2,
                    (PARTICLE_RADIUS / 5) as u32,
                    PARTICLE_RADIUS as u32,
                ))
                .unwrap();
            canvas
                .fill_rect(Rect::new(
                    x as i32 - PARTICLE_RADIUS / 2,
                    y as i32 - PARTICLE_RADIUS / 10,
                    PARTICLE_RADIUS as u32,
                    (PARTICLE_RADIUS / 5) as u32,
                ))
                .unwrap();
        }
        // Negatively charged particle, draw blue circle with plus in it
        Particle::Negative => {
            canvas
                .filled_circle(x, y, PARTICLE_RADIUS as i16, Color::RGB(0, 0, 255))
                .unwrap();
            canvas.set_draw_color(Color::WHITE);
            canvas
                .fill_rect(Rect::new(
                    x as i32 - PARTICLE_RADIUS / 2,
                    y as i32 - PARTICLE_RADIUS / 10,
                    PARTICLE_RADIUS as u32,
                    (PARTICLE_RADIUS / 5) as u32,
                ))
                .unwrap();
        }
        // Neutral particle, draw gray circle with an `n` in it
        Particle::Neutral => {
            canvas
                .filled_circle(x, y, PARTICLE_RADIUS as i16, Color::RGB(50, 50, 50))
                .unwrap();
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum Particle {
    Positive,
    Negative,
    Neutral,
}

struct Toolbar {
    selected_part: Option<Particle>,
    choices: Vec<Particle>,
}

impl Default for Toolbar {
    fn default() -> Self {
        Self {
            selected_part: None,
            choices: vec![Particle::Positive, Particle::Negative, Particle::Neutral],
        }
    }
}

impl Toolbar {
    fn get_option_rect(&self, canvas: &WindowCanvas, index: usize) -> Rect {
        let top_left = (canvas.output_size().unwrap().0 * 9 / 10, 0);
        let bottom_right = canvas.output_size().unwrap();

        let y_inc = (bottom_right.1 - top_left.1) / self.choices.len() as u32;
        let opt_top_left = (top_left.0, top_left.1 + (index as u32) * y_inc);

        Rect::new(
            opt_top_left.0 as i32,
            opt_top_left.1 as i32,
            bottom_right.0 - top_left.0,
            y_inc,
        )
    }

    fn get_selected_option(&self) -> Option<Particle> {
        self.selected_part
    }

    fn handle_mouse_down(&mut self, canvas: &mut WindowCanvas, x: i32, y: i32) {
        for i in 0..self.choices.len() {
            let opt_rect = self.get_option_rect(canvas, i);

            if x >= opt_rect.x
                && x <= opt_rect.x + opt_rect.w
                && y >= opt_rect.y
                && y <= opt_rect.y + opt_rect.h
            {
                self.selected_part = Some(self.choices[i]);
                break;
            }
        }

        self.on_update(canvas);
    }

    fn on_update(&self, canvas: &mut WindowCanvas) {
        let output_size = canvas.output_size().unwrap();

        canvas.set_draw_color(Color::GRAY);
        canvas
            .fill_rect(Rect::new(
                (output_size.0 * 9 / 10) as i32,
                0,
                output_size.0 / 10,
                output_size.1,
            ))
            .unwrap();

        for (i, c) in self.choices.iter().enumerate() {
            let opt_rect = self.get_option_rect(canvas, i);

            println!("{opt_rect:?}");

            if Some(*c) == self.selected_part {
                canvas.set_draw_color(Color::WHITE);
                canvas.draw_rect(opt_rect).unwrap();
            }

            draw_particle(
                canvas,
                *c,
                (opt_rect.x + opt_rect.w / 2) as i16,
                (opt_rect.y + opt_rect.h / 2) as i16,
            );
        }
    }
}

#[derive(Default)]
struct Game {
    particles: Vec<(f64, f64, f64)>,
    current_selected_charge: f64,
}

impl Game {
    fn get_field_strength(&self, x: f64, y: f64) -> (f64, f64) {
        let mut total_strength = (0.0, 0.0);

        for (part_x, part_y, charge) in self.particles.iter() {
            let direct_vec = (x - part_x, y - part_y);
            let direct_mag = direct_vec.0.hypot(direct_vec.1);

            let force_mag = COULUMBS_CONST * charge / (direct_mag * direct_mag);

            total_strength.0 += force_mag * direct_vec.0 / direct_mag;
            total_strength.1 += force_mag * direct_vec.1 / direct_mag;
        }

        total_strength
    }

    fn handle_mouse_down(&mut self, canvas: &mut WindowCanvas, x: i32, y: i32) {
        self.particles
            .push((x as f64, y as f64, self.current_selected_charge));
        println!(
            "Added particle with charge {} at ({x}, {y})",
            self.current_selected_charge
        );

        self.on_update(canvas);
    }

    fn handle_keydown(&mut self, _canvas: &mut WindowCanvas, keycode: Keycode) {
        match keycode {
            Keycode::Equals => {
                self.current_selected_charge += CHARGE_STEP;
            }
            Keycode::Minus => {
                self.current_selected_charge -= CHARGE_STEP;
            }
            Keycode::N => {
                self.current_selected_charge = 0.0;
            }
            _ => {}
        }

        if self.current_selected_charge.abs() < 1e-30 {
            self.current_selected_charge = 0.0;
        }

        println!("Current charge: {}", self.current_selected_charge);
    }

    fn handle_keyup(&mut self, _canvas: &mut WindowCanvas, _keycode: Keycode) {}

    fn on_update(&self, canvas: &mut WindowCanvas) {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        for (x, y, charge) in self.particles.iter() {
            let (rnded_x, rnded_y, charge) = (*x as i16, *y as i16, *charge);

            let particle_type = if charge > 0.0 {
                Particle::Positive
            } else if charge < 0.0 {
                Particle::Negative
            } else {
                Particle::Neutral
            };

            draw_particle(canvas, particle_type, rnded_x, rnded_y);
        }

        // Now we create 8 protruding lines from each positive particle
        for (x, y, charge) in self.particles.iter().filter(|(_, _, charge)| *charge > 0.0) {
            let (x, y, _charge) = (*x, *y, *charge);

            for i in 0..16 {
                let mut line_points = vec![];

                let starting_angle = i as f64 * std::f64::consts::PI / 8.0;
                let mut current_pos = (
                    x + (PARTICLE_RADIUS as f64 * 1.1) * starting_angle.cos(),
                    y + (PARTICLE_RADIUS as f64 * 1.1) * starting_angle.sin(),
                );

                line_points.push(Point::new(current_pos.0 as i32, current_pos.1 as i32));

                for _ in 0..MAX_LINE_ITERS {
                    let field_strength = self.get_field_strength(current_pos.0, current_pos.1);
                    let field_strength_mag = field_strength.0.hypot(field_strength.1);

                    current_pos.0 += field_strength.0 / field_strength_mag;
                    current_pos.1 += field_strength.1 / field_strength_mag;

                    if self
                        .particles
                        .iter()
                        .filter(|(_, _, charge)| *charge < 0.0)
                        .find(|(x, y, _)| {
                            (current_pos.0 - *x).hypot(current_pos.1 - *y)
                                <= PARTICLE_RADIUS as f64 * 1.1
                        })
                        .is_some()
                    {
                        break;
                    }

                    line_points.push(Point::new(current_pos.0 as i32, current_pos.1 as i32));
                }

                canvas.set_draw_color(Color::WHITE);
                canvas.draw_lines(&line_points[..]).unwrap();
            }
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Electric Field Visualizer", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut event_pump = sdl_context.event_pump()?;
    let mut game = Game::default();
    let mut toolbar = Toolbar::default();
    let mut canvas = window.into_canvas().build()?;

    let mut _frame_num = 0;

    game.on_update(&mut canvas);
    toolbar.on_update(&mut canvas);

    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'main_loop;
                }
                Event::MouseMotion { x, y, .. } => {
                    let (x_comp, y_comp) = game.get_field_strength(x as f64, y as f64);
                    println!(
                        "Total field strength @ ({x}, {y}): {}",
                        x_comp.hypot(y_comp)
                    );
                }
                Event::MouseButtonDown { x, y, .. } => {
                    if (x as u32) < canvas.output_size().unwrap().0 * 9 / 10 {
                        game.handle_mouse_down(&mut canvas, x, y);
                    } else {
                        toolbar.handle_mouse_down(&mut canvas, x, y);
                    }

                    toolbar.on_update(&mut canvas);
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Keycode::Q = keycode {
                        break 'main_loop;
                    }

                    game.handle_keydown(&mut canvas, keycode);
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    game.handle_keyup(&mut canvas, keycode);
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::GRAY);

        canvas.present();

        thread::sleep(Duration::from_millis(16));

        _frame_num += 1;
    }

    Ok(())
}
