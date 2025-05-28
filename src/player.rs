use bracket_lib::prelude::*;
use crate::{SCREEN_WIDTH};

pub struct Player {
    pub x: i32,
    pub y: i32,
    velocity: f32,
    speed: i32,
    // img: DynamicImage,
}

impl Player {
    pub fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
            speed: 1,
            // img: image::open("assets/player.png").unwrap(),
        }
    }
    pub fn render(&mut self, ctx: &mut BTerm) {
        // let (img_width, img_height) = self.img.dimensions();
        // for x in 0..img_width {
        //     for y in 0..img_height {
        //         let pixel = self.img.get_pixel(x, y);
        //         ctx.set_bg(0, self.y, (pixel[0], pixel[1], pixel[2]));
        //     }
        // }
          ctx.set(self.x, self.y, YELLOW, BLACK, to_cp437('🤣'))
    }
    pub fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2
        }
        self.y += self.velocity as i32;

        if self.y < 0 {
            self.y = 0;
        }
    }
     // 新增：水平移动方法
    pub fn move_left(&mut self) {
        self.x -= self.speed;
        self.x = self.x.max(0);  // 确保不超出左边界
    }

    pub fn move_right(&mut self) {
        self.x += self.speed;
        self.x = self.x.min(SCREEN_WIDTH/2);
        // 右边界检查在State中处理
    }

    pub fn flap(&mut self) {
        self.velocity = -2.0
    }
}
