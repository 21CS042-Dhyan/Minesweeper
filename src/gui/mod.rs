//gui.mod.rs
#![deny(clippy::all)]

use crate::db;
use eframe::egui;
use rand::Rng;
use std::collections::HashSet;
use std::time::{Duration, Instant};
// use std::io::stdin;

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Empty,
    Mine,
    Number(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellState {
    Hidden,
    Revealed,
    Flagged,
    Questioned,
}

struct Board {
    width: usize,
    height: usize,
    mine_count: usize,
    cells: Vec<Vec<Cell>>,
    cell_states: Vec<Vec<CellState>>,
    initialized: bool,
    revealed: Vec<Vec<bool>>,
    flagged: Vec<Vec<bool>>,
}

impl Board {
    fn new(width: usize, height: usize, mine_count: usize) -> Self {
        let cells = vec![vec![Cell::Empty; width]; height];
        let cell_states = vec![vec![CellState::Hidden; width]; height];
        let revealed = vec![vec![false; width]; height];
        let flagged = vec![vec![false; width]; height];
        Self {
            width,
            height,
            mine_count,
            cells,
            cell_states,
            initialized: false,
            revealed,
            flagged,
        }
    }

    fn initialize(&mut self, first_x: usize, first_y: usize) {
        let mut rng = rand::thread_rng();
        let mut mines_placed = 0;
        let mut positions = HashSet::new();

        let mut avoid_positions = HashSet::new();
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = (first_x as isize + dx) as usize;
                let ny = (first_y as isize + dy) as usize;
                if nx < self.width && ny < self.height {
                    avoid_positions.insert((nx, ny));
                }
            }
        }

        while mines_placed < self.mine_count {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);
            if !positions.contains(&(x, y)) && !avoid_positions.contains(&(x, y)) {
                self.cells[y][x] = Cell::Mine;
                positions.insert((x, y));
                mines_placed += 1;
            }
        }

        for y in 0..self.height {
            for x in 0..self.width {
                if self.cells[y][x] == Cell::Mine {
                    continue;
                }
                let mut mine_count = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = (x as isize + dx) as usize;
                        let ny = (y as isize + dy) as usize;
                        if nx < self.width && ny < self.height && self.cells[ny][nx] == Cell::Mine {
                            mine_count += 1;
                        }
                    }
                }
                if mine_count > 0 {
                    self.cells[y][x] = Cell::Number(mine_count);
                }
            }
        }

        self.initialized = true;
    }

    fn toggle_state(&mut self, x: usize, y: usize) {
        if !self.initialized {
            self.initialize(x, y);
        }
        self.cell_states[y][x] = match self.cell_states[y][x] {
            CellState::Hidden => {
                self.flagged[y][x] = true;
                CellState::Flagged
            }
            CellState::Flagged => {
                self.flagged[y][x] = false;
                CellState::Questioned
            }
            CellState::Questioned => {
                self.flagged[y][x] = false;
                CellState::Hidden
            }
            CellState::Revealed => CellState::Revealed,
        };
    }

    fn reveal(&mut self, x: usize, y: usize) -> Result<(), String> {
        if !self.initialized {
            self.initialize(x, y);
        }
        if self.revealed[y][x] {
            if let Cell::Number(num) = self.cells[y][x] {
                return self.multiguess(x, y, num);
            } else {
                return Err("Cell already revealed".to_string());
            }
        }

        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            if self.revealed[cy][cx] {
                continue;
            }
            self.revealed[cy][cx] = true;
            self.cell_states[cy][cx] = CellState::Revealed;

            match self.cells[cy][cx] {
                Cell::Mine => return Err("Game Over! You hit a mine.".to_string()),
                Cell::Empty => {
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let nx = cx as isize + dx;
                            let ny = cy as isize + dy;
                            if nx >= 0
                                && nx < self.width as isize
                                && ny >= 0
                                && ny < self.height as isize
                            {
                                let nx = nx as usize;
                                let ny = ny as usize;
                                if !self.revealed[ny][nx] {
                                    stack.push((nx, ny));
                                }
                            }
                        }
                    }
                }
                Cell::Number(_) => {}
            }
        }
        Ok(())
    }

    // fn check_win(&self) -> bool {
    //     let mut covered_cells = 0;
    //     for y in 0..self.height {
    //         for x in 0..self.width {
    //             if !self.revealed[y][x] {
    //                 covered_cells += 1;
    //             }
    //         }
    //     }
    //     covered_cells == self.mine_count
    // }

    fn multiguess(&mut self, x: usize, y: usize, num: u8) -> Result<(), String> {
        let mut flagged_count = 0;
        let mut unopened_cells = Vec::new();

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    if self.flagged[ny][nx] {
                        flagged_count += 1;
                    } else if !self.revealed[ny][nx] {
                        unopened_cells.push((nx, ny));
                    }
                }
            }
        }

        if flagged_count == num {
            for (nx, ny) in unopened_cells {
                self.reveal(nx, ny)?;
            }
            Ok(())
        } else {
            Err("Number of flags doesn't match the cell number".to_string())
        }
    }

    fn reveal_all_mines(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.cells[y][x] == Cell::Mine {
                    self.revealed[y][x] = true;
                    self.cell_states[y][x] = CellState::Revealed;
                }
            }
        }
    }

    fn reveal_all_cells(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.revealed[y][x] = true;
                self.cell_states[y][x] = CellState::Revealed;
            }
        }
    }
}

