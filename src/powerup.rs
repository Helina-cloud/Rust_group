use bracket_lib::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum PowerUpType {
    Shield,      // 护盾 - 免疫一次碰撞
    SlowMotion,  // 慢动作 - 减缓游戏速度
    DoubleScore, // 双倍积分 - 一段时间内得分翻倍
    ExtraLife,   // 额外生命 - 增加一条生命
}

pub struct PowerUp {
    pub x: i32,
    pub y: i32,
    pub power_type: PowerUpType,
    pub animation_timer: f32,
}

impl PowerUp {
    pub fn new(x: i32, y: i32, power_type: PowerUpType) -> Self {
        PowerUp {
            x,
            y,
            power_type,
            animation_timer: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.x -= 1; // 向左移动
        self.animation_timer += 16.67; // 假设60FPS，约16.67ms每帧
    }

    pub fn render(&self, ctx: &mut BTerm) {
        let (symbol, color) = self.get_visual_representation();
        
        // 添加闪烁效果
        let blink = ((self.animation_timer / 200.0) as i32) % 2 == 0;
        if blink {
            ctx.set(self.x, self.y, color, BLACK, to_cp437(symbol));
            
            // 添加发光效果（在道具周围显示小点）
            if (self.animation_timer / 400.0) as i32 % 2 == 0 {
                ctx.set(self.x - 1, self.y, color, BLACK, to_cp437('.'));
                ctx.set(self.x + 1, self.y, color, BLACK, to_cp437('.'));
                ctx.set(self.x, self.y - 1, color, BLACK, to_cp437('.'));
                ctx.set(self.x, self.y + 1, color, BLACK, to_cp437('.'));
            }
        }
    }

    fn get_visual_representation(&self) -> (char, (u8, u8, u8)) {
        match self.power_type {
            PowerUpType::Shield => ('S', CYAN),      // 青色盾牌
            PowerUpType::SlowMotion => ('T', PURPLE), // 紫色时间
            PowerUpType::DoubleScore => ('2', GOLD),  // 金色2倍
            PowerUpType::ExtraLife => ('+', GREEN),   // 绿色加号
        }
    }

    /// 检查是否与玩家碰撞
    pub fn collides_with_player(&self, player_x: i32, player_y: i32) -> bool {
        (self.x - player_x).abs() <= 1 && (self.y - player_y).abs() <= 1
    }
}

/// 激活中的道具效果
pub struct ActivePowerUp {
    pub power_type: PowerUpType,
    pub timer: f32, // 剩余时间（毫秒）
}

impl ActivePowerUp {
    pub fn new(power_type: PowerUpType, duration: f32) -> Self {
        ActivePowerUp {
            power_type,
            timer: duration,
        }
    }

    /// 检查效果是否仍然有效
    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }
}