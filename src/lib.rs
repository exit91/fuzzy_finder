use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use item::{Item, ScoredItem};
use pastel_colours::{
    BLUE_FG, DARK_BLUE_BG, DARK_GREY_BG, DARK_GREY_FG, GREEN_FG, RESET_BG, RESET_FG,
};
use std::io::{stdout, Stdout, Write};
use std::time::Instant;
use termion::clear::CurrentLine;
use termion::cursor::DetectCursorPos;
use termion::cursor::Show;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use view::*;

pub mod item;
pub mod view;

pub struct FuzzyFinder<T>
where
    T: Clone,
{
    search_term: String,
    all_items: Vec<Item<T>>,
    matches: Vec<ScoredItem<T>>,
    console_offset: u16,
    stdout: RawTerminal<Stdout>,
    first: bool,
    view: ScrollingView,
    positive_space_remaining: u16,
}

impl<T> FuzzyFinder<T>
where
    T: Clone,
{
    fn new(functions: Vec<Item<T>>, lines_to_show: i8) -> Self {
        // We need to know where to start rendering from. We can't do this later because
        // we overwrite the cursor. Maybe we shouldn't do this? (TODO)
        let mut stdout = stdout().into_raw_mode().unwrap();

        write!(stdout, "{}", termion::cursor::Save).unwrap();
        let mut positive_space_remaining = 0;
        let console_offset = if stdout.cursor_pos().is_ok() {
            let cursor_pos_y = stdout.cursor_pos().unwrap().1;

            let terminal_height = termion::terminal_size().unwrap().1;
            let starting_y = cursor_pos_y;
            let ending_y = starting_y + lines_to_show as u16;
            let space_remaining: i16 = terminal_height as i16 - ending_y as i16;
            positive_space_remaining = if space_remaining < 0 {
                space_remaining.abs().try_into().unwrap()
            } else {
                0
            };
            cursor_pos_y
        } else {
            log::error!("Cannot get cursor!");
            0
        };

        FuzzyFinder {
            search_term: String::from(""),
            all_items: functions,
            matches: vec![],
            console_offset,
            stdout,
            first: true,
            view: ScrollingView::new(lines_to_show as usize),
            positive_space_remaining,
        }
    }

    pub fn up(&mut self) -> Result<()> {
        self.view.up();
        self.update_matches();
        self.render()
    }

    pub fn down(&mut self) -> Result<()> {
        self.view.down();
        self.update_matches();
        self.render()
    }

    pub fn append(&mut self, c: char) -> Result<()> {
        // This is a normal key that we want to add to the search.
        self.search_term = format!("{}{}", self.search_term, c);

        self.update_matches();
        self.render()
    }

    pub fn backspace(&mut self) -> Result<()> {
        if self.search_term.chars().count() > 0 {
            self.search_term =
                String::from(&self.search_term[..self.search_term.chars().count() - 1]);
        }
        self.update_matches();
        self.render()
    }

    fn render_space(&mut self) -> Result<()> {
        // Drop down so we don't over-write the terminal line that instigated
        // this run of lk.
        write!(self.stdout, "{}", termion::cursor::Save).unwrap();
        if self.first {
            for _ in 0..self.view.capacity {
                writeln!(self.stdout, " ")?;
            }
            self.first = false
        }
        write!(self.stdout, "{}", termion::cursor::Restore).unwrap();

        Ok(())
    }

    fn goto_start(&mut self) -> Result<()> {
        write!(
            self.stdout,
            "{}",
            termion::cursor::Goto(1, self.console_offset - self.positive_space_remaining)
        )?;
        Ok(())
    }

    fn render_items(&mut self) -> Result<()> {
        self.goto_start()?;
        // render blank space
        let list = self.view.render(&self.matches);
        let num_blank = self.view.capacity - list.len();
        for _ in 0..num_blank {
            writeln!(self.stdout, "{}", termion::clear::CurrentLine)?;
        }
        for (is_selected, scored_item) in list {
            let fuzzy_indices = &scored_item.fuzzy_indices;

            // Do some string manipulation to colourise the indexed parts
            let coloured_line =
                get_coloured_line(&fuzzy_indices, &scored_item.item.name, is_selected);

            writeln!(
                self.stdout,
                "{}{}{}",
                termion::clear::CurrentLine,
                // Go maximum left, so we're at the start of the line
                termion::cursor::Left(1000),
                coloured_line
            )?;
        }
        Ok(())
    }

    fn render_prompt(&mut self) -> Result<()> {
        // Render the prompt
        let prompt_y = self.view.capacity as u16 + 1;
        let current_x = self.search_term.chars().count() + 2;

        // Go to the bottom line, where we'll render the prompt
        write!(
            self.stdout,
            "{CurrentLine}{}{CurrentLine}",
            termion::cursor::Goto(current_x as u16, prompt_y + self.console_offset),
        )?;
        write!(
            self.stdout,
            "{Show}{}{BLUE_FG}${RESET_FG} {}",
            termion::cursor::Goto(1, prompt_y + self.console_offset),
            self.search_term,
        )?;
        self.stdout.flush()?;
        Ok(())
    }

    /// Gets functions that match our current criteria, sorted by score.
    pub fn update_matches(&mut self) {
        self.matches.clear();
        let matcher = SkimMatcherV2::default();
        self.matches.extend(self.all_items.iter().flat_map(|f| {
            let (score, positions) = matcher.fuzzy_indices(&f.name, &self.search_term)?;
            Some(f.clone().with_score(score, positions))
        }));

        log::info!(
            "There are a total of {} item(s) and {} match(es)",
            self.all_items.len(),
            self.matches.len()
        );

        // We want these in the order of their fuzzy matched score, i.e. closed matches
        self.matches.sort_by(|a, b| b.score.cmp(&a.score));
    }

    /// Renders the current result set
    pub fn render(&mut self) -> Result<()> {
        self.render_space()?;
        self.render_items()?;
        self.render_prompt()?;
        Ok(())
    }

    /// The main entry point for the fuzzy finder.
    pub fn find(items: Vec<Item<T>>, lines_to_show: i8) -> Result<Option<T>> {
        let mut state = FuzzyFinder::new(items, lines_to_show);

        state.update_matches();

        state.render()?;

        let mut stdin = termion::async_stdin().keys();

        // Run 'sed -n l' to explore escape codes
        let mut escaped = String::from("");
        let mut instant = Instant::now();

        loop {
            // What's going on here? The problem is how we detect escape.
            // The key presses we're interested in, e.g. the arrows, are all preceded by escape, ^[.
            // E.g. up is ^[[A and down is ^[[B. So the question is how do we identify an escape
            // key by itself? If it's ^[[A then that's ^[ followed almost instantly by [A. If we have
            // ^[ followed by a pause then we know it's not an escape for some other key, but an
            // escape by itself. That's what the 100 136His below.
            // NB: some terminals might send these bytes too slowly and escape might not be caught.
            // NB: some terminals might use different escape keys entirely.
            if escaped == "^[" && instant.elapsed().as_micros() > 100 {
                write!(state.stdout, "{}", termion::cursor::Restore)?;
                break;
            }

            if let Some(Ok(key)) = stdin.next() {
                match key {
                    // ctrl-c and ctrl-d are two ways to exit.
                    Key::Ctrl('c') => break,
                    Key::Ctrl('d') => break,

                    // NB: It'd be neat if we could use Key::Up and Key::Down but they don't
                    // work in raw mode. So we've got to deal with the escape codes manually.

                    // This captures the enter key
                    Key::Char('\n') => {
                        return if !state.matches.is_empty() {
                            // Tidy up the console lines we've been writing
                            for _ in state.console_offset
                                ..state.console_offset + state.view.capacity as u16 + 4
                            {
                                write!(state.stdout, "{}", termion::clear::CurrentLine,)?;
                            }
                            Ok(state
                                .view
                                .render(&state.matches)
                                .selected()
                                .map(|f| f.item.data.to_owned()))
                        } else {
                            Ok(None)
                        };
                    }
                    Key::Char(c) => {
                        if !escaped.is_empty() {
                            escaped = format!("{}{}", escaped, c);
                            match escaped.as_str() {
                                "^[" => continue,
                                "^[[" => continue,
                                "^[[A" => {
                                    escaped = String::from("");
                                    state.up()?;
                                }
                                "^[[B" => {
                                    escaped = String::from("");
                                    state.down()?;
                                }
                                _ => {
                                    // This is nothing we recognise so let's abandon the escape sequence.
                                    escaped = String::from("");
                                }
                            }
                        } else {
                            state.append(c)?;
                        }
                    }
                    Key::Esc => {
                        // All we're doing here is recording that we've entered an escape sequence.
                        // It's actually handled when we handle chars.
                        if escaped.is_empty() {
                            escaped = String::from("^[");
                            instant = Instant::now();
                        }
                    }
                    Key::Backspace => {
                        state.backspace()?;
                    }
                    _ => {}
                }
                state.stdout.flush().unwrap();
            }
        }
        Ok(None)
    }
}

