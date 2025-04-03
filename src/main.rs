use std::thread;
use std::time::Duration;

use anyhow::anyhow;
use rand::{Rng, rng};
use sdl2::{keyboard::Keycode, pixels::Color, rect::Rect};

const W_WIDTH: u32 = 800;
const W_HEIGHT: u32 = 600;
const SPEED: i32 = 5;
const BLOCK_WIDTH: u32 = 30;
const BLOCK_HEIGHT: u32 = 30;

struct Block {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    velocity_x: i32,
    velocity_y: i32,
}

impl Block {
    fn new() -> Self {
        let mut rng = rng();
        let width = rng.random_range(30..70);
        let height = rng.random_range(30..70);
        let x = rng.random_range(0..W_WIDTH - width) as i32;
        let y = rng.random_range(0..W_HEIGHT - height) as i32;

        let velocity_x = if rng.random_bool(0.5) { 1 } else { -1 };
        let velocity_y = if rng.random_bool(0.5) { 1 } else { -1 };
        Self {
            x,
            y,
            width: BLOCK_WIDTH,
            height: BLOCK_HEIGHT,
            velocity_x,
            velocity_y,
        }
    }
}

fn get_block_position(block: &mut Block) -> Rect {
    // update position based on velocity and SPEED
    block.x += block.velocity_x * SPEED;
    block.y += block.velocity_y * SPEED;

    // check horizontal boundaries
    if block.x <= 0 {
        block.x = 0;
        block.velocity_x = 1;
    } else if block.x + block.width as i32 >= W_WIDTH as i32 {
        block.x = W_WIDTH as i32 - block.width as i32;
        block.velocity_x = -1;
    }

    // check for vertical boundaries
    if block.y <= 0 {
        block.y = 0;
        block.velocity_y = 1;
    } else if block.y + block.width as i32 >= W_HEIGHT as i32 {
        block.y = W_HEIGHT as i32 - block.width as i32;
        block.velocity_y = -1;
    }
    Rect::new(block.x, block.y, block.width, block.height)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sdl_context = sdl2::init().map_err(|err| anyhow::anyhow!("initialise sdl: {}", err))?;
    let video_subsystem = sdl_context
        .video()
        .map_err(|err| anyhow!("get video subsystem: {}", err))?;
    let window = video_subsystem
        .window("Wins", W_WIDTH, W_HEIGHT)
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

    // 5 blocks
    let mut blocks = (0..5).map(|_| Block::new()).collect::<Vec<Block>>();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let b_color = Color {
            r: 0,
            g: 255,
            b: 0,
            a: 5,
        };
        canvas.set_draw_color(b_color);

        for block in blocks.iter_mut() {
            canvas
                .fill_rect(get_block_position(block))
                .map_err(|err| anyhow!("ERROR: {}", err))?;
        }
        canvas.present();
        let c_color = Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        };
        canvas.set_draw_color(c_color);
        canvas.clear();
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
