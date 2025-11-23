use std::time::Instant;

use raylib::prelude::*;

const W: i32 = 1024;
const H: i32 = 768;
const PAD_W: i32 = 60;
const PAD_H: i32 = 10;
const BALL_R: f32 = 10.0;
const BRICK_W: i32 = 40;
const BRICK_H: i32 = 20;

struct Brick {
    pos: Vector2,
    size: Vector2,
    hp: i32,
}

struct Pad {
    pos: Vector2,
    size: Vector2,
}

struct Ball {
    pos: Vector2,
    speed: Vector2,
}

struct Game {
    pad: Pad,
    bricks: Vec<Brick>,
    ball: Ball,
    last_update: Instant,
}

/*
impl From<(f32, f32)> for Vector2 {
    fn from(v: (f32, f32)) -> Self {
        Self { x: v.0, y: v.1 }
    }
}

impl From<(i32, i32)> for Vector2 {
    fn from(v: (i32, i32)) -> Self {
        Self {
            x: v.0 as f32,
            y: v.1 as f32,
        }
    }
}

impl std::ops::AddAssign<Vector2> for Vector2 {
    fn add_assign(&mut self, rhs: Vector2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
*/

impl Brick {
    fn new(x: f32, y: f32, hp: i32) -> Self {
        Self {
            pos: Vector2 { x, y },
            size: Vector2 {
                x: BRICK_W as f32,
                y: BRICK_H as f32,
            },
            hp,
        }
    }
}

impl Ball {
    fn collides(&self, rect: Vector2, size: Vector2) -> Option<Vector2> {
        let (left, right, bottom, top) = (rect.x, rect.x + size.x, rect.y, rect.y + size.y);

        let closest_x = self.pos.x.clamp(left, right);
        let closest_y = self.pos.y.clamp(bottom, top);

        let dx = self.pos.x - closest_x;
        let dy = self.pos.y - closest_y;
        if dx * dx + dy * dy > BALL_R * BALL_R {
            return None;
        }

        if closest_y == self.pos.y {
            // hit horizontal side
            if self.pos.x < left {
                return Some(Vector2::new(-1.0, 0.0));
            } else {
                return Some(Vector2::new(1.0, 0.0));
            }
        }
        if closest_x == self.pos.x {
            // hit vertical side
            if self.pos.y < bottom {
                return Some(Vector2::new(0.0, -1.0));
            } else {
                return Some(Vector2::new(0.0, 1.0));
            }
        }

        None
    }
}

impl Game {
    fn render(&self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        rl.draw(&thread, |mut d| {
            d.clear_background(Color::BLACK);
            let p = &self.pad;

            d.draw_rectangle(
                p.pos.x as i32,
                p.pos.y as i32,
                p.size.x as i32,
                p.size.y as i32,
                Color::LIGHTGREEN,
            );

            for b in &self.bricks {
                if b.hp < 1 {
                    continue;
                }
                d.draw_rectangle(
                    b.pos.x as i32,
                    b.pos.y as i32,
                    b.size.x as i32,
                    b.size.y as i32,
                    Color::RED,
                );
                d.draw_rectangle_lines(
                    b.pos.x as i32,
                    b.pos.y as i32,
                    b.size.x as i32,
                    b.size.y as i32,
                    Color::DARKRED,
                );
                let txt = format!("{}", b.hp);

                d.draw_text(
                    &txt,
                    (b.pos.x) as i32 + b.size.x as i32 / 2,
                    (b.pos.y + b.size.y * 0.2) as i32,
                    (b.size.y * 0.75) as i32,
                    Color::YELLOW,
                );
            }

            d.draw_circle(
                self.ball.pos.x as i32,
                self.ball.pos.y as i32,
                BALL_R,
                Color::LIGHTBLUE,
            );
        });
    }

    fn update(&mut self, rl: &RaylibHandle) {
        self.last_update = Instant::now();

        // moving pad
        self.pad.pos.x += rl.get_mouse_delta().x / 2.0;
        if self.pad.pos.x >= (W - PAD_W) as f32 {
            self.pad.pos.x = (W - PAD_W) as f32;
        }
        if self.pad.pos.x <= 0.0 {
            self.pad.pos.x = 0.0
        }

        // ball collisions
        self.ball.pos += self.ball.speed;
        if self.ball.pos.x <= BALL_R || self.ball.pos.x >= W as f32 - BALL_R {
            self.ball.speed.x *= -1.0;
        }
        if self.ball.pos.y <= BALL_R || self.ball.pos.y >= H as f32 - BALL_R {
            self.ball.speed.y *= -1.0;
        }

        if let Some(n) = self.ball.collides(self.pad.pos, self.pad.size) {
            self.ball.speed = reflect(self.ball.speed, n);
        }

        for b in &mut self.bricks {
            if b.hp < 1 {
                continue;
            }
            if let Some(n) = self.ball.collides(b.pos, b.size) {
                self.ball.speed = reflect(self.ball.speed, n);
                b.hp -= 1;
            }
        }
    }
}

fn reflect(v: Vector2, n: Vector2) -> Vector2 {
    let d = v.dot(n);
    Vector2 {
        x: v.x - 2.0 * n.x * d,
        y: v.y - 2.0 * n.y * d,
    }
}

fn main() {
    let (mut rl, thread) = raylib::init().size(W, H).title("Arkanoid").build();
    let mut game = Game {
        last_update: Instant::now(),
        pad: Pad {
            pos: Vector2::new((W / 2) as f32, (H - PAD_H) as f32),
            size: Vector2::new(PAD_W as f32, PAD_H as f32),
        },
        bricks: Vec::new(),
        ball: Ball {
            pos: Vector2::new(
                (W / 2 + PAD_W / 2) as f32,
                1.0 + (H - PAD_H) as f32 - BALL_R,
            ),
            speed: Vector2::new(8.0, -5.0),
        },
    };
    for col in 0..W / BRICK_W {
        for row in 0..H / BRICK_H / 2 {
            game.bricks.push(Brick {
                pos: Vector2::new((col * BRICK_W) as f32, (row * BRICK_H) as f32),
                size: Vector2::new(BRICK_W as f32, BRICK_H as f32),
                hp: 1,
            });
        }
    }

    rl.set_target_fps(120);
    rl.gui_lock();
    rl.disable_cursor();
    while !rl.window_should_close() {
        game.update(&rl);
        game.render(&mut rl, &thread);
    }
    rl.gui_unlock();
    rl.enable_cursor();
}
