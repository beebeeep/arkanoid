use rand::prelude::*;
use std::{f32, time::Instant};

// use delaunator::{Point, triangulate};
use raylib::prelude::*;
use voronoice::{BoundingBox, Point, VoronoiBuilder};

const W: i32 = 1024;
const H: i32 = 768;
const PAD_W: i32 = 60;
const PAD_H: i32 = 10;
const BALL_R: f32 = 10.0;

struct Shard {
    edges: Vec<Vector2>,
    center: Vector2,
    hp: i32,
    id: usize,
}

struct Pad {
    poly: Vec<Vector2>,
}

struct Ball {
    pos: Vector2,
    radius: f32,
    speed: Vector2,
}

struct Game {
    pad: Pad,
    shards: Vec<Shard>,
    ball: Ball,
    last_update: Instant,
}

impl Shard {
    fn render(&self, dh: &mut RaylibDrawHandle) {
        let color = match self.hp {
            1 => Color::DARKRED,
            2 => Color::ORANGERED,
            3 => Color::ORANGE,
            _ => Color::DARKORANGE,
        };

        dh.draw_triangle_fan(&self.edges, color);
        for i in 0..self.edges.len() {
            // outline
            dh.draw_line_v(
                self.edges[i],
                self.edges[(i + 1) % self.edges.len()],
                Color::WHITE,
            );
        }
    }
}

impl Pad {
    fn render(&self, dh: &mut RaylibDrawHandle) {
        dh.draw_triangle_fan(&self.poly, Color::GREEN);
        for i in 0..self.poly.len() {
            // outline
            dh.draw_line_v(
                self.poly[i],
                self.poly[(i + 1) % self.poly.len()],
                Color::WHITE,
            );
        }
    }

    fn translate(&mut self, delta: &Vector2) {
        if delta.x < 0.0 && self.poly[0].x <= 0.0 {
            // left border
            return;
        }
        if delta.x > 0.0 && self.poly[1].x >= W as f32 {
            // right border
            return;
        }
        for e in &mut self.poly {
            *e += *delta;
        }
    }
}

impl Ball {
    fn collides(&self, poly: &[Vector2]) -> Option<Vector2> {
        if poly.len() < 2 {
            return None;
        }

        let mut best_pen = -f32::INFINITY;
        let mut best_n = Vector2::default();
        for i in 0..poly.len() {
            let a = poly[i];
            let b = poly[(i + 1) % poly.len()];
            let cp = self.closest_point(a, b);
            let delta = self.pos - cp;
            let dist = delta.length();
            if dist > self.radius {
                continue;
            }

            let pen = self.radius - dist;
            if pen <= best_pen {
                continue;
            }
            best_pen = pen;
            best_n = if dist > 0.0 {
                delta * (1.0 / dist)
            } else {
                // exactly on edge
                Vector2::new(1.0, 0.0)
            }
        }

        if best_pen > 0.0 {
            Some(best_n.normalized())
        } else {
            None
        }
    }

    fn closest_point(&self, a: Vector2, b: Vector2) -> Vector2 {
        let d = b - a;
        let t = (self.pos - a).dot(d) / d.dot(d);
        a + d * t.clamp(0.0, 1.0)
    }
}

impl Game {
    fn render(&self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);

        self.pad.render(&mut d);

        for s in &self.shards {
            if s.hp < 1 {
                continue;
            }
            // d.draw_triangle_fan(&s.edges, Color::PINK);
            // d.draw_triangle_strip(&s.edges, Color::HOTPINK);
            // d.draw_line_strip(&s.edges, Color::PINK);
            s.render(&mut d);
            d.draw_text(
                &format!("{}", s.id),
                s.center.x as i32,
                s.center.y as i32,
                10,
                Color::PINK,
            );
        }

        d.draw_circle(
            self.ball.pos.x as i32,
            self.ball.pos.y as i32,
            self.ball.radius,
            Color::LIGHTBLUE,
        );
    }

    fn update(&mut self, rl: &RaylibHandle) {
        self.last_update = Instant::now();

        // moving pad
        self.pad
            .translate(&Vector2::new(rl.get_mouse_delta().x / 2.0, 0.0));

        // ball collisions
        self.ball.pos += self.ball.speed;
        if self.ball.pos.x <= self.ball.radius || self.ball.pos.x >= W as f32 - self.ball.radius {
            self.ball.speed.x *= -1.0;
        }
        if self.ball.pos.y <= self.ball.radius || self.ball.pos.y >= H as f32 - self.ball.radius {
            self.ball.speed.y *= -1.0;
        }

        if let Some(n) = self.ball.collides(&self.pad.poly) {
            self.ball.speed = reflect(self.ball.speed, n);
        }

        for s in &mut self.shards {
            if s.hp < 1 {
                continue;
            }
            if let Some(n) = self.ball.collides(&s.edges) {
                eprintln!("hit shard {}", s.id);
                self.ball.speed = reflect(self.ball.speed, n);
                s.hp -= 1;
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
    let mut rng = rand::rng();
    let mut points = Vec::with_capacity(100);
    for _ in 0..points.capacity() {
        points.push(Point {
            x: rng.random_range(0.0..W as f64),
            y: rng.random_range(0.0..(H / 3) as f64),
        });
    }
    let voronoi = VoronoiBuilder::default()
        .set_sites(points)
        .set_bounding_box(BoundingBox::new(
            Point {
                x: (W / 2) as f64,
                y: (H / 6) as f64,
            },
            W as f64,
            (H / 3) as f64,
        ))
        .set_lloyd_relaxation_iterations(5)
        .build()
        .expect("building shards");
    let shards: Vec<_> = voronoi
        .iter_cells()
        .enumerate()
        .map(|(i, c)| Shard {
            center: Vector2 {
                x: c.site_position().x as f32,
                y: c.site_position().y as f32,
            },
            edges: c
                .iter_vertices()
                .map(|v| Vector2::new(v.x as f32, v.y as f32))
                .collect(),
            hp: rng.random_range(1..5),
            id: i,
        })
        .collect();

    let (mut rl, thread) = raylib::init().size(W, H).title("Arkanoid").build();
    let mut game = Game {
        last_update: Instant::now(),
        pad: Pad {
            // vertexes shall go counter-clockwise:
            //  3 +--------+ 2
            //    |        |
            //  0 +--------+ 1
            poly: vec![
                Vector2::new((W / 2) as f32, (H) as f32),
                Vector2::new((W / 2 + PAD_W) as f32, (H) as f32),
                Vector2::new((W / 2 + PAD_W) as f32, (H - PAD_H) as f32),
                Vector2::new((W / 2) as f32, (H - PAD_H) as f32),
            ],
        },
        shards,
        ball: Ball {
            pos: Vector2::new((W / 2 + PAD_W / 2) as f32, (H - PAD_H - 1) as f32 - BALL_R),
            radius: BALL_R,
            speed: Vector2::new(rng.random_range(-10.0..10.0), rng.random_range(-10.0..0.0)),
        },
    };

    rl.set_target_fps(60);
    rl.gui_lock();
    rl.disable_cursor();
    while !rl.window_should_close() {
        game.update(&rl);
        game.render(&mut rl, &thread);
    }
    rl.gui_unlock();
    rl.enable_cursor();
}
