mod obstacle;
mod player;

use bracket_lib::prelude::*;
use image::*;
use obstacle::Obstacle;
use player::Player;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};

enum GameMode {
    Menu,
    Playing,
    End,
    HighScores, // 新增状态
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
    score: i32, // 分数
    score_saved: bool, // 标志位，避免重复保存分数
}

impl State {
    fn new() -> Self {
        State {
            player: Player::new(5, 25),
            frame_time: 0.0,
            mode: GameMode::Menu,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            score: 0,
            score_saved: false, // 初始化为未保存
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        
        //ctx.print_centered(5, "Welcome to Flappy Dragon！");
        // 先清空屏幕
        ctx.cls();
        self.set_background(ctx, "assets/menu_bg.png"); // 菜单背景
        ctx.print_centered(5, "Welcome to Flappy Dragon！");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");
        
        //self.set_background(ctx, "assets/background.png");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        self.set_background(ctx, "assets/game_bg.png"); // 游戏背景
        //ctx.cls_bg(YELLOWGREEN);
        // frame_time_ms 记录了每次调用tick所经过的时间
        self.frame_time += ctx.frame_time_ms;

        // 向前移动并且重力增加
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }

        // 新增：处理左右移动输入
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Left => self.player.move_left(),
                VirtualKeyCode::Right => self.player.move_right(),
                VirtualKeyCode::Space => self.player.flap(),
                _ => {}
            }
        }
        // 空格触发，往上飞
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        // 渲染
        self.player.render(ctx);
        ctx.print(0, 0, "Press Space to Flap");
        ctx.print(0, 1, &format!("Score: {}", self.score));
        // 渲染障碍物
        self.obstacle.render(ctx, self.player.x);
        // 判断是否越过障碍物
        if self.player.x > self.obstacle.x {
            self.score += 1;
            // 渲染新的障碍物
            self.obstacle = Obstacle::new(SCREEN_WIDTH, self.score);
        }
        // 如果y 大于游戏高度，就是坠地或者撞到障碍物，则游戏结束
        if self.player.y > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        self.set_background(ctx, "assets/end_bg.png"); // 结束背景
        ctx.print_centered(5, "You are dead！");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");
        ctx.print_centered(10, "(H) High Scores");

        // 保存分数到文件（仅保存一次）
        if !self.score_saved {
            if let Err(err) = Self::save_score(self.score) {
                ctx.print_centered(12, &format!("Error saving score: {}", err));
            }
            self.score_saved = true; // 标记为已保存
        }

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => {
                    self.restart();
                    self.score_saved = false; // 重置标志位
                }
                VirtualKeyCode::Q => ctx.quitting = true,
                VirtualKeyCode::H => {
                    self.mode = GameMode::HighScores; // 切换到排行榜模式
                }
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
        self.score_saved = false; // 重置保存状态
    }

    pub fn set_background(&mut self, ctx: &mut BTerm, url: &str) {
        if let Ok(img) = image::open(url) {
            let (img_width, img_height) = img.dimensions();
            for x in 0..img_width {
                for y in 0..img_height {
                    // 计算图片坐标（平铺效果）
                    // let img_x = (x as u32) % img_width;
                    // let img_y = (y as u32) % img_height;
                    let pixel = img.get_pixel(x, y);
                    ctx.set_bg(x as i32, y as i32, (pixel[0], pixel[1], pixel[2]));
                }
            }
        } else {
            ctx.cls_bg(BLUE); // 如果图片加载失败，设置默认背景颜色为蓝色
        }
    }

    fn save_score(score: i32) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("scores.txt")?;
        writeln!(file, "{}", score)?;
        Ok(())
    }

    fn load_scores() -> Vec<i32> {
        if let Ok(content) = fs::read_to_string("scores.txt") {
            let mut scores: Vec<i32> = content
                .lines()
                .filter_map(|line| line.parse::<i32>().ok())
                .collect();
            scores.sort_unstable_by(|a, b| b.cmp(a)); // 按分数从高到低排序
            scores
        } else {
            vec![]
        }
    }

    fn display_high_scores(&mut self, ctx: &mut BTerm) {
        let scores = Self::load_scores();
        ctx.cls();
        self.set_background(ctx, "assets/scores_bg.png"); // 高分榜背景
        ctx.print_centered(5, "High Scores:");
        for (i, score) in scores.iter().enumerate().take(10) {
            ctx.print_centered(7 + i as i32, &format!("{}. {}", i + 1, score));
        }
        ctx.print_centered(18, "(M) 返回菜单");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::M => self.mode = GameMode::Menu, // 返回主菜单
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::HighScores => self.display_high_scores(ctx), // 新增处理
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;
    // context.ba
    main_loop(context, State::new())
}
