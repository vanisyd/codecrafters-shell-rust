use crate::helper;
use std::io::{Stdout, Write};
use std::iter::Peekable;
use std::path::Path;
use std::str::CharIndices;
use std::{fs, io};

static NORMAL_MODE_ESCAPE_CHARS: [char; 5] = [CH_SINGLE_QUOTE, CH_DOUBLE_QUOTE, '$', '*', '?'];
static DOUBLE_Q_MODE_ESCAPE_CHARS: [char; 4] = [CH_DOUBLE_QUOTE, CH_ESCAPE, '$', '`'];

const CH_REDIRECT: char = '>';
const CH_ESCAPE: char = '\\';
const CH_DOUBLE_QUOTE: char = '"';
const CH_SINGLE_QUOTE: char = '\'';
const CH_SPACE: char = ' ';
const CH_REDIRECT_PREFIX: char = '1';

#[derive(Copy, Clone)]
#[repr(u8)]
enum LexerMode {
    Normal = 0x01,
    SingleQuote = 0x02,
    DoubleQuote = 0x03,
    Escape = 0x04,
}

pub enum OutputWriter {
    Stdout(io::Stdout),
    File(fs::File),
}

#[derive(Default, Debug)]
pub enum Output {
    #[default]
    Default,
    Redirect {
        dest: String,
        rewrite: bool,
    },
}

impl Write for OutputWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            OutputWriter::Stdout(s) => s.write(buf),
            OutputWriter::File(f) => f.write(buf),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            OutputWriter::Stdout(s) => s.flush(),
            OutputWriter::File(f) => f.flush(),
        }
    }
}

impl Output {
    pub fn as_writer(&self) -> Result<OutputWriter, String> {
        let writer = match self {
            Output::Default => OutputWriter::Stdout(io::stdout()),
            Output::Redirect { dest, rewrite } => OutputWriter::File(
                helper::file_into_output(Path::new(&dest), *rewrite)
                    .map_err(|e| format!("{}: {}", dest, e))?,
            ),
        };

        Ok(writer)
    }
}

#[derive(Default, Debug)]
pub struct ParsedCommand {
    arena: String,
    args: Vec<std::ops::Range<usize>>,
    output: Output,
    error_output: Output,
}

impl ParsedCommand {
    pub fn args(&self) -> impl Iterator<Item = &str> {
        self.args.iter().skip(1).map(|r| &self.arena[r.clone()])
    }

    pub fn cmd(&self) -> Option<&str> {
        self.arena.get(self.args.first().unwrap().clone())
    }

    pub fn output(&self) -> (&Output, &Output) {
        (&self.output, &self.error_output)
    }
}

pub struct Parser<'a> {
    input: &'a str,
    iter: Peekable<CharIndices<'a>>,
    mode: LexerMode,
    _output: Option<Output>,
    _word_start: Option<usize>
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            iter: input.char_indices().peekable(),
            mode: LexerMode::Normal,
            _output: None,
            _word_start: None,
        }
    }

    pub fn parse_command(&mut self) -> ParsedCommand {
        let mut cmd = ParsedCommand::default();
        self.parse(&mut cmd.arena, Some(&mut cmd.args));
        cmd.output = self._output.take().unwrap_or_else(|| Output::Default);

        cmd
    }

    fn parse(&mut self, arena: &mut String, mut args: Option<&mut Vec<std::ops::Range<usize>>>) {
        let mut prev_mode: Option<LexerMode> = None;
        while let Some((_, c)) = self.iter.next() {
            let end_of_token = match self.mode {
                LexerMode::Normal => match c {
                    CH_SPACE => true,
                    CH_ESCAPE => {
                        self.start_token(arena);
                        prev_mode = Some(self.mode);
                        self.mode = LexerMode::Escape;
                        false
                    }
                    CH_DOUBLE_QUOTE => {
                        self.start_token(arena);
                        self.mode = LexerMode::DoubleQuote;
                        false
                    }
                    CH_SINGLE_QUOTE => {
                        self.start_token(arena);
                        self.mode = LexerMode::SingleQuote;
                        false
                    }
                    CH_REDIRECT => {
                        self.start_token(arena);
                        let mut rewrite = true;
                        while let Some((_, next_c)) = self.iter.peek() {
                            if next_c.is_whitespace() {
                                self.iter.next();
                                continue;
                            } else if next_c.eq_ignore_ascii_case(&CH_REDIRECT) {
                                rewrite = false;
                                self.iter.next();
                                continue;
                            } else {
                                break;
                            }
                        }

                        let mut dest = String::with_capacity(
                            (self._word_start.take().unwrap_or(0)..self.input.len()).len(),
                        );
                        self.parse(&mut dest, None);
                        self._output = Some(Output::Redirect { dest, rewrite });

                        false
                    }
                    CH_REDIRECT_PREFIX => {
                        if let Some((_, next_c)) = self.iter.peek() {
                            if !next_c.eq_ignore_ascii_case(&CH_REDIRECT) {
                                self.push_char(arena, c);
                            }
                        } else {
                            self.push_char(arena, c);
                        }

                        false
                    }
                    _ => {
                        self.push_char(arena, c);
                        false
                    }
                },
                LexerMode::Escape => {

                    if !matches!(prev_mode, Some(LexerMode::Normal))
                        && !(matches!(prev_mode, Some(LexerMode::DoubleQuote))
                            && DOUBLE_Q_MODE_ESCAPE_CHARS.contains(&c))
                    {
                        self.push_char(arena, c);
                    }
                    self.mode = prev_mode.unwrap_or(LexerMode::Normal);
                    prev_mode = None;
                    false
                }
                LexerMode::SingleQuote => {
                    if c == CH_SINGLE_QUOTE {
                        self.mode = LexerMode::Normal;
                    } else {
                        self.push_char(arena, c);
                    }
                    false
                }
                LexerMode::DoubleQuote => {
                    if c == CH_DOUBLE_QUOTE {
                        self.mode = LexerMode::Normal;
                    } else if c == CH_ESCAPE {
                        self.mode = LexerMode::Escape;
                        prev_mode = Some(LexerMode::DoubleQuote);
                    } else {
                        self.push_char(arena, c);
                    }
                    false
                }
            };

            if end_of_token && let Some(ref mut args) = args {
                if let Some(ws) = self._word_start {
                    args.push((ws..arena.len()));
                    self.end_token();
                }
            }
        }

        if let Some(ref mut args) = args
            && let Some(word_start) = self._word_start
        {
            args.push(word_start..arena.len());
        }
    }

    fn push_char(&mut self, arena: &mut String, c: char) {
        self.start_token(arena);
        arena.push(c);
    }

    fn start_token(&mut self, arena: &String) {
        if self._word_start.is_none() {
            self._word_start = Some(arena.len());
        }
    }

    fn end_token(&mut self) {
        self._word_start = None;
    }
}
