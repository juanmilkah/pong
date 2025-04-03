use std::thread;
use std::time::Duration;

use anyhow::anyhow;
use rand::{Rng, rng};
use sdl2::{keyboard::Keycode, pixels::Color, rect::Rect};

const W_WIDTH: u32 = 800;
const W_HEIGHT: u32 = 600;
const BLOCK_SPEED: i32 = 5;
const BAR_SPEED: i32 = 60;
const BLOCK_WIDTH: u32 = 30;
const BLOCK_HEIGHT: u32 = 30;
const BAR_WIDTH: u32 = 200;
const BAR_HEIGHT: u32 = 30;
const PADDING: i32 = 50;
const BAR_COLOR: Color = Color {
    r: 0,
    g: 0,
    b: 255,
    a: 255,
};
const BLOCK_COLOR: Color = Color {
    r: 0,
    g: 255,
    b: 0,
    a: 255,
};
const WINDOW_COLOR: Color = Color {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
};

struct Block {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    velocity_x: i32,
    velocity_y: i32,
    first_bounce: bool,
}

impl Block {
    fn new() -> Self {
        let mut rng = rng();
        let width = rng.random_range(30..70);
        let x = rng.random_range(0..W_WIDTH - width) as i32;

        Self {
            x,
            y: PADDING,
            width: BLOCK_WIDTH,
            height: BLOCK_HEIGHT,
            velocity_x: 0,
            velocity_y: 1,
            first_bounce: true,
        }
    }

    fn update_position(&mut self, bar: &Bar) {
        let old_y = self.y;
        // update position based on velocity and SPEED
        self.x += self.velocity_x * BLOCK_SPEED;
        self.y += self.velocity_y * BLOCK_SPEED;

        let mut rng = rng();

        // check horizontal boundaries
        if !self.first_bounce {
            if self.x <= 0 {
                self.x = 0;
                self.velocity_x = 1;
            } else if self.x + self.width as i32 >= W_WIDTH as i32 {
                self.x = W_WIDTH as i32 - self.width as i32;
                self.velocity_x = -1;
            }
        }

        // check for vertical boundaries
        if self.y <= 0 {
            self.y = 0;
            self.velocity_y = 1;
            if self.first_bounce {
                self.first_bounce = false;
                // randomly choose x direction
                self.velocity_x = if rng.random_bool(0.5) { 1 } else { -1 };
            }
        } else if self.y + self.width as i32 >= W_HEIGHT as i32 {
            self.y = W_HEIGHT as i32 - self.width as i32;
            self.velocity_y = -1;
            if self.first_bounce {
                self.first_bounce = false;
                // randomly choose x direction
                self.velocity_x = if rng.random_bool(0.5) { 1 } else { -1 };
            }
        }

        // check for collision with bar
        if self.velocity_y > 0 // moving downwards 
            && self.y + self.height as i32 >= bar.y // block on top of bar 
            && old_y + self.height as i32 <= bar.y // on prev frame block above bar
            && self.x + self.width as i32 >= bar.x // block left > bar right
            && self.x <= bar.x +bar.width as i32
        // block right < bar left
        {
            // bounce the block off the bar
            self.velocity_y = -1;
            self.y = bar.y - self.height as i32;
            if self.first_bounce {
                self.first_bounce = false;
                // randomly choose x direction
                self.velocity_x = if rng.random_bool(0.5) { 1 } else { -1 };
            }
        }
    }
}

enum Direction {
    Left,
    Right,
}

struct Bar {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    velocity_x: i32,
}

impl Bar {
    fn new() -> Self {
        let x = (W_WIDTH as i32 - BAR_WIDTH as i32) / 2; // center
        let y = W_HEIGHT as i32 - PADDING;
        Self {
            x,
            y,
            width: BAR_WIDTH,
            height: BAR_HEIGHT,
            velocity_x: 1,
        }
    }

    fn update_position(&mut self, dir: Direction) {
        self.velocity_x = match dir {
            Direction::Right => 1,
            Direction::Left => -1,
        };

        self.x += self.velocity_x * BAR_SPEED;

        // check horizontal boundaries
        if self.x <= 0 {
            self.x = 0;
            self.velocity_x = 1;
        } else if self.x + self.width as i32 >= W_WIDTH as i32 {
            self.x = W_WIDTH as i32 - self.width as i32;
            self.velocity_x = -1;
        }
    }
}

fn main() -> anyhow::Result<()> {
    let sdl_context = sdl2::init().map_err(|err| anyhow::anyhow!("initialise sdl: {}", err))?;
    let video_subsystem = sdl_context
        .video()
        .map_err(|err| anyhow!("get video subsystem: {}", err))?;
    let window = video_subsystem
        .window("Pong", W_WIDTH, W_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|err| anyhow!("build window: {}", err))?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|err| anyhow!("window into canvas: {}", err))?;

    let mut block = Block::new();
    let mut bar = Bar::new();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::LEFT),
                    ..
                } => {
                    bar.update_position(Direction::Left);
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::RIGHT),
                    ..
                } => {
                    bar.update_position(Direction::Right);
                }
                _ => {}
            }
        }

        // clear screen
        canvas.set_draw_color(WINDOW_COLOR);
        canvas.clear();

        canvas.set_draw_color(BAR_COLOR);
        let b_position = Rect::new(bar.x, bar.y, bar.width, bar.height);
        canvas
            .fill_rect(b_position)
            .map_err(|err| anyhow!("ERROR: {}", err))?;

        let position = Rect::new(block.x, block.y, block.width, block.height);
        canvas.set_draw_color(BLOCK_COLOR);
        block.update_position(&bar);
        canvas
            .fill_rect(position)
            .map_err(|err| anyhow!("ERROR: {}", err))?;
        canvas.present();
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
