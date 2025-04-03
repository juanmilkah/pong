use std::time::Duration;
use std::{path::Path, thread};

use anyhow::anyhow;
use rand::{Rng, rng};
use sdl2::{keyboard::Keycode, pixels::Color, rect::Rect, ttf::FontStyle};

const BLOCK_SPEED: i32 = 5;
const BAR_SPEED: i32 = 70;
const BLOCK_WIDTH: u32 = 30;
const BLOCK_HEIGHT: u32 = 30;
const BAR_WIDTH: u32 = 200;
const BAR_HEIGHT: u32 = 30;
const BRICK_WIDTH: u32 = 60;
const BRICK_HEIGHT: u32 = 20;
const ROW_COUNT: usize = 5;
const COL_COUNT: usize = 10;
const BRICK_PADDING: u32 = 10;
const POINTS: u32 = 10;
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
const BRICK_COLOR: Color = Color {
    r: 255,
    g: 0,
    b: 0,
    a: 255,
};
const SCORE_COLOR: Color = Color {
    r: 255,
    g: 165,
    b: 0,
    a: 255,
};

struct GameState {
    score: u32,
    lives: u8,
    game_over: bool,
}

impl GameState {
    fn new() -> Self {
        Self {
            score: 0,
            lives: 3,
            game_over: false,
        }
    }
}

struct WinDimensions {
    width: u32,
    height: u32,
}

struct Brick {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    is_visible: bool,
}

impl Brick {
    fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            width: BRICK_WIDTH,
            height: BRICK_HEIGHT,
            is_visible: true,
        }
    }
}

