use macroquad::prelude::*;

#[derive(Clone, Copy)]
struct Unit {
    position: Vec2,
}

impl Unit {
    fn go(&mut self, prev_unit_pos: Vec2) {
        let to_prev = prev_unit_pos - self.position;
        let distance = to_prev.length();
        let shift = distance - 2.0 * UNIT_RADIUS;
        if shift > 0.0 {
            self.position += to_prev.normalize() * shift;
        }
    }

    fn draw(&self) {
        let ppm = pixels_per_meter();
        let radius_pixels = UNIT_RADIUS * ppm;
        let screen_pos = to_screen_coords(self.position);
        draw_circle(screen_pos.x, screen_pos.y, radius_pixels, WHITE);
    }

    fn intersect(&self, position: Vec2, radius: f32) -> bool {
        self.position.distance(position) < radius + UNIT_RADIUS
    }
}

struct Fruit {
    position: Vec2,
}

impl Fruit {
    fn respawn() -> Self {
        Self {
            position: random_position(),
        }
    }

    fn draw(&self) {
        let ppm = pixels_per_meter();
        let radius_pixels = FRUIT_RADIUS * ppm;
        let screen_pos = to_screen_coords(self.position);
        draw_circle(screen_pos.x, screen_pos.y, radius_pixels, RED);
    }
}

struct Head {
    unit: Unit,
    direction: Vec2,
    speed: f32,
}

impl Head {
    pub fn rotate(&mut self, angle: f32) {
        let rotation = Vec2::from_angle(angle);
        let new_head_direction = rotation.rotate(self.direction);
        self.direction = new_head_direction;
    }

    pub fn go(&mut self, dt: f32) {
        self.unit.position += self.speed * dt * self.direction;
    }

    fn position(&self) -> Vec2 {
        self.unit.position
    }

    fn draw(&self) {
        self.unit.draw();

        let angle = 0.3;
        let left_eye_shift = Vec2::from_angle(angle).rotate(self.direction) * UNIT_RADIUS;
        let left_eye_pos = to_screen_coords(self.position() + left_eye_shift);
        let right_eye_shift = Vec2::from_angle(-angle).rotate(self.direction) * UNIT_RADIUS;
        let right_eye_pos = to_screen_coords(self.position() + right_eye_shift);
        let eye_r = UNIT_RADIUS / 6.0 * pixels_per_meter();

        draw_circle(left_eye_pos.x, left_eye_pos.y, eye_r, BLACK);
        draw_circle(right_eye_pos.x, right_eye_pos.y, eye_r, BLACK);
    }

    pub fn intersect(&self, position: Vec2, radius: f32) -> bool {
        self.unit.intersect(position, radius)
    }
}

struct Snake {
    head: Head,
    units: Vec<Unit>,
}

impl Snake {
    pub fn go(&mut self, dt: f32, rotation: f32) {
        let angle = rotation * dt;
        self.head.rotate(angle);
        self.head.go(dt);

        let mut prev_unit_pos = self.head.position();
        for unit in &mut self.units {
            unit.go(prev_unit_pos);
            prev_unit_pos = unit.position;
        }
    }

    pub fn draw(&self) {
        self.head.draw();
        for unit in &self.units {
            unit.draw();
        }
    }

    pub fn can_eat(&self, fruit: &Fruit) -> bool {
        self.head.intersect(fruit.position, FRUIT_RADIUS)
    }

    pub fn is_lose(&self) -> bool {
        let intersect_unit = self
            .units
            .iter()
            .skip(1)
            .any(|u| self.head.intersect(u.position, UNIT_RADIUS * 0.8));

        let max_coord = FIELD_SIZE / 2.0 - UNIT_RADIUS;
        let intersect_wall =
            self.head.position().x.abs() > max_coord || self.head.position().y.abs() > max_coord;

        intersect_unit || intersect_wall
    }

    pub fn add_unit(&mut self) {
        let last_unit = self.units.last().unwrap_or(&self.head.unit);
        self.units.push(*last_unit);
    }

    pub fn length(&self) -> u32 {
        (self.units.len() + 1) as _
    }
}

impl Default for Snake {
    fn default() -> Self {
        let head_unit = Unit {
            position: Vec2::ZERO,
        };

        let head = Head {
            unit: head_unit,
            direction: Vec2::X,
            speed: INIT_SPEED,
        };

        Self {
            head,
            units: vec![],
        }
    }
}

const INIT_SPEED: f32 = 0.4;
const UNIT_RADIUS: f32 = 0.04;
const FRUIT_RADIUS: f32 = 0.06;
const ROTATION_PER_SEC_RAD: f32 = 2.0;
const FIELD_SIZE: f32 = 2.0;

fn pixels_per_meter() -> f32 {
    screen_width().min(screen_height()) / 2.0
}

fn to_screen_coords(pos: Vec2) -> Vec2 {
    let min_dim = screen_width().min(screen_height());
    let width_offset = (screen_width() - min_dim) / 2.0;
    let height_offset = (screen_height() - min_dim) / 2.0;
    let offset = Vec2::new(width_offset, height_offset);

    let shift = Vec2::new(1.0, -1.0);
    let scale = Vec2::new(1.0, -1.0) * pixels_per_meter();
    (pos + shift) * scale + offset
}

fn draw_field() {
    let top_left = to_screen_coords(Vec2::new(-1.0, 1.0));
    let size = pixels_per_meter() * FIELD_SIZE * Vec2::ONE;
    draw_rectangle(top_left.x, top_left.y, size.x, size.y, GREEN);
}

fn rand_f32() -> f32 {
    (rand::rand() as f64 / u32::MAX as f64) as f32
}

fn random_position() -> Vec2 {
    Vec2::new(rand_f32() * 2.0 - 1.0, rand_f32() * 2.0 - 1.0)
}

#[macroquad::main("Snake")]
async fn main() {
    let mut snake = Snake::default();
    let mut fruit = Fruit::respawn();
    loop {
        clear_background(LIGHTGRAY);

        let dt = get_frame_time();
        let mut rotation = 0.0;

        if is_key_down(KeyCode::Left) {
            rotation = ROTATION_PER_SEC_RAD;
        }
        if is_key_down(KeyCode::Right) {
            rotation = -ROTATION_PER_SEC_RAD;
        }

        if snake.can_eat(&fruit) {
            fruit = Fruit::respawn();
            snake.add_unit();
        }

        if snake.is_lose() {
            snake = Snake::default();
            fruit = Fruit::respawn();
        }

        draw_field();
        snake.go(dt, rotation);
        snake.draw();
        fruit.draw();

        let scores_text = format!("scores: {}", snake.length() - 1);
        draw_text(&scores_text, 10.0, 10.0, 24.0, BLACK);

        next_frame().await
    }
}
