use std::time::Duration;
use std::{path::Path, thread};

use anyhow::anyhow;
use rand::{Rng, rng};
use sdl2::keyboard::KeyboardState;
use sdl2::{keyboard::Keycode, pixels::Color, rect::Rect, ttf::FontStyle};

const BALL_SPEED: i32 = 7;
const BAR_SPEED: i32 = 20;
const BALL_WIDTH: u32 = 30;
const BALL_HEIGHT: u32 = 30;
const BAR_WIDTH: u32 = 200;
const BAR_HEIGHT: u32 = 30;
const BRICK_WIDTH: u32 = 60;
const BRICK_HEIGHT: u32 = 20;
const ROW_COUNT: usize = 5;
const BRICK_PADDING: u32 = 10;
const POINTS: u32 = 10;
const BAR_COLOR: Color = Color {
    r: 0,
    g: 0,
    b: 255,
    a: 255,
};
const BALL_COLOR: Color = Color {
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

// Game Struct
// Init()
// this sets up everything
// Play()
// Exit()

struct Game {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
    font: sdl2::ttf::Font<'static, 'static>,
    state: GameState,
    bar: Bar,
    ball: Ball,
    bricks: Vec<Brick>,
    dimensions: WinDimensions,
}

impl Game {
    fn new() -> anyhow::Result<Self> {
        let sdl_context = sdl2::init().map_err(|err| anyhow::anyhow!("initialise sdl: {}", err))?;
        // leaked for static lifetime
        let ttf_context = Box::leak(Box::new(
            sdl2::ttf::init().map_err(|err| anyhow!("Initialise ttf: {}", err))?,
        ));
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

        let dimensions = WinDimensions {
            width: display_bounds.width(),
            height: display_bounds.height(),
        };

        let window = video_subsystem
            .window("Pong", dimensions.width, dimensions.height)
            .position_centered()
            .opengl()
            .build()
            .map_err(|err| anyhow!("build window: {}", err))?;

        let canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|err| anyhow!("window into canvas: {}", err))?;

        let bar = Bar::new(&dimensions);
        let bricks = Vec::new();
        let event_pump = sdl_context.event_pump().unwrap();
        let state = GameState::new();
        let ball = Ball::new(&bar);
        Ok(Self {
            bar,
            bricks,
            canvas,
            event_pump,
            font,
            state,
            ball,
            dimensions,
        })
    }

    fn play(&mut self) -> anyhow::Result<()> {
        self.get_bricks();
        while !self.state.game_over {
            for event in self.event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. }
                    | sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => self.state.game_over = true,
                    _ => {}
                }
            }

            let k_state = KeyboardState::new(&self.event_pump);
            let left_pressed = k_state.is_scancode_pressed(sdl2::keyboard::Scancode::Left);
            let right_pressed = k_state.is_scancode_pressed(sdl2::keyboard::Scancode::Right);
            if left_pressed {
                self.update_bar_position(Direction::Left);
            }
            if right_pressed {
                self.update_bar_position(Direction::Right);
            };
            // clear screen
            self.canvas.set_draw_color(WINDOW_COLOR);
            self.canvas.clear();

            let score_text = format!("SCORE: {} LIVES {}", self.state.score, self.state.lives);
            let surface = self
                .font
                .render(&score_text)
                .blended(SCORE_COLOR)
                .map_err(|err| anyhow!("render score: {}", err))?;

            let tx_creator = self.canvas.texture_creator();
            let texture = tx_creator
                .create_texture_from_surface(&surface)
                .map_err(|err| anyhow!("create texture from surface: {}", err))?;
            let s_target = Rect::new(10, 10, surface.width(), surface.height());
            self.canvas
                .copy(&texture, None, s_target)
                .map_err(|err| anyhow!("copy texture to score_target: {}", err))?;

            self.canvas.set_draw_color(BRICK_COLOR);
            let any_visible = self.bricks.iter().any(|brick| brick.is_visible);
            if !any_visible {
                self.state.game_over = true;
            }

            for brick in &self.bricks {
                if brick.is_visible {
                    let brick_position = Rect::new(brick.x, brick.y, brick.width, brick.height);
                    self.canvas
                        .fill_rect(brick_position)
                        .map_err(|err| anyhow!("fill brick: {}", err))?;
                }
            }

            self.canvas.set_draw_color(BAR_COLOR);
            let b_position = Rect::new(self.bar.x, self.bar.y, self.bar.width, self.bar.height);
            self.canvas
                .fill_rect(b_position)
                .map_err(|err| anyhow!("fill bar: {}", err))?;

            let position = Rect::new(self.ball.x, self.ball.y, self.ball.width, self.ball.height);
            self.canvas.set_draw_color(BALL_COLOR);
            self.update_ball_position();
            self.canvas
                .fill_rect(position)
                .map_err(|err| anyhow!("fill ball: {}", err))?;
            self.canvas.present();
            thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(())
    }

    fn update_ball_position(&mut self) {
        let old_y = self.ball.y;

        // update position based on velocity and BALL_SPEED
        self.ball.x += (self.ball.velocity_x * BALL_SPEED as f32) as i32;
        self.ball.y += (self.ball.velocity_y * BALL_SPEED as f32) as i32;

        let mut rng = rng();
        let mut collision_occurred = false;

        // Check brick collisions
        for brick in self.bricks.iter_mut() {
            if !brick.is_visible {
                continue;
            }

            // check ball collision with brick
            if self.ball.x < brick.x + brick.width as i32
                && self.ball.x + self.ball.width as i32 > brick.x
                && self.ball.y < brick.y + brick.height as i32
                && self.ball.y + self.ball.height as i32 > brick.y
            {
                brick.is_visible = false;
                self.state.score += POINTS;
                collision_occurred = true;

                // Calculate collision physics with brick
                let ball_center_x = self.ball.x + (self.ball.width as i32 / 2);
                let ball_center_y = self.ball.y + (self.ball.height as i32 / 2);
                let brick_center_x = brick.x + (brick.width as i32 / 2);
                let brick_center_y = brick.y + (brick.height as i32 / 2);

                // Calculate angle of impact for more realistic bounce
                let dx = ball_center_x - brick_center_x;
                let dy = ball_center_y - brick_center_y;

                // Determine the collision side (top/bottom or left/right)
                // For simplicity, we'll use penetration depth to decide
                let w = (self.ball.width as i32 / 2) + (brick.width as i32 / 2);
                let h = (self.ball.height as i32 / 2) + (brick.height as i32 / 2);
                let wx = w - (dx.abs());
                let wy = h - (dy.abs());

                // Bounce based on the side with least penetration
                if wx < wy {
                    // Horizontal collision (left/right)
                    self.ball.velocity_x = -self.ball.velocity_x;

                    // Add slight angle variation for more interesting gameplay
                    self.ball.velocity_y += rng.random_range(-10..10) as f32 / 100.0;
                } else {
                    // Vertical collision (top/bottom)
                    self.ball.velocity_y = -self.ball.velocity_y;

                    // Add slight angle variation for more interesting gameplay
                    self.ball.velocity_x += rng.random_range(-10..10) as f32 / 100.0;
                }

                // Normalize velocity to maintain consistent speed
                self.ball.normalize_velocity();
                break;
            }
        }

        // check horizontal boundaries
        if self.ball.x <= 0 {
            self.ball.x = 0;
            self.ball.velocity_x = self.ball.velocity_x.abs(); // Ensure positive
            collision_occurred = true;
        } else if self.ball.x + self.ball.width as i32 >= self.dimensions.width as i32 {
            self.ball.x = self.dimensions.width as i32 - self.ball.width as i32;
            self.ball.velocity_x = -self.ball.velocity_x.abs(); // Ensure negative
            collision_occurred = true;
        }

        // check for vertical boundaries
        if self.ball.y <= 0 {
            self.ball.y = 0;
            self.ball.velocity_y = self.ball.velocity_y.abs(); // Ensure positive
            collision_occurred = true;

            if self.ball.first_bounce {
                self.ball.first_bounce = false;
                // randomly choose x direction
                self.ball.velocity_x = if rng.random_bool(0.5) { 1.0 } else { -1.0 };
            }
        } else if self.ball.y + self.ball.height as i32 >= self.dimensions.height as i32 {
            self.state.lives -= 1;
            if self.state.lives == 0 {
                self.state.game_over = true;
            }

            //reset ball position
            self.ball = Ball::new(&self.bar);
            return;
        }

        // check for collision with bar
        if self.ball.velocity_y > 0.0 // moving downwards
            && self.ball.y + self.ball.height as i32 >= self.bar.y // ball bottom at or below bar top
            && old_y + self.ball.height as i32 <= self.bar.y // on prev frame ball was above bar
            && self.ball.x + self.ball.width as i32 >= self.bar.x // ball right > bar left
            && self.ball.x <= self.bar.x + self.bar.width as i32
        // ball left < bar right
        {
            // Physics for bar collision - angle of bounce depends on where the ball hits the bar
            collision_occurred = true;

            // Calculate relative position of collision on bar (0.0 to 1.0)
            let hit_position = (self.ball.x + (self.ball.width as i32 / 2) - self.bar.x) as f32
                / self.bar.width as f32;

            // Clamp hit_position between 0.0 and 1.0 to prevent errors
            let hit_position = hit_position.clamp(0.0, 1.0);

            // Map hit position to an angle range: -60 degrees to +60 degrees
            // Convert to radians: -π/3 to +π/3
            let angle = (hit_position - 0.5) * std::f32::consts::PI / 1.5;

            // Set the velocity components based on the angle
            self.ball.velocity_x = angle.sin();
            self.ball.velocity_y = -0.8; // Fixed upward component for more predictable gameplay

            // Factor in paddle movement for more dynamic gameplay
            if self.bar.velocity_x != 0.0 {
                self.ball.velocity_x += (self.bar.velocity_x) * 0.15;
            }

            // Normalize velocity to maintain consistent speed
            self.ball.normalize_velocity();

            // Ensure ball doesn't get stuck in the bar
            self.ball.y = self.bar.y - self.ball.height as i32;

            if self.ball.first_bounce {
                self.ball.first_bounce = false;
            }
        }

        // Add some jitter on collision to make the game less predictable
        if collision_occurred {
            // Add tiny random variations to avoid repetitive patterns
            self.ball.velocity_x += rng.random_range(-5..5) as f32 / 100.0;
            self.ball.velocity_y += rng.random_range(-5..5) as f32 / 100.0;
            self.ball.normalize_velocity();
        }
    }

    fn update_bar_position(&mut self, dir: Direction) {
        self.bar.velocity_x = match dir {
            Direction::Right => 1.0,
            Direction::Left => -1.0,
        };

        self.bar.x += self.bar.velocity_x as i32 * BAR_SPEED;

        // check horizontal boundaries
        if self.bar.x <= 0 {
            self.bar.x = 0;
            self.bar.velocity_x = 1.0;
        } else if self.bar.x + self.bar.width as i32 >= self.dimensions.width as i32 {
            self.bar.x = self.dimensions.width as i32 - self.bar.width as i32;
            self.bar.velocity_x = -1.0;
        }
    }

    fn get_bricks(&mut self) {
        let col_count =
            self.dimensions.width as i32 / (BRICK_WIDTH as i32 + BRICK_PADDING as i32) - 5;

        let total_width =
            (BRICK_WIDTH as i32 * col_count) + (BRICK_PADDING as i32 * (col_count - 1));

        // starting  positions
        let start_x = (self.dimensions.width as i32 - total_width) / 2;
        let start_y = 50;

        for row in 0..ROW_COUNT {
            for col in 0..col_count {
                let brick_x = start_x + col * (BRICK_WIDTH as i32 + BRICK_PADDING as i32);
                let brick_y = start_y + row as i32 * (BRICK_HEIGHT as i32 + BRICK_PADDING as i32);
                self.bricks.push(Brick::new(brick_x, brick_y));
            }
        }
    }

    fn exit(&mut self) -> anyhow::Result<()> {
        self.canvas.set_draw_color(WINDOW_COLOR);
        self.canvas.clear();

        let text = format!(
            "GAME OVER! \nSCORE: {}\nPress Esc to Exit Game",
            self.state.score
        );

        let surface = self
            .font
            .render(&text)
            .blended(SCORE_COLOR)
            .map_err(|err| anyhow!("render game over message: {}", err))?;

        let tx_creator = self.canvas.texture_creator();
        let texture = tx_creator
            .create_texture_from_surface(&surface)
            .map_err(|err| anyhow!("create texture from surface: {}", err))?;
        let target = Rect::new(
            (self.dimensions.width - surface.width()) as i32 / 2,
            (self.dimensions.height - surface.height()) as i32 / 2,
            surface.width(),
            surface.height(),
        );
        self.canvas
            .copy(&texture, None, target)
            .map_err(|err| anyhow!("copy texture to game_over target: {}", err))?;

        self.canvas.present();

        'game_over: loop {
            for event in self.event_pump.poll_iter() {
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
        Ok(())
    }
}

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

