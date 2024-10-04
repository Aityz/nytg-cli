use crate::tabs::Tabber;
use chrono::{DateTime, Duration, Local};
use rand::seq::SliceRandom;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph, Tabs},
    Frame,
};
use GameResult::*;

#[derive(PartialEq, Debug, Clone)]
enum GameResult {
    Green,
    Yellow,
    Grey,
}

static WORDS: &str = include_str!("../assets/wordle.txt");

fn calc_game(correct: &str, guess: &str) -> [GameResult; 5] {
    const ARRAY_REPEAT_VALUE: GameResult = GameResult::Grey;

    let mut res = [ARRAY_REPEAT_VALUE; 5];

    let correct_chars: Vec<char> = correct.chars().collect();
    let guess_chars: Vec<char> = guess.chars().collect();

    let mut correct_count = [0; 26]; // assuming only lowercase letters
    let mut guess_count = [0; 26];

    // First pass: identify all Green matches
    for i in 0..5 {
        if correct_chars[i] == guess_chars[i] {
            res[i] = Green;
        } else {
            correct_count[correct_chars[i] as usize - 'a' as usize] += 1;
            guess_count[guess_chars[i] as usize - 'a' as usize] += 1;
        }
    }

    // Second pass: identify Yellow matches
    for i in 0..5 {
        if res[i] != Green && correct_count[guess_chars[i] as usize - 'a' as usize] > 0 {
            res[i] = Yellow;
            correct_count[guess_chars[i] as usize - 'a' as usize] -= 1;
        }
    }

    res
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct App {
    pub page: Tabber,      // keeps track of which page we are on
    pub should_quit: bool, // determines whether to quit or not
    pub game_cache: Vec<(u8, DateTime<Local>, String)>, // Vector of game cached data
    pub date: DateTime<Local>, // date to check games on
    pub current_game: (u8, serde_json::Value), // The current game being played
    pub guess_buffer: Vec<char>, // The guess
    pub guesses: Vec<String>, // Guesses are stored here
    pub lines: Vec<String>,

    #[serde(skip)]
    pub words: Vec<String>,

    pub game_string: Vec<String>,

    pub game_complete: bool,

    pub word_order: Vec<String>, // for connections

    pub used_words: Vec<String>,

    pub found_words: Vec<String>,
    pub needed_words: u8,
}

impl Default for App {
    fn default() -> Self {
        Self {
            page: Tabber {
                index: 0,
                values: vec![
                    "Wordle".to_string(),
                    "Connections".to_string(),
                    "Strands".to_string(),
                ],
            },
            should_quit: false,
            game_cache: Vec::new(),
            date: Local::now(),
            current_game: (255, serde_json::json!({})),
            guess_buffer: Vec::new(),
            guesses: Vec::new(),
            lines: Vec::new(),

            words: WORDS.split('\n').map(|x| x.trim().to_string()).collect(),

            game_string: Vec::new(),

            game_complete: false,

            word_order: Vec::new(),
            used_words: Vec::new(),

            found_words: Vec::new(),
            needed_words: 0,
        }
    }
}

impl App {
    pub fn generate_game_string(&mut self) {
        match self.page.index {
            0 => {
                self.game_string.append(&mut vec![
                    "Wordle: Guess a five letter word to win the game.".into(),
                    "".to_string(),
                ]);
            }

            1 => {
                self.game_string.append(&mut vec![
                    "Connections: Group words by a common thread.".into(),
                    "".to_string(),
                ]);

                let mut words: Vec<String> = Vec::new();

                let temp_vec = Vec::new();

                let categories = self.current_game.1["categories"]
                    .as_array()
                    .unwrap_or(&temp_vec);

                for cat in categories {
                    let cards = cat["cards"].as_array().unwrap_or(&temp_vec);

                    for card in cards {
                        let card = card.as_object();

                        if let Some(c) = card {
                            words.push(c["content"].as_str().unwrap_or_default().to_string());
                        }
                    }
                }

                words.shuffle(&mut rand::thread_rng());

                self.word_order = words.clone();

                let letters = [
                    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                    'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
                ];

                for (i, word) in words.into_iter().enumerate() {
                    self.game_string.push(format!("{}. {}", letters[i], word));
                }

                self.game_string.push("".into());
            }

            2 => {
                self.game_string.push("Strands: Uncover words.".to_string());

                self.game_string.push(format!(
                    "Clue: {}",
                    self.current_game.1["clue"].as_str().unwrap_or_default()
                ));

                // generate self.needed_words

                self.needed_words = (self.current_game.1["themeWords"]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .len()
                    + 1) as u8;

                self.game_string
                    .push(format!("Theme words: {}", self.needed_words));

                self.game_string.push("".into());

                let null = Vec::new();

                let starting_board = self.current_game.1["startingBoard"]
                    .as_array()
                    .unwrap_or(&null);

                for line in starting_board {
                    self.game_string
                        .push(line.as_str().unwrap_or_default().to_string());
                }

                self.game_string.push("".to_string());
            }

            _ => {}
        }
    }

    pub fn key(&mut self, char: char) {
        let max = match self.page.index {
            0 => 5,
            1 => 4,
            2 => 20,
            _ => 1,
        };

        if self.guess_buffer.len() != max {
            self.guess_buffer.push(char);
        }
    }

    pub fn clear_state(&mut self) {
        self.guess_buffer.clear();
        self.guesses.clear();
        self.lines.clear();
        self.game_string.clear();
        self.used_words.clear();
        self.word_order.clear();
        self.found_words.clear();
        self.needed_words = 0;
    }

    pub fn download(&mut self) -> Result<(u8, serde_json::Value), ()> {
        for game in &self.game_cache {
            if self.page.index == game.0 && self.date.date_naive() == game.1.date_naive() {
                return Ok((game.0, serde_json::from_str(&game.2).unwrap_or_default()));
            }
        }

        let base_url = match self.page.index {
            0 => "https://www.nytimes.com/svc/wordle/v2/",
            1 => "https://www.nytimes.com/svc/connections/v2/",
            2 => "https://www.nytimes.com/svc/strands/v2/",
            _ => "",
        };

        let data = reqwest::blocking::get(format!("{}{}.json", base_url, self.date.date_naive()));

        if let Ok(d) = data {
            if let Ok(json) = d.json::<serde_json::Value>() {
                if json["status"].as_str().unwrap_or("OK") == "ERROR" {
                    return Err(());
                }
                self.game_cache
                    .push((self.page.index, self.date, json.to_string()));

                Ok((self.page.index, json))
            } else {
                self.lines
                    .push("Failed to convert data to JSON".to_string());

                Err(())
            }
        } else {
            self.lines.push("Failed to download game".to_string());

            Err(())
        }
    }
    pub fn left(&mut self) {
        self.page.prev();

        self.clear_state();

        if let Ok(data) = self.download() {
            self.current_game = data;
        }

        self.game_complete = false;

        self.generate_game_string();
    }

    pub fn right(&mut self) {
        self.page.next();

        self.clear_state();

        if let Ok(data) = self.download() {
            self.current_game = data;
        }

        self.game_complete = false;

        self.generate_game_string();
    }

    pub fn up(&mut self) {
        // increment the date by one

        self.date += Duration::days(1);

        self.clear_state();

        if let Ok(data) = self.download() {
            self.current_game = data;
        }

        self.game_complete = false;

        self.generate_game_string();
    }

    pub fn down(&mut self) {
        // decrement the date

        self.date -= Duration::days(1);

        self.clear_state();

        if let Ok(data) = self.download() {
            self.current_game = data;
        }

        self.game_complete = false;

        self.generate_game_string();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn enter(&mut self) {
        let max = match self.page.index {
            0 => 5,
            1 => 4,
            2 => 20,
            _ => 1,
        };

        match self.page.index {
            0 | 1 => {
                if self.guess_buffer.len() != max {
                    return;
                }
            }
            2 => {
                if self.guess_buffer.len() < 4 {
                    return;
                }
            }
            _ => (),
        }

        self.guesses.push(
            self.guess_buffer
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(""),
        );

        // game logic

        match self.page.index {
            0 => {
                // wordle

                if !self.words.contains(self.guesses.last().unwrap()) {
                    return;
                }
            }

            1 => {
                // connections

                let guess = self.guesses.last().unwrap();

                let chars = guess.chars().collect::<Vec<_>>();

                let mut dedup_chars = chars.clone();

                dedup_chars.dedup();

                if dedup_chars != chars {
                    return;
                }
            }

            _ => {}
        }

        // now push it to lines

        match self.page.index {
            0 => {
                // color the grid

                let guess = self.guesses.last().unwrap();
                let correct = self.current_game.1["solution"].as_str().unwrap_or("crane");

                let result = calc_game(correct, guess)
                    .iter()
                    .map(|x| match x {
                        Green => "\x1b[32mG\x1b[0m",
                        Yellow => "\x1b[33mY\x1b[0m",
                        Grey => "\x1b[90mN\x1b[0m",
                    })
                    .collect::<Vec<_>>();

                self.lines.push(format!("{}, {}", guess, result.join("")));

                if guess == correct {
                    // correct guess :D

                    self.lines.push("Game complete!".to_string());

                    self.game_complete = true;
                }
            }
            1 => {
                // push the guess into the lines vector

                let letters = [
                    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                    'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
                ];

                let guess = self.guesses.last().unwrap();

                let mut words: Vec<&str> = Vec::new();

                let unknown_str = "Unknown".to_string();

                for ch in guess.chars() {
                    let word = self
                        .word_order
                        .get(letters.iter().position(|&x| x == ch).unwrap_or(1000))
                        .unwrap_or(&unknown_str)
                        .as_str();

                    if self.used_words.contains(&word.to_string()) {
                        return;
                    }

                    words.push(word);
                }

                let mut status = 0; // 0 => Incorrect, 1 => One Away, 2 => Correct

                let temp_vec = Vec::new();

                for cat in self.current_game.1["categories"]
                    .as_array()
                    .unwrap_or(&temp_vec)
                {
                    let mut correct = 0;

                    let cards = cat["cards"].as_array().unwrap_or(&temp_vec);

                    for card in cards {
                        for word in &words {
                            if card["content"].as_str().unwrap_or("") == *word {
                                correct += 1;
                            }
                        }
                    }

                    if correct == 3 && status != 2 {
                        status = 1;
                    } else if correct == 4 {
                        status = 2;
                    }
                }

                self.lines.push(format!(
                    "{} - {}",
                    words.join(", "),
                    match status {
                        0 => "More than one away",
                        1 => "One away",
                        2 => "Correct!",
                        _ => "Unknown",
                    }
                ));

                if status == 2 {
                    self.used_words
                        .append(&mut words.iter().map(|x| x.to_string()).collect::<Vec<String>>());
                }

                if self.used_words.len() == 16 {
                    self.game_complete = true;

                    self.lines.push("Game complete!".to_string());
                }
            }

            2 => {
                let guess = self.guesses.last().unwrap().to_lowercase();

                if self.found_words.contains(&guess) {
                    self.guess_buffer.clear();

                    return;
                }

                let null_vec = vec![];

                // detect if its the spangram or not

                if self.current_game.1["spangram"]
                    .as_str()
                    .unwrap_or_default()
                    .to_lowercase()
                    == *guess
                {
                    self.lines.push(format!("{} is the Spangram!", guess));

                    self.found_words.push(guess.clone());
                }

                let mut theme_word = false;

                for tw in self.current_game.1["themeWords"]
                    .as_array()
                    .unwrap_or(&null_vec)
                {
                    if tw.as_str().unwrap_or_default().to_lowercase() == *guess {
                        theme_word = true;
                    }
                }

                if theme_word {
                    self.lines.push(format!("{} is a theme word!", guess));

                    self.found_words.push(guess);
                }
                if self.found_words.len() as u8 == self.needed_words {
                    self.game_complete = true;

                    self.lines.push("Game complete!".to_string());
                }
            }

            _ => {
                self.lines.push(format!(
                    "Unknown Game Guess {}",
                    self.guesses.last().unwrap()
                ));
            }
        }

        self.guess_buffer.clear();
    }
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    // our layout generally has a title, and then the rest is allocated to the game.

    let layout = Layout::vertical([
        Constraint::Length(3), // tabs
        Constraint::Min(0),    // rest of the data
        Constraint::Length(1), // informing about controls
    ])
    .split(frame.area());

    let tabs = Tabs::new(app.page.values.clone())
        .block(
            Block::bordered()
                .title("NYT Games CLI")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().blue())
        .highlight_style(Style::default().green())
        .select(app.page.index.into());

    frame.render_widget(tabs, layout[0]);

    // detect if the game is wrodle, connections or strands

    let mut text: Vec<Line> = Vec::new();

    // the part that describes the game is here

    for line in &app.game_string {
        text.push(line.as_str().into());
    }

    for line in &app.lines {
        text.push(line.as_str().into());
    }

    if !app.game_complete {
        text.push(
            format!(
                "GUESS: {}",
                app.guess_buffer
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("")
            )
            .into(),
        );
    }

    match app.page.index {
        0 => {
            frame.render_widget(
                Paragraph::new(text).block(
                    Block::bordered()
                        .title(format!("Wordle on {}", app.date.date_naive()))
                        .title_alignment(Alignment::Left),
                ),
                layout[1],
            );
        }
        1 => {
            frame.render_widget(
                Paragraph::new(text).block(
                    Block::bordered()
                        .title(format!("Connections on {}", app.date.date_naive()))
                        .title_alignment(Alignment::Left),
                ),
                layout[1],
            );
        }
        2 => {
            frame.render_widget(
                Paragraph::new(text).block(
                    Block::bordered()
                        .title(format!("Strands on {}", app.date.date_naive()))
                        .title_alignment(Alignment::Left),
                ),
                layout[1],
            );
        }
        _ => {
            println!("Unknown game!");

            frame.render_widget(Text::from("Unknown Game!"), layout[1]);
        }
    }

    frame.render_widget(
        Text::from("Controls: ~: exit, up/down: change date, left/right: change tab"),
        layout[2],
    );
}