pub struct MinesweeperApp {
    board: Board,
    game_over: bool,
    game_won: bool,
    cursor_x: usize,
    cursor_y: usize,
    difficulty_selection: bool,
    show_end_game_popup: bool,
    db_connection: Option<db::DbConnection>,
    game_start_time: Option<Instant>,
    flags_count: usize,
    game_duration: Duration,
    last_update: Instant,
    name_input: String,
    show_name_input: bool,
}

impl MinesweeperApp {
    pub fn new() -> Self {
        let db_connection =
            db::DbConnection::new("mysql://dhyan:root@localhost:3306/minesweeper_db").ok();
        Self {
            board: Board::new(8, 8, 10), // Default to Easy
            game_over: false,
            game_won: false,
            cursor_x: 0,
            cursor_y: 0,
            difficulty_selection: true,
            show_end_game_popup: false,
            db_connection,
            game_start_time: None,
            flags_count: 0,
            game_duration: Duration::new(0, 0),
            last_update: Instant::now(),
            name_input: String::new(),
            show_name_input: false,
        }
    }

    fn restart(&mut self, width: usize, height: usize, mine_count: usize) {
        self.board = Board::new(width, height, mine_count);
        self.game_over = false;
        self.game_won = false;
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.difficulty_selection = false;
        self.show_end_game_popup = false;
        self.game_start_time = Some(std::time::Instant::now());
        self.flags_count = 0;
        self.game_duration = Duration::new(0, 0);
        self.last_update = Instant::now();
        self.name_input = String::new();
        self.show_name_input = false;
    }

    fn check_win_condition(&mut self) {
        let mut correctly_flagged_mines = 0;
        let mut revealed_cells = 0;

        for y in 0..self.board.height {
            for x in 0..self.board.width {
                if self.board.revealed[y][x] {
                    revealed_cells += 1;
                } else if self.board.flagged[y][x] && self.board.cells[y][x] == Cell::Mine {
                    correctly_flagged_mines += 1;
                }
            }
        }

        if revealed_cells + correctly_flagged_mines == self.board.width * self.board.height {
            self.game_won = true;
            self.board.reveal_all_cells();
            self.show_end_game_popup = true;

            if let Some(db) = &self.db_connection {
                let difficulty = match (self.board.width, self.board.height, self.board.mine_count)
                {
                    (8, 8, 10) => "Easy",
                    (16, 16, 40) => "Medium",
                    _ => "Custom",
                };
                if let Ok(scores) = db.get_top_10_scores(difficulty) {
                    if scores.len() < 10
                        || self.game_duration.as_secs_f32() < scores.last().unwrap().time
                    {
                        self.show_name_input = true;
                    }
                }
            }
        }
    }

    fn update_flags_count(&mut self) {
        self.flags_count = self.board.flagged.iter().flatten().filter(|&&f| f).count();
    }

    fn submit_high_score(&mut self) {
        if let Some(db) = &self.db_connection {
            let difficulty = match (self.board.width, self.board.height, self.board.mine_count) {
                (8, 8, 10) => "Easy",
                (16, 16, 40) => "Medium",
                _ => "Custom",
            };
            if let Err(e) = db.add_high_score(
                &self.name_input,
                self.game_duration.as_secs_f32(),
                difficulty,
            ) {
                eprintln!("Failed to save high score: {}", e);
            }
        }
        self.show_name_input = false;
    }

    fn display_high_scores(&self, ui: &mut egui::Ui) {
        if let Some(db) = &self.db_connection {
            let difficulty = match (self.board.width, self.board.height, self.board.mine_count) {
                (8, 8, 10) => "Easy",
                (16, 16, 40) => "Medium",
                (30, 16, 99) => "Hard",
                _ => "Custom",
            };

            if let Ok(scores) = db.get_top_10_scores(difficulty) {
                ui.heading("Top 10 High Scores");
                for (i, score) in scores.iter().enumerate() {
                    ui.label(format!("{}. {} - {:.2}s", i + 1, score.name, score.time));
                }
            } else {
                ui.label("Failed to retrieve high scores");
            }
        } else {
            ui.label("Database connection not available");
        }
    }
}

