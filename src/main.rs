mod obstacle;
mod player;
mod powerup;
mod difficulty;

use bracket_lib::prelude::*;
use image::*;
use obstacle::Obstacle;
use player::Player;
use powerup::{PowerUp, PowerUpType, ActivePowerUp};
use difficulty::{Difficulty, DifficultySettings};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};

#[derive(Clone)]
enum GameMode {
    Menu,
    DifficultySelect,
    Playing,
    End,
    HighScores,
    Paused,
}

/// 游戏屏幕宽度
const SCREEN_WIDTH: i32 = 90;
/// 游戏屏幕高度
const SCREEN_HEIGHT: i32 = 50;
/// 每隔75毫秒做一些事情
const FRAME_DURATION: f32 = 75.0;

struct State {
    player: Player,
    frame_time: f32,
    mode: GameMode,
    obstacle: Obstacle,
    score: i32,
    score_saved: bool,
    // 新增：难度系统
    selected_difficulty: Difficulty,
    difficulty_settings: DifficultySettings,
    // 新增：道具系统
    powerups: Vec<PowerUp>,
    active_powerups: Vec<ActivePowerUp>,
    powerup_spawn_timer: f32,
    // 新增：游戏效果
    slow_motion_timer: f32,
    shield_active: bool,
    shield_timer: f32,
    lives: i32,
    combo_count: i32,
    last_obstacle_passed: i32,
}

impl State {
    fn new() -> Self {
        State {
            player: Player::new(5, 25),
            frame_time: 0.0,
            mode: GameMode::Menu,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            score: 0,
            score_saved: false,
            selected_difficulty: Difficulty::Normal,
            difficulty_settings: DifficultySettings::new(Difficulty::Normal),
            powerups: Vec::new(),
            active_powerups: Vec::new(),
            powerup_spawn_timer: 0.0,
            slow_motion_timer: 0.0,
            shield_active: false,
            shield_timer: 0.0,
            lives: 3,
            combo_count: 0,
            last_obstacle_passed: -1,
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        self.set_background(ctx, "assets/menu_bg.png");
        
        ctx.print_centered(5, "Welcome to Flappy Dragon！");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(D) Select Difficulty");
        ctx.print_centered(10, "(H) High Scores");
        ctx.print_centered(11, "(Q) Quit Game");
        
        // 显示当前难度
        ctx.print_centered(13, &format!("Current Difficulty: {:?}", self.selected_difficulty));
        
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => {
                    self.restart();
                    self.apply_difficulty_settings();
                }
                VirtualKeyCode::D => self.mode = GameMode::DifficultySelect,
                VirtualKeyCode::H => self.mode = GameMode::HighScores,
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn difficulty_select(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        self.set_background(ctx, "assets/menu_bg.png");
        
        ctx.print_centered(5, "Select Difficulty");
        ctx.print_centered(8, "(E) Easy - More lives, slower obstacles");
        ctx.print_centered(9, "(N) Normal - Balanced gameplay");
        ctx.print_centered(10, "(H) Hard - Faster obstacles, smaller gaps");
        ctx.print_centered(11, "(I) Insane - Maximum challenge!");
        ctx.print_centered(13, "(M) Back to Menu");
        
        // 高亮当前选择
        let highlight_y = match self.selected_difficulty {
            Difficulty::Easy => 8,
            Difficulty::Normal => 9,
            Difficulty::Hard => 10,
            Difficulty::Insane => 11,
        };
        ctx.print_centered(highlight_y, ">>> SELECTED <<<");
        
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::E => {
                    self.selected_difficulty = Difficulty::Easy;
                    self.difficulty_settings = DifficultySettings::new(Difficulty::Easy);
                }
                VirtualKeyCode::N => {
                    self.selected_difficulty = Difficulty::Normal;
                    self.difficulty_settings = DifficultySettings::new(Difficulty::Normal);
                }
                VirtualKeyCode::H => {
                    self.selected_difficulty = Difficulty::Hard;
                    self.difficulty_settings = DifficultySettings::new(Difficulty::Hard);
                }
                VirtualKeyCode::I => {
                    self.selected_difficulty = Difficulty::Insane;
                    self.difficulty_settings = DifficultySettings::new(Difficulty::Insane);
                }
                VirtualKeyCode::M => self.mode = GameMode::Menu,
                _ => {}
            }
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        self.set_background(ctx, "assets/game_bg.png");
        
        // 计算实际帧时间（考虑慢动作效果）
        let effective_frame_time = if self.slow_motion_timer > 0.0 {
            ctx.frame_time_ms * 0.5 // 慢动作时减半速度
        } else {
            ctx.frame_time_ms
        };
        
        self.frame_time += effective_frame_time;
        
        // 更新计时器
        if self.slow_motion_timer > 0.0 {
            self.slow_motion_timer -= ctx.frame_time_ms;
        }
        if self.shield_timer > 0.0 {
            self.shield_timer -= ctx.frame_time_ms;
            if self.shield_timer <= 0.0 {
                self.shield_active = false;
            }
        }

        // 游戏主循环
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }

        // 处理输入
        self.handle_input(ctx);

        // 更新道具
        self.update_powerups(ctx);

        // 渲染
        self.render_game(ctx);

        // 碰撞检测
        self.check_collisions();
    }

