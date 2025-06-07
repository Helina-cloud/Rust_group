use bracket_lib::prelude::*;
use crate::{player::Player, difficulty::DifficultySettings, SCREEN_HEIGHT};

pub struct Obstacle {
    pub x: i32,
    pub gap_y: i32,
    pub size: i32,
    pub speed: f32,
    pub obstacle_type: ObstacleType,
}

#[derive(Clone, Debug)]
pub enum ObstacleType {
    Static,    // 静态障碍物
    Moving,    // 上下移动的障碍物
    Rotating,  // 旋转障碍物（视觉效果）
}

impl Obstacle {
    pub fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        
        // 根据分数决定障碍物类型
        let obstacle_type = if score > 20 && random.range(0, 100) < 30 {
            if random.range(0, 2) == 0 {
                ObstacleType::Moving
            } else {
                ObstacleType::Rotating
            }
        } else {
            ObstacleType::Static
        };

        Obstacle {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(4, 20 - score), // 基础大小，随分数减小
            speed: 1.0,
            obstacle_type,
        }
    }

    pub fn render(&mut self, ctx: &mut BTerm, player_x: i32, difficulty_settings: &DifficultySettings) {
        // 应用难度设置
        let dynamic_speed = difficulty_settings.get_dynamic_speed(0) as i32;
        let actual_size = difficulty_settings.get_dynamic_gap_size(self.size, 0);
        
        // 根据难度调整移动速度
        self.x -= dynamic_speed.max(1);
        
        // 处理移动障碍物
        self.update_position();
        
        let half_size = actual_size / 2;
        
        // 选择渲染样式
        let (symbol, color) = self.get_obstacle_appearance();
        
        // 渲染上半部分障碍物
        for y in 0..self.gap_y - half_size {
            ctx.set(self.x, y, color, BLACK, to_cp437(symbol));
        }
        
        // 渲染下半部分障碍物
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(self.x, y, color, BLACK, to_cp437(symbol));
        }
        
        // 为移动障碍物添加视觉指示
        if matches!(self.obstacle_type, ObstacleType::Moving) {
            ctx.set(self.x, self.gap_y - half_size - 1, YELLOW, BLACK, to_cp437('↕'));
        }
    }

    fn update_position(&mut self) {
        match self.obstacle_type {
            ObstacleType::Moving => {
                // 上下移动逻辑
                static mut MOVE_TIMER: f32 = 0.0;
                static mut MOVE_DIRECTION: i32 = 1;
                
                unsafe {
                    MOVE_TIMER += 1.0;
                    if MOVE_TIMER > 20.0 {
                        self.gap_y += MOVE_DIRECTION;
                        MOVE_TIMER = 0.0;
                        
                        // 边界检查
                        if self.gap_y <= 15 {
                            MOVE_DIRECTION = 1;
                        } else if self.gap_y >= 35 {
                            MOVE_DIRECTION = -1;
                        }
                    }
                }
            }
            ObstacleType::Rotating => {
                // 旋转只是视觉效果，不改变实际碰撞
            }
            ObstacleType::Static => {
                // 静态障碍物不移动
            }
        }
    }

    fn get_obstacle_appearance(&self) -> (char, (u8, u8, u8)) {
        match self.obstacle_type {
            ObstacleType::Static => ('|', RED),
            ObstacleType::Moving => ('║', ORANGE),
            ObstacleType::Rotating => {
                // 简单的旋转效果
                let rotation_chars = ['|', '/', '-', '\\'];
                let char_index = ((self.x / 3) % 4) as usize;
                (rotation_chars[char_index], MAGENTA)
            }
        }
    }

    pub fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_below_gap = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_below_gap)
    }

    /// 检查玩家是否成功通过障碍物
    pub fn is_passed_by_player(&self, player: &Player) -> bool {
        player.x > self.x
    }

    /// 获取障碍物的奖励分数（根据类型）
    pub fn get_score_value(&self) -> i32 {
        match self.obstacle_type {
            ObstacleType::Static => 1,
            ObstacleType::Moving => 2,
            ObstacleType::Rotating => 3,
        }
    }
}