use crate::api;
use crate::control::app::{App, AppCommand};
use alloc::string::ToString;
use pc_keyboard::{DecodedKey, KeyCode};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use time::OffsetDateTime;

const BRICKS_X: usize = 8;
const BRICKS_Y: usize = 2;
const PADDLE_WIDTH: u16 = 12;

const FIXED_STEP_MS: i64 = 8;

/// A simple breakout game.
#[derive(Clone)]
pub struct BreakoutApp {
    bricks: [[bool; BRICKS_X]; BRICKS_Y],
    ball: (f32, f32),
    prev_ball: (f32, f32),
    ball_vel: (f32, f32),
    paddle_x: f32,
    width: i16,
    height: i16,
    last_update: Option<OffsetDateTime>,
    accumulator_ms: i64,
}

impl BreakoutApp {
    /// Creates a new instance of the breakout game.
    pub fn new() -> Self {
        Self {
            bricks: [[true; BRICKS_X]; BRICKS_Y],
            ball: (10.0, 10.0),
            prev_ball: (10.0, 10.0),
            ball_vel: (0.20, -0.12),
            paddle_x: 10.0,
            width: 0,
            height: 0,
            last_update: None,
            accumulator_ms: 0,
        }
    }

    fn set_cell(&self, buf: &mut Buffer, x: u16, y: u16, char: char) {
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_char(char);
        }
    }

    fn update_physics(&mut self) -> Option<AppCommand> {
        if self.width == 0 || self.height == 0 {
            return None;
        }

        self.prev_ball = self.ball;

        let mut next_x = self.ball.0 + self.ball_vel.0;
        let mut next_y = self.ball.1 + self.ball_vel.1;

        /* ---------- Wall collisions ---------- */
        if next_x < 0.0 {
            next_x = 0.0;
            self.ball_vel.0 = -self.ball_vel.0;
        }
        if next_x >= self.width as f32 {
            next_x = self.width as f32 - 1.0;
            self.ball_vel.0 = -self.ball_vel.0;
        }
        if next_y < 0.0 {
            next_y = 0.0;
            self.ball_vel.1 = -self.ball_vel.1;
        }

        /* ---------- Paddle collision ---------- */
        let paddle_y = self.height as f32 - 2.0;

        if self.ball_vel.1 > 0.0
            && self.ball.1 < paddle_y - 0.5
            && next_y >= paddle_y - 0.5
            && next_x >= self.paddle_x - 0.5
            && next_x < self.paddle_x + PADDLE_WIDTH as f32 + 0.5
        {
            next_y = paddle_y - 0.5;
            self.ball_vel.1 = -self.ball_vel.1;

            let hit_pos = next_x - self.paddle_x;
            let center = PADDLE_WIDTH as f32 / 2.0;
            let offset = (hit_pos - center) / center;

            self.ball_vel.0 = offset * 0.35;
            if self.ball_vel.0.abs() < 0.08 {
                self.ball_vel.0 = if offset >= 0.0 { 0.12 } else { -0.12 };
            }
        }

        /* ---------- Brick collision ---------- */
        let brick_height = 1.0;
        if next_y >= 0.0 && next_y < BRICKS_Y as f32 * brick_height {
            let bx = ((next_x / self.width as f32) * BRICKS_X as f32) as usize;
            let by = (next_y / brick_height) as usize;

            if bx < BRICKS_X && by < BRICKS_Y && self.bricks[by][bx] {
                self.bricks[by][bx] = false;
                self.ball_vel.1 = -self.ball_vel.1;
                next_y = self.ball.1 + self.ball_vel.1;
            }
        }

        self.ball.0 = next_x;
        self.ball.1 = next_y;

        /* ---------- Win/Lose ---------- */
        if self.ball.1 >= self.height as f32 {
            return Some(AppCommand::Exit(Some("You lose!".to_string())));
        }

        if self.bricks.iter().all(|row| row.iter().all(|&b| !b)) {
            return Some(AppCommand::Exit(Some("You win!".to_string())));
        }

        None
    }
}

impl App for BreakoutApp {
    fn render(&mut self, frame: &mut Frame) -> AppCommand {
        let area = frame.area();
        self.width = area.width as i16;
        self.height = area.height as i16;

        let now = api::time().read_local();

        if let Some(last) = self.last_update {
            let delta_ms = (now.unix_timestamp() - last.unix_timestamp()) * 1000
                + (now.millisecond() as i64 - last.millisecond() as i64);

            // Cap maximum step to avoid spiral of death on long frames
            let delta_ms = delta_ms.min(100);
            self.accumulator_ms += delta_ms;

            while self.accumulator_ms >= FIXED_STEP_MS {
                if let Some(cmd) = self.update_physics() {
                    return cmd;
                }
                self.accumulator_ms -= FIXED_STEP_MS;
            }
        }
        self.last_update = Some(now);

        // Sub-step Interpolation Factor
        let alpha = (self.accumulator_ms as f32) / (FIXED_STEP_MS as f32);
        let render_ball_x = self.prev_ball.0 + (self.ball.0 - self.prev_ball.0) * alpha;
        let render_ball_y = self.prev_ball.1 + (self.ball.1 - self.prev_ball.1) * alpha;

        let buf = frame.buffer_mut();
        buf.reset();

        // Draw Bricks
        let brick_width = area.width / BRICKS_X as u16;
        let brick_height = 1;

        for by in 0..BRICKS_Y {
            for bx in 0..BRICKS_X {
                if !self.bricks[by][bx] {
                    continue;
                }
                let x0 = bx as u16 * brick_width;
                let y0 = by as u16 * brick_height;

                for dy in 0..brick_height {
                    for dx in 0..brick_width {
                        let x = x0 + dx;
                        let y = y0 + dy;
                        if x < area.width && y < area.height {
                            self.set_cell(buf, x, y, '#');
                        }
                    }
                }
            }
        }

        // Clamp & Draw Paddle
        let paddle_y = self.height as u16 - 2;
        let max_paddle_x = self.width as f32 - PADDLE_WIDTH as f32;

        if self.paddle_x < 0.0 {
            self.paddle_x = 0.0;
        }
        if self.paddle_x > max_paddle_x {
            self.paddle_x = max_paddle_x;
        }

        for i in 0..PADDLE_WIDTH {
            let x = (self.paddle_x as u16).saturating_add(i);
            if x < area.width {
                self.set_cell(buf, x, paddle_y, '=');
            }
        }

        // Draw Interpolated Ball Position
        let ball_draw_x = (render_ball_x as u16).min(area.width.saturating_sub(1));
        let ball_draw_y = (render_ball_y as u16).min(area.height.saturating_sub(1));
        self.set_cell(buf, ball_draw_x, ball_draw_y, '@');

        AppCommand::Continue
    }

    fn handle_input(&mut self, key: DecodedKey) -> AppCommand {
        match key {
            DecodedKey::RawKey(KeyCode::ArrowLeft) => {
                self.paddle_x = (self.paddle_x - 2.0).max(0.0);
            }
            DecodedKey::RawKey(KeyCode::ArrowRight) => {
                self.paddle_x = (self.paddle_x + 2.0).min(self.width as f32 - PADDLE_WIDTH as f32);
            }
            DecodedKey::Unicode('q') => return AppCommand::Exit(None),
            _ => {}
        }

        AppCommand::Continue
    }

    fn exit(&mut self) {}
}