struct Ball {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    velocity_x: f32,
    velocity_y: f32,
    first_bounce: bool,
}

impl Ball {
    fn new(bar: &Bar) -> Self {
        // the ball should start on top of the bar
        let mut rng = rng();
        let x = bar.x + (BAR_WIDTH as i32 / 2) + rng.random_range(0..100);
        let y = bar.y - (BAR_HEIGHT as i32);

        // let velocity_x = if rng.random_bool(0.5) { 1 } else { -1 };
        Self {
            x,
            y,
            width: BALL_WIDTH,
            height: BALL_HEIGHT,
            velocity_x: 0.0,
            velocity_y: -1.0,
            first_bounce: true,
        }
    }

    fn normalize_velocity(&mut self) {
        let speed =
            f32::sqrt(self.velocity_x * self.velocity_x + self.velocity_y * self.velocity_y);
        if speed != 0.0 {
            self.velocity_x /= speed;
            self.velocity_y /= speed;
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
    velocity_x: f32,
}

impl Bar {
    fn new(dimensions: &WinDimensions) -> Self {
        let x = (dimensions.width as i32 - BAR_WIDTH as i32) / 2; // center
        let y = dimensions.height as i32 - (dimensions.height as f32 * 0.10) as i32;

        Self {
            x,
            y,
            width: BAR_WIDTH,
            height: BAR_HEIGHT,
            velocity_x: 1.0,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut game = Game::new()?;
    let _ = game.play();

    let _ = game.exit();

    Ok(())
}
