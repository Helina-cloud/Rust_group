use bracket_lib::prelude::*;
use crate::{SCREEN_WIDTH, difficulty::DifficultySettings};

pub struct Player {
    pub x: i32,
    pub y: i32,
    velocity: f32,
    speed: i32,
    animation_frame: i32,
    trail_positions: Vec<(i32, i32)>, // 飞行轨迹
}

impl Player {
    pub fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
            speed: 1,
            animation_frame: 0,
            trail_positions: Vec::new(),
        }
    }

    pub fn render(&mut self, ctx: &mut BTerm) {
        // 更新动画帧
        self.animation_frame = (self.animation_frame + 1) % 60;
        
        // 渲染飞行轨迹
        self.render_trail(ctx);
        
        // 选择玩家外观（简单的动画效果）
        let player_char = if self.velocity < -1.0 {
            '^' // 向上飞行
        } else if self.velocity > 1.0 {
            'v' // 向下坠落
        } else {
            '>' // 水平飞行
        };
        
        // 根据速度改变颜色
        let color = if self.velocity.abs() > 1.5 {
            ORANGE // 快速移动时橙色
        } else {
            YELLOW // 正常时黄色
        };
        
        ctx.set(self.x, self.y, color, BLACK, to_cp437(player_char));
        
        // 添加发光效果
        if self.animation_frame % 10 < 5 {
            ctx.set(self.x - 1, self.y, color, BLACK, to_cp437('.'));
        }
        
        // 更新轨迹
        self.update_trail();
    }

    fn render_trail(&self, ctx: &mut BTerm) {
        for (i, &(trail_x, trail_y)) in self.trail_positions.iter().enumerate() {
            let alpha = ((self.trail_positions.len() - i) as f32 / self.trail_positions.len() as f32 * 3.0) as u8;
            let trail_color = (alpha, alpha, 0); // 渐变的黄色轨迹
            if trail_x >= 0 && trail_x < SCREEN_WIDTH && trail_y >= 0 && trail_y < 50 {
                ctx.set(trail_x, trail_y, trail_color, BLACK, to_cp437('·'));
            }
        }
    }

    fn update_trail(&mut self) {
        // 添加当前位置到轨迹
        self.trail_positions.push((self.x, self.y));
        
        // 限制轨迹长度
        if self.trail_positions.len() > 8 {
            self.trail_positions.remove(0);
        }
    }

    pub fn gravity_and_move(&mut self) {
        self.apply_gravity();
        self.y += self.velocity as i32;
        self.clamp_position();
    }

    pub fn gravity_and_move_with_difficulty(&mut self, difficulty_settings: &DifficultySettings) {
        // 应用难度调整的重力
        if self.velocity < 1.5 {
            self.velocity += 0.3 * difficulty_settings.gravity_multiplier;
        }
        self.y += self.velocity as i32;
        self.clamp_position();
    }

    fn apply_gravity(&mut self) {
        if self.velocity < 1.5 {
            self.velocity += 0.3;
        }
    }

    fn clamp_position(&mut self) {
        if self.y < 0 {
            self.y = 0;
            self.velocity = 0.0;
        }
    }

    pub fn move_left(&mut self) {
        self.x -= self.speed;
        self.x = self.x.max(0);
    }

    pub fn move_right(&mut self) {
        self.x += self.speed;
        self.x = self.x.min(SCREEN_WIDTH / 2);
    }

    pub fn flap(&mut self) {
        self.velocity = -2.0;
        // 清空轨迹以创建跳跃效果
        self.trail_positions.clear();
    }

    pub fn flap_with_difficulty(&mut self, difficulty_settings: &DifficultySettings) {
        // 根据难度调整跳跃力度
        let flap_strength = -2.0 / difficulty_settings.gravity_multiplier;
        self.velocity = flap_strength;
        self.trail_positions.clear();
    }

    pub fn move_down(&mut self) {
        if self.velocity < 1.5 {
            self.velocity += 0.5;
        }
        self.y += self.velocity as i32;
        self.clamp_position();
    }

    /// 应用慢动作效果
    pub fn apply_slow_motion(&mut self, factor: f32) {
        self.velocity *= factor;
    }

    /// 重置玩家状态（用于复活或重新开始）
    pub fn reset(&mut self) {
        self.velocity = 0.0;
        self.trail_positions.clear();
        self.animation_frame = 0;
    }

    /// 获取玩家当前状态信息
    pub fn get_status(&self) -> PlayerStatus {
        PlayerStatus {
            velocity: self.velocity,
            is_ascending: self.velocity < -0.5,
            is_descending: self.velocity > 0.5,
            is_stable: self.velocity.abs() <= 0.5,
        }
    }
}

pub struct PlayerStatus {
    pub velocity: f32,
    pub is_ascending: bool,
    pub is_descending: bool,
    pub is_stable: bool,
}