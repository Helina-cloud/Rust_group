use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Difficulty {
    Easy,
    Normal, 
    Hard,
    Insane,
}

impl FromStr for Difficulty {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Easy" => Ok(Difficulty::Easy),
            "Normal" => Ok(Difficulty::Normal),
            "Hard" => Ok(Difficulty::Hard),
            "Insane" => Ok(Difficulty::Insane),
            _ => Err(()),
        }
    }
}

pub struct DifficultySettings {
    pub obstacle_speed_multiplier: f32,
    pub obstacle_gap_size_modifier: i32,
    pub starting_lives: i32,
    pub powerup_spawn_rate: f32, // 毫秒
    pub score_multiplier: i32,
    pub gravity_multiplier: f32,
}

impl DifficultySettings {
    pub fn new(difficulty: Difficulty) -> Self {
        match difficulty {
            Difficulty::Easy => DifficultySettings {
                obstacle_speed_multiplier: 0.7,
                obstacle_gap_size_modifier: 3, // 增加空隙大小
                starting_lives: 5,
                powerup_spawn_rate: 3000.0, // 3秒生成一个道具
                score_multiplier: 1,
                gravity_multiplier: 0.8,
            },
            Difficulty::Normal => DifficultySettings {
                obstacle_speed_multiplier: 1.0,
                obstacle_gap_size_modifier: 0,
                starting_lives: 3,
                powerup_spawn_rate: 4000.0, // 4秒生成一个道具
                score_multiplier: 1,
                gravity_multiplier: 1.0,
            },
            Difficulty::Hard => DifficultySettings {
                obstacle_speed_multiplier: 1.3,
                obstacle_gap_size_modifier: -2, // 减少空隙大小
                starting_lives: 2,
                powerup_spawn_rate: 5000.0, // 5秒生成一个道具
                score_multiplier: 2,
                gravity_multiplier: 1.2,
            },
            Difficulty::Insane => DifficultySettings {
                obstacle_speed_multiplier: 1.6,
                obstacle_gap_size_modifier: -4, // 大幅减少空隙大小
                starting_lives: 1,
                powerup_spawn_rate: 6000.0, // 6秒生成一个道具
                score_multiplier: 3,
                gravity_multiplier: 1.5,
            },
        }
    }

    /// 根据当前分数动态调整难度
    pub fn get_dynamic_speed(&self, score: i32) -> f32 {
        let base_speed = self.obstacle_speed_multiplier;
        let score_factor = (score as f32 * 0.02).min(0.5); // 最多增加50%速度
        base_speed + score_factor
    }

    /// 根据分数动态调整空隙大小
    pub fn get_dynamic_gap_size(&self, base_size: i32, score: i32) -> i32 {
        let score_reduction = (score / 10).min(3); // 每10分减少1点空隙，最多减少3点
        let modified_size = base_size + self.obstacle_gap_size_modifier - score_reduction;
        modified_size.max(4) // 确保最小空隙为4
    }
}