    fn handle_input(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Left => self.player.move_left(),
                VirtualKeyCode::Right => self.player.move_right(),
                VirtualKeyCode::Up | VirtualKeyCode::Space => self.player.flap(),
                VirtualKeyCode::Down => self.player.move_down(),
                VirtualKeyCode::Escape => self.mode = GameMode::Paused,
                _ => {}
            }
        }
    }

    fn update_powerups(&mut self, ctx: &mut BTerm) {
        // 生成新道具
        self.powerup_spawn_timer += ctx.frame_time_ms;
        if self.powerup_spawn_timer > self.difficulty_settings.powerup_spawn_rate {
            self.spawn_powerup();
            self.powerup_spawn_timer = 0.0;
        }

        // 更新道具位置
        for powerup in &mut self.powerups {
            powerup.update();
        }

        // 移除超出屏幕的道具
        self.powerups.retain(|p| p.x > -5);

        // 检查道具收集
        // let mut collected_powerups = Vec::new();
        // for (i, powerup) in self.powerups.iter().enumerate() {
        //     if powerup.x == self.player.x && (powerup.y - self.player.y).abs() <= 1 {
        //         collected_powerups.push(i);
        //         self.activate_powerup(powerup.power_type.clone());
        //     }
        // }
        let mut collected_powerups = Vec::new();
        let mut power_types_to_activate = Vec::new();

        for (i, powerup) in self.powerups.iter().enumerate() {
            if powerup.x == self.player.x && (powerup.y - self.player.y).abs() <= 1 {
                collected_powerups.push(i);
                power_types_to_activate.push(powerup.power_type.clone());
            }
        }

        // Now activate all collected power-ups
        for power_type in power_types_to_activate {
            self.activate_powerup(power_type);
        }

        // 移除被收集的道具
        for &i in collected_powerups.iter().rev() {
            self.powerups.remove(i);
        }

        // 更新激活的道具效果
        self.active_powerups.retain_mut(|active| {
            active.timer -= ctx.frame_time_ms;
            active.timer > 0.0
        });
    }

    fn spawn_powerup(&mut self) {
        let mut random = RandomNumberGenerator::new();
        let power_type = match random.range(0, 4) {
            0 => PowerUpType::Shield,
            1 => PowerUpType::SlowMotion,
            2 => PowerUpType::DoubleScore,
            _ => PowerUpType::ExtraLife,
        };
        
        self.powerups.push(PowerUp::new(
            SCREEN_WIDTH + 10,
            random.range(5, SCREEN_HEIGHT - 5),
            power_type,
        ));
    }

    fn activate_powerup(&mut self, power_type: PowerUpType) {
        match power_type {
            PowerUpType::Shield => {
                self.shield_active = true;
                self.shield_timer = 5000.0; // 5秒护盾
            }
            PowerUpType::SlowMotion => {
                self.slow_motion_timer = 3000.0; // 3秒慢动作
            }
            PowerUpType::DoubleScore => {
                self.active_powerups.push(ActivePowerUp {
                    power_type: PowerUpType::DoubleScore,
                    timer: 10000.0, // 10秒双倍积分
                });
            }
            PowerUpType::ExtraLife => {
                self.lives += 1;
            }
        }
    }

    fn render_game(&mut self, ctx: &mut BTerm) {
        // 渲染玩家
        self.player.render(ctx);
        
        // 渲染障碍物
        self.obstacle.render(ctx, self.player.x, &self.difficulty_settings);
        
        // 渲染道具
        for powerup in &self.powerups {
            powerup.render(ctx);
        }
        
        // 渲染UI
        self.render_ui(ctx);
        
        // 渲染特效
        if self.shield_active {
            self.render_shield_effect(ctx);
        }
        if self.slow_motion_timer > 0.0 {
            ctx.print(0, 3, "SLOW MOTION!");
        }
    }

    fn render_ui(&mut self, ctx: &mut BTerm) {
        ctx.print(0, 0, "Controls: Arrow Keys/Space, ESC to Pause");
        ctx.print(0, 1, &format!("Score: {} | Lives: {} | Combo: {}", 
                                 self.score, self.lives, self.combo_count));
        ctx.print(0, 2, &format!("Difficulty: {:?}", self.selected_difficulty));
        
        // 显示激活的道具效果
        let mut y_offset = 4;
        for active in &self.active_powerups {
            ctx.print(0, y_offset, &format!("{:?}: {:.1}s", 
                                          active.power_type, active.timer / 1000.0));
            y_offset += 1;
        }
        
        if self.shield_active {
            ctx.print(0, y_offset, &format!("Shield: {:.1}s", self.shield_timer / 1000.0));
        }
    }

    fn render_shield_effect(&mut self, ctx: &mut BTerm) {
        // 在玩家周围渲染护盾效果
        let shield_char = if (self.shield_timer as i32 / 200) % 2 == 0 { 'O' } else { 'o' };
        ctx.set(self.player.x - 1, self.player.y, CYAN, BLACK, to_cp437(shield_char));
        ctx.set(self.player.x + 1, self.player.y, CYAN, BLACK, to_cp437(shield_char));
        ctx.set(self.player.x, self.player.y - 1, CYAN, BLACK, to_cp437(shield_char));
        ctx.set(self.player.x, self.player.y + 1, CYAN, BLACK, to_cp437(shield_char));
    }

    fn check_collisions(&mut self) {
        // 检查越过障碍物
        if self.player.x > self.obstacle.x && self.last_obstacle_passed != self.obstacle.x {
            let score_multiplier = if self.active_powerups.iter()
                .any(|p| matches!(p.power_type, PowerUpType::DoubleScore)) { 2 } else { 1 };
            
            self.score += score_multiplier;
            self.combo_count += 1;
            self.last_obstacle_passed = self.obstacle.x;
            
            // 生成新障碍物
            self.obstacle = Obstacle::new(SCREEN_WIDTH, self.score);
        }

        // 检查碰撞
        if self.player.y > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            if self.shield_active {
                // 护盾保护，不死亡但移除护盾
                self.shield_active = false;
                self.shield_timer = 0.0;
                self.combo_count = 0; // 重置连击
            } else {
                self.lives -= 1;
                self.combo_count = 0;
                
                if self.lives <= 0 {
                    self.mode = GameMode::End;
                } else {
                    // 重置玩家位置，继续游戏
                    self.player = Player::new(5, 25);
                    self.obstacle = Obstacle::new(SCREEN_WIDTH, self.score);
                }
            }
        }
    }

    fn paused(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        self.set_background(ctx, "assets/game_bg.png");
        
        // 半透明覆盖层效果
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                ctx.set_bg(x, y, (0, 0, 0)); // 黑色覆盖层
            }
        }
        
        ctx.print_centered(20, "GAME PAUSED");
        ctx.print_centered(22, "(R) Resume");
        ctx.print_centered(23, "(M) Main Menu");
        ctx.print_centered(24, "(Q) Quit");
        
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::R | VirtualKeyCode::Escape => self.mode = GameMode::Playing,
                VirtualKeyCode::M => self.mode = GameMode::Menu,
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        self.set_background(ctx, "assets/end_bg.png");
        ctx.print_centered(5, "Game Over！");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(7, &format!("Best combo: {}", self.combo_count));
        ctx.print_centered(8, &format!("Difficulty: {:?}", self.selected_difficulty));
        ctx.print_centered(10, "(P) Play Again");
        ctx.print_centered(11, "(M) Main Menu");
        ctx.print_centered(12, "(H) High Scores");
        ctx.print_centered(13, "(Q) Quit Game");

        if !self.score_saved {
            if let Err(err) = Self::save_score(self.score, self.selected_difficulty.clone()) {
                ctx.print_centered(15, &format!("Error saving score: {}", err));
            }
            self.score_saved = true;
        }

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => {
                    self.restart();
                    self.apply_difficulty_settings();
                    self.score_saved = false;
                }
                VirtualKeyCode::M => self.mode = GameMode::Menu,
                VirtualKeyCode::H => self.mode = GameMode::HighScores,
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        self.score = 0;
        self.score_saved = false;
        self.powerups.clear();
        self.active_powerups.clear();
        self.powerup_spawn_timer = 0.0;
        self.slow_motion_timer = 0.0;
        self.shield_active = false;
        self.shield_timer = 0.0;
        self.combo_count = 0;
        self.last_obstacle_passed = -1;
    }

    fn apply_difficulty_settings(&mut self) {
        self.lives = self.difficulty_settings.starting_lives;
        // 其他难度设置将在游戏过程中应用
    }

    pub fn set_background(&mut self, ctx: &mut BTerm, url: &str) {
        if let Ok(img) = image::open(url) {
            let (img_width, img_height) = img.dimensions();
            for x in 0..img_width {
                for y in 0..img_height {
                    let pixel = img.get_pixel(x, y);
                    ctx.set_bg(x as i32, y as i32, (pixel[0], pixel[1], pixel[2]));
                }
            }
        } else {
            ctx.cls_bg(BLUE);
        }
    }

    fn save_score(score: i32, difficulty: Difficulty) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("scores.txt")?;
        writeln!(file, "{}:{:?}", score, difficulty)?;
        Ok(())
    }

    fn load_scores() -> Vec<(i32, Difficulty)> {
        if let Ok(content) = fs::read_to_string("scores.txt") {
            let mut scores: Vec<(i32, Difficulty)> = content
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() == 2 {
                        if let (Ok(score), Ok(difficulty)) = (
                            parts[0].parse::<i32>(),
                            parts[1].parse::<Difficulty>(),
                        ) {
                            Some((score, difficulty))
                        } else {
                            None
                        }
                    } else {
                        // 兼容旧格式（只有分数）
                        line.parse::<i32>().ok().map(|score| (score, Difficulty::Normal))
                    }
                })
                .collect();
            scores.sort_unstable_by(|a, b| b.0.cmp(&a.0));
            scores
        } else {
            vec![]
        }
    }

    fn display_high_scores(&mut self, ctx: &mut BTerm) {
        let scores = Self::load_scores();
        ctx.cls();
        self.set_background(ctx, "assets/scores_bg.png");
        ctx.print_centered(5, "High Scores:");
        
        for (i, (score, difficulty)) in scores.iter().enumerate().take(10) {
            ctx.print_centered(
                7 + i as i32,
                &format!("{}. {} ({:?})", i + 1, score, difficulty),
            );
        }
        
        ctx.print_centered(18, "(M) Back to Menu");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::M => self.mode = GameMode::Menu,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::DifficultySelect => self.difficulty_select(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::Paused => self.paused(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::HighScores => self.display_high_scores(ctx),
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon - Enhanced Edition")
        .build()?;
    main_loop(context, State::new())
}