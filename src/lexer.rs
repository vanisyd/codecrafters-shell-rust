static NORMAL_MODE_ESCAPE_CHARS: [char; 5] = ['\'', '"', '$', '*', '?'];
static DOUBLE_Q_MODE_ESCAPE_CHARS: [char; 4] = ['"', '\\', '$', '`'];

enum LexerMode {
    Normal,
    SingleQuote,
    DoubleQuote,
    Escape,
}

#[derive(Default)]
pub struct ParsedCommand {
    arena: String,
    args: Vec<std::ops::Range<usize>>,
    word_start: Option<usize>, // TODO: move out of state
}

impl ParsedCommand {
    fn push_char(&mut self, c: char) {
        if self.word_start.is_none() {
            self.word_start = Some(self.arena.len());
        }
        self.arena.push(c);
    }

    fn push_arg(&mut self) {
        if let Some(w_start) = self.word_start {
            self.args.push(w_start..self.arena.len());
            self.word_start = None;
        }
        // TODO: handle None value?
    }

    pub fn args(&self) -> impl Iterator<Item = &str> {
        self.args.iter().skip(1).map(|r| &self.arena[r.clone()])
    }

    pub fn cmd(&self) -> Option<&str> {
        self.arena.get(self.args.first().unwrap().clone())
    }
}

fn escape(cmd: &mut ParsedCommand, c: char, mode: LexerMode) {}

pub fn parse_command(string: &str) -> ParsedCommand {
    let mut cmd = ParsedCommand::default();

    let mut mode = LexerMode::Normal;
    let mut prev_mode = LexerMode::Normal;
    for c in string.chars() {
        match mode {
            LexerMode::Escape => {
                if !matches!(prev_mode, LexerMode::Normal)
                    && !(matches!(prev_mode, LexerMode::DoubleQuote) && DOUBLE_Q_MODE_ESCAPE_CHARS.contains(&c))
                {
                    cmd.push_char('\\');
                }
                cmd.push_char(c);
                mode = prev_mode;
                prev_mode = LexerMode::Escape;
            }
            LexerMode::Normal => {
                if c == ' ' {
                    cmd.push_arg();
                } else if c == '\\' {
                    prev_mode = mode;
                    mode = LexerMode::Escape;
                } else if c == '"' {
                    mode = LexerMode::DoubleQuote;
                } else if c == '\'' {
                    mode = LexerMode::SingleQuote;
                } else {
                    cmd.push_char(c);
                }
            }
            LexerMode::SingleQuote => {
                if c == '\'' {
                    mode = LexerMode::Normal;
                } else {
                    cmd.push_char(c);
                }
            }
            LexerMode::DoubleQuote => {
                if c == '"' {
                    mode = LexerMode::Normal;
                } else if c == '\\' {
                    mode = LexerMode::Escape;
                    prev_mode = LexerMode::DoubleQuote;
                } else {
                    cmd.push_char(c);
                }
            }
        }
    }
    cmd.push_arg();

    cmd

    // TODO: Валідація незакритих лапок: якщо mode == SingleQuote || mode == DoubleQuote після циклу → помилка
    // TODO: Escape тільки ", \, $, `
}
