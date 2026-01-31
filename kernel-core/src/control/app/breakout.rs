use crate::control::app::{App, AppCommand};
use alloc::string::ToString;
use pc_keyboard::{DecodedKey, KeyCode};
use ratatui::Frame;
use ratatui::buffer::Buffer;

const BRICKS_X: usize = 8;
const BRICKS_Y: usize = 2;

const PADDLE_WIDTH: u16 = 12;

/// A simple breakout game.
#[derive(Clone)]
pub struct BreakoutApp {
    bricks: [[bool; BRICKS_X]; BRICKS_Y],
    ball: (i16, i16),
    ball_vel: (i16, i16),
    paddle_x: i16,
    width: i16,
    height: i16,
}

impl BreakoutApp {
    /// Create a new breakout app.
    pub fn new() -> Self {
        Self {
            bricks: [[true; BRICKS_X]; BRICKS_Y],
            ball: (10, 10),
            ball_vel: (1, -1),
            paddle_x: 10,
            width: 0,
            height: 0,
        }
    }

    fn set_cell(&self, buf: &mut Buffer, x: u16, y: u16, char: char) {
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_char(char);
        }
    }
}

impl App for BreakoutApp {
    fn render(&mut self, frame: &mut Frame) -> AppCommand {
        let area = frame.area();
        self.width = area.width as i16;
        self.height = area.height as i16;

        let buf = frame.buffer_mut();

        // Clear screen
        buf.reset();

        /* ---------- Draw bricks ---------- */
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

        /* ---------- Draw paddle ---------- */
        let paddle_y = self.height as u16 - 2;
        let max_paddle_x = self.width - PADDLE_WIDTH as i16;

        if self.paddle_x < 0 {
            self.paddle_x = 0;
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

        /* ---------- Draw ball ---------- */
        self.set_cell(buf, self.ball.0 as u16, self.ball.1 as u16, '@');

        /* ---------- Predict next position ---------- */
        let mut next_x = self.ball.0 + self.ball_vel.0;
        let mut next_y = self.ball.1 + self.ball_vel.1;

        /* ---------- Wall collision ---------- */
        if next_x < 0 {
            next_x = 0;
            self.ball_vel.0 = -self.ball_vel.0;
        }
        if next_x >= self.width {
            next_x = self.width - 1;
            self.ball_vel.0 = -self.ball_vel.0;
        }

        if next_y < 0 {
            next_y = 0;
            self.ball_vel.1 = -self.ball_vel.1;
        }

        /* ---------- Paddle collision ---------- */
        let paddle_y = self.height - 2;

        if self.ball_vel.1 > 0 // only when moving downward
            && self.ball.1 < paddle_y - 1
            && next_y >= paddle_y - 1
        {
            if next_x >= self.paddle_x && next_x < self.paddle_x + PADDLE_WIDTH as i16 {
                next_y = paddle_y - 1;
                self.ball_vel.1 = -self.ball_vel.1;

                // Optional: angle control based on hit position
                let hit_pos = next_x - self.paddle_x;
                let center = PADDLE_WIDTH as i16 / 2;

                let mut new_vx = (hit_pos - center) / 3;

                // If near center, keep previous horizontal direction
                if new_vx == 0 {
                    new_vx = if self.ball_vel.0 >= 0 { 1 } else { -1 };
                }

                self.ball_vel.0 = new_vx;
            }
        }

        /* ---------- Brick collision ---------- */
        let brick_height = 1;

        if next_y >= 0 && next_y < BRICKS_Y as i16 * brick_height {
            let bx = (next_x as usize * BRICKS_X) / self.width as usize;
            let by = (next_y as usize) / brick_height as usize;

            if bx < BRICKS_X && by < BRICKS_Y && self.bricks[by][bx] {
                self.bricks[by][bx] = false;
                self.ball_vel.1 = -self.ball_vel.1;
                next_y = self.ball.1 + self.ball_vel.1;
            }
        }

        /* ---------- Commit position ---------- */
        self.ball.0 = next_x;
        self.ball.1 = next_y;

        /* ---------- Lose condition ---------- */
        if self.ball.1 >= self.height {
            return AppCommand::Exit(Some("You lose!".to_string()));
        }

        /* ---------- Win condition ---------- */
        if self.bricks.iter().all(|row| row.iter().all(|&b| !b)) {
            return AppCommand::Exit(Some("You win!".to_string()));
        }

        AppCommand::Continue
    }

    fn handle_input(&mut self, key: DecodedKey) -> AppCommand {
        match key {
            DecodedKey::RawKey(KeyCode::ArrowLeft) => {
                self.paddle_x = self.paddle_x.saturating_sub(2)
            }

            DecodedKey::RawKey(KeyCode::ArrowRight) => self.paddle_x += 2,

            DecodedKey::Unicode('q') => return AppCommand::Exit(None),
            _ => {}
        }

        AppCommand::Continue
    }

    fn exit(&mut self) {}
}
