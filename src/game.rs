use piston_window::types::Color;
use piston_window::*;
use std::collections::LinkedList;

use drawing::{draw_block, draw_rectangle};
use rand::{thread_rng, Rng};
use snake::{Direction, Snake};

const FOOD_COLOR: Color = [0.90, 0.49, 0.13, 1.0];
const BORDER_COLOR: Color = [0.741, 0.765, 0.78, 1.0];
const GAMEOVER_COLOR: Color = [0.91, 0.30, 0.24, 0.5];

const MOVING_PERIOD: f64 = 0.08; // in second
const RESTART_TIME: f64 = 1.0; // in second

#[derive(Debug, Clone)]
struct Obstacle {
    x: i32,
    y: i32,
}

pub struct Game {
    snake: Snake,

    // Food
    food_exist: bool,
    food_x: i32,
    food_y: i32,

    // Game Space
    width: i32,
    height: i32,

    // Game state
    is_game_over: bool,
    // When the game is running, it represents the waiting time from the previous moving
    // When the game is over, it represents the waiting time from the end of the game
    waiting_time: f64,
    obstacles: LinkedList<Obstacle>,
    attach_chance: f64,
}

impl Game {
    pub fn new(width: i32, height: i32) -> Game {
        Game {
            snake: Snake::new(2, 2),
            waiting_time: 0.0,
            food_exist: true,
            food_x: 5,
            food_y: 3,
            width,
            height,
            is_game_over: false,
            obstacles: LinkedList::new(),
            attach_chance: 0.60,
        }
    }

    pub fn key_pressed(&mut self, key: Key) {
        if self.is_game_over {
            return;
        }

        let dir: Option<Direction> = match key {
            Key::Up => Some(Direction::Up),
            Key::Down => Some(Direction::Down),
            Key::Left => Some(Direction::Left),
            Key::Right => Some(Direction::Right),
            // Ignore other keys
            _ => return,
        };

        if dir.unwrap() == self.snake.head_direction().opposite() {
            return;
        }

        // Check if the snake hits the border
        self.update_snake(dir);
    }

    pub fn draw(&self, con: &Context, g: &mut G2d) {
        self.snake.draw(con, g);

        // Draw the food
        if self.food_exist {
            draw_block(FOOD_COLOR, self.food_x, self.food_y, con, g);
        }
        // Draw obstacles
        for obstacle in &self.obstacles {
            draw_block([0.0, 0.0, 0.0, 1.0], obstacle.x, obstacle.y, con, g);
        }

        // Draw the border
        draw_rectangle(BORDER_COLOR, 0, 0, self.width, 1, con, g);
        draw_rectangle(BORDER_COLOR, 0, self.height - 1, self.width, 1, con, g);
        draw_rectangle(BORDER_COLOR, 0, 0, 1, self.height, con, g);
        draw_rectangle(BORDER_COLOR, self.width - 1, 0, 1, self.height, con, g);

        // Draw a game-over rectangle
        if self.is_game_over {
            draw_rectangle(GAMEOVER_COLOR, 0, 0, self.width, self.height, con, g);
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.waiting_time += delta_time;

        // If the game is over
        if self.is_game_over {
            if self.waiting_time > RESTART_TIME {
                self.restart();
            }
            return;
        }

        // Check if the food still exists
        if !self.food_exist {
            self.add_food();
        }

        // Move the snake
        if self.waiting_time > MOVING_PERIOD {
            self.update_snake(None);
        }
    }
    fn add_obstacle(&mut self) {
        let mut rng: rand::rngs::ThreadRng = thread_rng();

        let (mut new_x, mut new_y): (i32, i32);

        if rng.gen::<f64>() < self.attach_chance && !self.obstacles.is_empty() {
            // attach to existing obstacle
            let last_obstacle = self.obstacles.back().unwrap();
            // find an adjacent spot
            let offset = rng.gen_range(0..4);
            new_x = last_obstacle.x
                + match offset {
                    0 => -1,
                    1 => 1,
                    _ => 0,
                };
            new_y = last_obstacle.y
                + match offset {
                    2 => -1,
                    3 => 1,
                    _ => 0,
                };
        } else {
            // place randomly
            new_x = rng.gen_range(1..(self.width - 1));
            new_y = rng.gen_range(1..(self.height - 1));
        }

        while self.snake.is_overlap_except_tail(new_x, new_y)
            || (self.food_exist && self.food_x == new_x && self.food_y == new_y)
        {
            new_x = rng.gen_range(1..(self.width - 1));
            new_y = rng.gen_range(1..(self.height - 1));
        }

        self.obstacles.push_back(Obstacle { x: new_x, y: new_y });
    }

    fn check_eating(&mut self) {
        let (head_x, head_y): (i32, i32) = self.snake.head_position();
        if self.food_exist && self.food_x == head_x && self.food_y == head_y {
            self.food_exist = false;
            self.snake.restore_last_removed();
            self.add_obstacle();
            if self.attach_chance < 1.0 {
                self.attach_chance += 0.02;
            }
            if self.attach_chance > 0.99 {
                self.add_obstacle();
                self.add_obstacle();
            }
        }
    }

    fn check_if_the_snake_alive(&self, dir: Option<Direction>) -> bool {
        let (next_x, next_y) = self.snake.next_head_position(dir);

        // Check if the snake hits itself
        if self.snake.is_overlap_except_tail(next_x, next_y) {
            return false;
        }

        // Check if the snake hits an obstacle
        for obstacle in &self.obstacles {
            if next_x == obstacle.x && next_y == obstacle.y {
                return false;
            }
        }

        // Check if the snake overlaps with the border
        next_x > 0 && next_y > 0 && next_x < self.width - 1 && next_y < self.height - 1
    }

    fn add_food(&mut self) {
        let mut rng: rand::rngs::ThreadRng = thread_rng();

        // Decide the position of the new food
        let mut new_x: i32 = rng.gen_range(1..(self.width - 1));
        let mut new_y: i32 = rng.gen_range(1..(self.height - 1));
        while self.snake.is_overlap_except_tail(new_x, new_y) {
            new_x = rng.gen_range(1..(self.width - 1));
            new_y = rng.gen_range(1..(self.height - 1));
        }

        // Add the new food
        self.food_x = new_x;
        self.food_y = new_y;
        self.food_exist = true;
    }

    fn update_snake(&mut self, dir: Option<Direction>) {
        if self.check_if_the_snake_alive(dir) {
            self.snake.move_forward(dir);
            self.check_eating();
        } else {
            self.is_game_over = true;
        }
        self.waiting_time = 0.0;
    }

    fn restart(&mut self) {
        self.snake = Snake::new(2, 2);
        self.waiting_time = 0.0;
        self.food_exist = true;
        self.food_x = 5;
        self.food_y = 3;
        self.is_game_over = false;
        self.obstacles.clear();
        self.attach_chance = 0.60;
    }
}