fn get_bricks(w_dimensions: &WinDimensions) -> Vec<Brick> {
    let mut bricks = Vec::new();

    let total_width =
        (BRICK_WIDTH as i32 * COL_COUNT as i32) + (BRICK_PADDING as i32 * (COL_COUNT as i32 - 1));

    // starting  positions
    let start_x = (w_dimensions.width as i32 - total_width) / 2;
    let start_y = 50;

    for row in 0..ROW_COUNT {
        for col in 0..COL_COUNT {
            let brick_x = start_x + col as i32 * (BRICK_WIDTH as i32 + BRICK_PADDING as i32);
            let brick_y = start_y + row as i32 * (BRICK_HEIGHT as i32 + BRICK_PADDING as i32);
            bricks.push(Brick::new(brick_x, brick_y));
        }
    }
    bricks
}

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
    fn new(w_dimensions: &WinDimensions) -> Self {
        let mut rng = rng();
        let width = (w_dimensions.width as f32 * 0.05) as u32; // 5% of screen width
        let x = rng.random_range(0..w_dimensions.width - width) as i32;
        let padding = w_dimensions.height as i32 - (w_dimensions.height as f32 * 0.10) as i32;
        Self {
            x,
            y: padding,
            width: BLOCK_WIDTH,
            height: BLOCK_HEIGHT,
            velocity_x: 0,
            velocity_y: -1,
            first_bounce: true,
        }
    }

    fn update_position(
        &mut self,
        bar: &Bar,
        bricks: &mut Vec<Brick>,
        w_dimensions: &WinDimensions,
        game: &mut GameState,
    ) {
        let old_x = self.x;
        let old_y = self.y;
        // update position based on velocity and SPEED
        self.x += self.velocity_x * BLOCK_SPEED;
        self.y += self.velocity_y * BLOCK_SPEED;

        let mut rng = rng();

        for brick in bricks.iter_mut() {
            if !brick.is_visible {
                continue;
            }

            // check block collision with brick
            if self.x < brick.x + brick.width as i32
                && self.x + self.width as i32 > brick.x
                && self.y < brick.y + brick.height as i32
                && self.y + self.height as i32 > brick.y
            {
                brick.is_visible = false;
                game.score += POINTS;

                // determine bounce direction
                // top or below
                if old_y + self.height as i32 <= brick.y || old_y >= brick.y + brick.height as i32 {
                    self.velocity_y = -self.velocity_y;
                } else if old_x + self.width as i32 <= brick.x
                    || old_x >= brick.x + brick.width as i32
                {
                    self.velocity_y = -self.velocity_y;
                }
                break;
            }

            // check horizontal boundaries
            if !self.first_bounce {
                if self.x <= 0 {
                    self.x = 0;
                    self.velocity_x = 1;
                } else if self.x + self.width as i32 >= w_dimensions.width as i32 {
                    self.x = w_dimensions.width as i32 - self.width as i32;
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
            } else if self.y + self.width as i32 >= w_dimensions.height as i32 {
                game.lives -= 1;
                if game.lives == 0 {
                    game.game_over = true;
                }

                //reset block position
                *self = Block::new(w_dimensions);
                return;
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
    fn new(w_dimensions: &WinDimensions) -> Self {
        let x = (w_dimensions.width as i32 - BAR_WIDTH as i32) / 2; // center
        let y = w_dimensions.height as i32 - (w_dimensions.height as f32 * 0.10) as i32;
        Self {
            x,
            y,
            width: BAR_WIDTH,
            height: BAR_HEIGHT,
            velocity_x: 1,
        }
    }

    fn update_position(&mut self, dir: Direction, w_dimensions: &WinDimensions) {
        self.velocity_x = match dir {
            Direction::Right => 1,
            Direction::Left => -1,
        };

        self.x += self.velocity_x * BAR_SPEED;

        // check horizontal boundaries
        if self.x <= 0 {
            self.x = 0;
            self.velocity_x = 1;
        } else if self.x + self.width as i32 >= w_dimensions.width as i32 {
            self.x = w_dimensions.width as i32 - self.width as i32;
            self.velocity_x = -1;
        }
    }
}

fn main() -> anyhow::Result<()> {
    let sdl_context = sdl2::init().map_err(|err| anyhow::anyhow!("initialise sdl: {}", err))?;
    let ttf_context = sdl2::ttf::init().map_err(|err| anyhow!("Initialise ttf: {}", err))?;
    let font_path = Path::new("resources/EnvyCodeRNerdFontMono-Bold.ttf");

    let mut font = ttf_context
        .load_font(font_path, 36)
        .map_err(|err| anyhow!("load font:{}", err))?;
    font.set_style(FontStyle::BOLD);

    let video_subsystem = sdl_context
        .video()
        .map_err(|err| anyhow!("get video subsystem: {}", err))?;

    let display_bounds = video_subsystem
        .display_bounds(0)
        .map_err(|err| anyhow!("display bounds: {}", err))?;

    let w_dimensions = WinDimensions {
        width: display_bounds.width(),
        height: display_bounds.height(),
    };

    let window = video_subsystem
        .window("Pong", w_dimensions.width, w_dimensions.height)
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

    let mut game = GameState::new();
    let mut block = Block::new(&w_dimensions);
    let mut bar = Bar::new(&w_dimensions);
    let mut bricks = get_bricks(&w_dimensions);

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        if game.game_over {
            break;
        }

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
                    bar.update_position(Direction::Left, &w_dimensions);
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::RIGHT),
                    ..
                } => {
                    bar.update_position(Direction::Right, &w_dimensions);
                }
                _ => {}
            }
        }

        // clear screen
        canvas.set_draw_color(WINDOW_COLOR);
        canvas.clear();

        let score_text = format!("SCORE: {} LIVES {}", game.score, game.lives);
        let surface = font
            .render(&score_text)
            .blended(SCORE_COLOR)
            .map_err(|err| anyhow!("render score: {}", err))?;

        let tx_creator = canvas.texture_creator();
        let texture = tx_creator
            .create_texture_from_surface(&surface)
            .map_err(|err| anyhow!("create texture from surface: {}", err))?;
        let s_target = Rect::new(10, 10, surface.width(), surface.height());
        canvas
            .copy(&texture, None, s_target)
            .map_err(|err| anyhow!("copy texture to score_target: {}", err))?;

        canvas.set_draw_color(BRICK_COLOR);
        let any_visible = bricks.iter().any(|brick| brick.is_visible);
        if !any_visible {
            game.game_over = true;
            break 'running;
        }

        for brick in &bricks {
            if brick.is_visible {
                let brick_position = Rect::new(brick.x, brick.y, brick.width, brick.height);
                canvas
                    .fill_rect(brick_position)
                    .map_err(|err| anyhow!("fill brick: {}", err))?;
            }
        }

        canvas.set_draw_color(BAR_COLOR);
        let b_position = Rect::new(bar.x, bar.y, bar.width, bar.height);
        canvas
            .fill_rect(b_position)
            .map_err(|err| anyhow!("fill bar: {}", err))?;

        let position = Rect::new(block.x, block.y, block.width, block.height);
        canvas.set_draw_color(BLOCK_COLOR);
        block.update_position(&bar, &mut bricks, &w_dimensions, &mut game);
        canvas
            .fill_rect(position)
            .map_err(|err| anyhow!("fill block: {}", err))?;
        canvas.present();
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    if game.game_over {
        canvas.set_draw_color(WINDOW_COLOR);
        canvas.clear();

        let text = format!("GAME OVER! \nSCORE: {}\nPress Esc to Exit Game", game.score);

        let surface = font
            .render(&text)
            .blended(SCORE_COLOR)
            .map_err(|err| anyhow!("render game over message: {}", err))?;

        let tx_creator = canvas.texture_creator();
        let texture = tx_creator
            .create_texture_from_surface(&surface)
            .map_err(|err| anyhow!("create texture from surface: {}", err))?;
        let target = Rect::new(
            (w_dimensions.width - surface.width()) as i32 / 2,
            (w_dimensions.width - surface.height()) as i32 / 2 - 50,
            surface.width(),
            surface.height(),
        );
        canvas
            .copy(&texture, None, target)
            .map_err(|err| anyhow!("copy texture to game_over target: {}", err))?;

        canvas.present();

        'game_over: loop {
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. }
                    | sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'game_over,
                    _ => {}
                }
            }

            // Short delay to prevent high CPU usage
            thread::sleep(Duration::from_millis(100));
        }
    }

    Ok(())
}