/// Highlights the line. Will highlight matching search items, and also indicate
/// if it's a selected item.
fn get_coloured_line(fuzzy_indecies: &[usize], text: &str, is_selected: bool) -> String {
    // Do some string manipulation to colourise the indexed parts
    let mut coloured_line = String::from("");
    let mut start = 0;

    let text_vec = text.chars().collect::<Vec<_>>();
    for i in fuzzy_indecies {
        let part = &text_vec[start..*i].iter().cloned().collect::<String>();
        let matching_char = &text_vec[*i..*i + 1].iter().cloned().collect::<String>();
        if is_selected {
            coloured_line = format!(
                "{coloured_line}{DARK_GREY_BG}{part}{RESET_BG}{DARK_BLUE_BG}{matching_char}{RESET_BG}"
            );
        } else {
            coloured_line = format!("{coloured_line}{part}{DARK_BLUE_BG}{matching_char}{RESET_BG}");
        }
        start = i + 1;
    }
    let remaining_chars = &text_vec[start..text.chars().count()]
        .iter()
        .cloned()
        .collect::<String>();
    if is_selected {
        let prompt: String = format!("{DARK_GREY_BG}{GREEN_FG}>{RESET_FG}{RESET_BG}",);
        let spacer: String = format!("{DARK_GREY_FG}  {RESET_FG}");
        let remaining: String = format!("{DARK_GREY_BG}{remaining_chars}{RESET_BG}");
        coloured_line = format!("{prompt}{spacer}{coloured_line}{remaining}");
    } else {
        coloured_line = format!("{DARK_GREY_BG} {RESET_BG}  {coloured_line}{remaining_chars}");
    }
    coloured_line
}