impl eframe::App for MinesweeperApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Update game duration
        let now = Instant::now();
        if !self.game_over && !self.game_won && self.board.initialized {
            self.game_duration += now - self.last_update;
        }
        self.last_update = now;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Minesweeper");
                ui.separator();

                if self.difficulty_selection {
                    ui.heading("Choose difficulty:");
                    if ui.button("Easy (8x8, 10 mines)").clicked() {
                        self.restart(8, 8, 10);
                    }
                    if ui.button("Medium (16x16, 40 mines)").clicked() {
                        self.restart(16, 16, 40);
                    }
                    if ui.button("Hard (30x16, 99 mines)").clicked() {
                        self.restart(30, 16, 99);
                    }
                } else {
                    if ui.button("Restart").clicked() {
                        self.difficulty_selection = true;
                    }

                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Flags: {}/{}",
                            self.flags_count, self.board.mine_count
                        ));
                        ui.label(format!("Time: {:.1}s", self.game_duration.as_secs_f32()));
                    });

                    let available_size = ui.available_size();
                    let cell_size = (available_size.x / self.board.width as f32)
                        .min(available_size.y / self.board.height as f32);
                    let board_width = self.board.width as f32 * cell_size;
                    let board_height = self.board.height as f32 * cell_size;

                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .show_viewport(ui, |ui, viewport| {
                            let (response, painter) = ui.allocate_painter(
                                egui::vec2(board_width, board_height),
                                egui::Sense::click_and_drag(),
                            );

                            let to_screen = egui::emath::RectTransform::from_to(
                                egui::Rect::from_min_size(egui::Pos2::ZERO, response.rect.size()),
                                response.rect,
                            );

                            for y in 0..self.board.height {
                                for x in 0..self.board.width {
                                    let cell_rect = egui::Rect::from_min_size(
                                        egui::pos2(x as f32 * cell_size, y as f32 * cell_size),
                                        egui::vec2(cell_size, cell_size),
                                    );

                                    if viewport.intersects(cell_rect) {
                                        let cell_rect = to_screen.transform_rect(cell_rect);

                                        let fill_color = match self.board.cell_states[y][x] {
                                            CellState::Hidden => egui::Color32::LIGHT_GRAY,
                                            CellState::Revealed => {
                                                match self.board.cells[y][x] {
                                                    Cell::Empty => egui::Color32::WHITE,
                                                    Cell::Mine => egui::Color32::RED,
                                                    Cell::Number(n) => match n {
                                                        1 => egui::Color32::from_rgb(173, 216, 230), // Light Blue
                                                        2 => egui::Color32::from_rgb(144, 238, 144), // Light Green
                                                        3 => egui::Color32::from_rgb(255, 255, 224), // Light Yellow
                                                        4 => egui::Color32::from_rgb(255, 218, 185), // Peach
                                                        5 => egui::Color32::from_rgb(255, 192, 203), // Pink
                                                        6 => egui::Color32::from_rgb(255, 160, 122), // Light Salmon
                                                        7 => egui::Color32::from_rgb(216, 191, 216), // Thistle
                                                        8 => egui::Color32::from_rgb(221, 160, 221), // Plum
                                                        _ => egui::Color32::WHITE,
                                                    },
                                                }
                                            }
                                            CellState::Flagged => egui::Color32::RED,
                                            CellState::Questioned => egui::Color32::YELLOW,
                                        };

                                        painter.rect_filled(cell_rect, 0.0, fill_color);
                                        painter.rect_stroke(
                                            cell_rect,
                                            0.0,
                                            egui::Stroke::new(1.0, egui::Color32::BLACK),
                                        );

                                        if self.board.cell_states[y][x] == CellState::Revealed {
                                            match self.board.cells[y][x] {
                                                Cell::Empty => {}
                                                Cell::Mine => {
                                                    painter.text(
                                                        cell_rect.center(),
                                                        egui::Align2::CENTER_CENTER,
                                                        "*",
                                                        egui::FontId::proportional(cell_size * 0.8),
                                                        egui::Color32::BLACK,
                                                    );
                                                }
                                                Cell::Number(n) => {
                                                    painter.text(
                                                        cell_rect.center(),
                                                        egui::Align2::CENTER_CENTER,
                                                        n.to_string(),
                                                        egui::FontId::proportional(cell_size * 0.8),
                                                        egui::Color32::BLACK,
                                                    );
                                                }
                                            }
                                        } else if self.board.cell_states[y][x]
                                            == CellState::Questioned
                                        {
                                            painter.text(
                                                cell_rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                "?",
                                                egui::FontId::proportional(cell_size * 0.8),
                                                egui::Color32::BLACK,
                                            );
                                        }

                                        if x == self.cursor_x && y == self.cursor_y {
                                            painter.rect_stroke(
                                                cell_rect,
                                                0.0,
                                                egui::Stroke::new(2.0, egui::Color32::BLUE),
                                            );
                                            painter.rect_filled(
                                                cell_rect,
                                                0.0,
                                                egui::Color32::from_rgba_unmultiplied(
                                                    0, 0, 255, 64,
                                                ),
                                            );
                                        }
                                    }
                                }
                            }

                            if let Some(pos) = response.hover_pos() {
                                let pos = to_screen.inverse().transform_pos(pos);
                                let x = (pos.x / cell_size) as usize;
                                let y = (pos.y / cell_size) as usize;
                                if x < self.board.width && y < self.board.height {
                                    if response.clicked() {
                                        if !self.game_over && !self.game_won {
                                            if !self.board.initialized {
                                                self.game_start_time = Some(Instant::now());
                                            }
                                            if let Err(err) = self.board.reveal(x, y) {
                                                if err == "Game Over! You hit a mine." {
                                                    self.game_over = true;
                                                    self.board.reveal_all_mines();
                                                    self.show_end_game_popup = true;
                                                }
                                            } else {
                                                self.check_win_condition();
                                            }
                                        }
                                    } else if response.secondary_clicked() {
                                        if !self.game_over && !self.game_won {
                                            self.board.toggle_state(x, y);
                                            self.update_flags_count();
                                            self.check_win_condition();
                                        }
                                    }
                                }
                            }
                        });

                    // Keyboard input handling
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) && self.cursor_x > 0 {
                        self.cursor_x -= 1;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowRight))
                        && self.cursor_x < self.board.width - 1
                    {
                        self.cursor_x += 1;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) && self.cursor_y > 0 {
                        self.cursor_y -= 1;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown))
                        && self.cursor_y < self.board.height - 1
                    {
                        self.cursor_y += 1;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Space)) {
                        if ui.input(|i| i.modifiers.ctrl) {
                            if !self.game_over && !self.game_won {
                                self.board.toggle_state(self.cursor_x, self.cursor_y);
                                self.update_flags_count();
                                self.check_win_condition();
                            }
                        } else {
                            if !self.game_over && !self.game_won {
                                if !self.board.initialized {
                                    self.game_start_time = Some(Instant::now());
                                }
                                if let Err(err) = self.board.reveal(self.cursor_x, self.cursor_y) {
                                    if err == "Game Over! You hit a mine." {
                                        self.game_over = true;
                                        self.board.reveal_all_mines();
                                        self.show_end_game_popup = true;
                                    }
                                } else {
                                    self.check_win_condition();
                                }
                            }
                        }
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::R)) {
                        if ui.input(|i| i.modifiers.ctrl) {
                            self.difficulty_selection = true;
                            self.show_end_game_popup = false;
                        }
                    }

                    if self.game_over {
                        ui.colored_label(egui::Color32::RED, "Game Over! You hit a mine.");
                    } else if self.game_won {
                        ui.colored_label(egui::Color32::GREEN, "Congratulations! You won!");
                    }
                }
            });
        });

        if self.show_end_game_popup {
            egui::Window::new("Game Over")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if self.game_won {
                        ui.heading("Congratulations! You won!");
                    } else {
                        ui.heading("Game Over! You hit a mine.");
                    }
                    ui.add_space(20.0);

                    if self.show_name_input {
                        ui.horizontal(|ui| {
                            ui.label("Enter your name:");
                            ui.text_edit_singleline(&mut self.name_input);
                        });
                        if ui.button("Submit").clicked() {
                            self.submit_high_score();
                        }
                    } else {
                        self.display_high_scores(ui);
                    }

                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui.button("Restart").clicked() {
                            self.difficulty_selection = true;
                            self.show_end_game_popup = false;
                        }
                        if ui.button("Quit").clicked() {
                            frame.close();
                        }
                    });
                });
        }

        // Request a repaint to ensure continuous updates
        ctx.request_repaint();
    }
}

pub fn run() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Minesweeper",
        options,
        Box::new(|_cc| Box::new(MinesweeperApp::new())),
    )
}