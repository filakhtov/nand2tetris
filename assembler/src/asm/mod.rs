use std::{collections, error, ffi, fs, io, iter, path};

#[derive(Debug)]
pub enum Error {
    LoadSource(io::Error),
    WriteOutput(io::Error),
    Parse {
        line_no: usize,
        source: String,
        cause: String,
    },
    SourceTooLong(usize),
}

impl Error {
    pub fn cause(&self) -> String {
        match self {
            Self::LoadSource(e) => {
                format!("Failed to read the input source file: {}", trace_error(e))
            }
            Self::WriteOutput(e) => {
                format!("Failed to write the output file: {}", trace_error(e))
            }
            Self::Parse {
                line_no,
                source,
                cause,
            } => {
                format!("Error on line {line_no}: {cause}:\n{source}")
            }
            Self::SourceTooLong(len) => {
                format!(
                    "The source code is too long, maximum number of lines \
                    must be less or equal to {ROM_SIZE}, got {len}",
                )
            }
        }
    }
}

fn trace_error(err: impl error::Error) -> String {
    match err.source() {
        Some(prev) => format!("{err}: {}", trace_error(prev)),
        None => format!("{err}"),
    }
}

pub fn compile(path: &path::Path) -> Result<path::PathBuf, Error> {
    let source = fs::read_to_string(path).map_err(Error::LoadSource)?;

    let binary = parse(&source)?;

    let output = path::PathBuf::from(
        path.with_extension("hack")
            .file_name()
            .unwrap_or(&ffi::OsString::from("out.hack")),
    );

    fs::write(&output, binary).map_err(Error::WriteOutput)?;

    Ok(output)
}

// Symbols a-z A-Z 0-9 _ . $ : but does not begin with a digit
// A instructions 0-32767
// Predefined: R0..R15, SP=0, LCL=1, ARG=2, THIS=3, THAT=4, SCREEN=16384, KBD=24576
// Dest: M, D, A, DM, A, AM, AD, ADM
enum Instruction {
    A(u16),
    L(u32),
    C(u16),
}

struct SymbolTable {
    symbols: collections::HashMap<String, u16>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        let symbols = collections::HashMap::from([
            ("R0".to_string(), 0x0),
            ("R1".to_string(), 0x1),
            ("R2".to_string(), 0x2),
            ("R3".to_string(), 0x3),
            ("R4".to_string(), 0x4),
            ("R5".to_string(), 0x5),
            ("R6".to_string(), 0x6),
            ("R7".to_string(), 0x7),
            ("R8".to_string(), 0x8),
            ("R9".to_string(), 0x9),
            ("R10".to_string(), 0xa),
            ("R11".to_string(), 0xb),
            ("R12".to_string(), 0xc),
            ("R13".to_string(), 0xd),
            ("R14".to_string(), 0xe),
            ("R15".to_string(), 0xf),
            ("SP".to_string(), 0),
            ("LCL".to_string(), 1),
            ("ARG".to_string(), 2),
            ("THIS".to_string(), 3),
            ("THAT".to_string(), 4),
            ("SCREEN".to_string(), 0x4000),
            ("KBD".to_string(), 0x6000),
        ]);

        Self { symbols }
    }
}

impl SymbolTable {
    fn add_entry(mut self, label: Label, value: u16) -> Self {
        self.symbols.entry(label.0).or_insert(value);

        self
    }

    fn contains(&self, label: &Label) -> bool {
        self.symbols.get(&label.0).is_some()
    }
}

struct Label(String);

impl TryFrom<&str> for Label {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, String> {
        if value.is_empty() {
            Err("Empty symbol".to_string())
        } else if value.chars().enumerate().all(is_valid_label_char) {
            Ok(Self(value.to_string()))
        } else {
            Err(format!("Invalid symbol: `{value}`"))
        }
    }
}

#[inline(always)]
fn is_valid_label_char((idx, chr): (usize, char)) -> bool {
    chr.is_ascii_alphabetic()
        || chr == '$'
        || chr == '_'
        || chr == '.'
        || (idx == 0 && chr.is_ascii_digit())
}

type Source<'a> = Vec<(usize, usize, &'a str)>;

const ROM_SIZE: usize = 32767;

/*
enum ParserState {
    Normal(usize, usize),
    Comment,
    MultiLineComment(usize),
    CommentStart(usize, usize),
    CommentEnd(usize),
}

fn normalize_source(source: &str) -> Result<Source, Error> {
    let mut line_no = 1;
    let mut line_offset = 0;
    let mut state = ParserState::Normal(0, 0);
    let mut output = vec![];

    for (char_no, sym) in source.chars().enumerate() {
        let char_no = char_no + 1;

        let (new_state, symbol) = match state {
            ParserState::CommentStart(start, end) => match sym {
                '/' => (ParserState::Comment, Some((start, end))),
                '*' => (
                    ParserState::MultiLineComment(char_no - 2),
                    Some((start, end)),
                ),
                _ => (ParserState::Normal(start, char_no), None),
            },
            c @ ParserState::Comment => match sym {
                '\n' => (ParserState::Normal(char_no, char_no), None),
                _ => (c, None),
            },
            c @ ParserState::MultiLineComment(start) => match sym {
                '*' => (ParserState::CommentEnd(start), None),
                _ => (c, None),
            },
            c @ ParserState::CommentEnd(start) => match sym {
                '/' => (ParserState::Normal(char_no, char_no), None),
                '*' => (c, None),
                _ => (ParserState::MultiLineComment(start), None),
            },
            ParserState::Normal(start, end) => match sym {
                '/' => (ParserState::CommentStart(start, end), None),
                c if c.is_whitespace() => {
                    (ParserState::Normal(char_no, char_no), Some((start, end)))
                }
                _ => (ParserState::Normal(start, char_no), None),
            },
        };

        match symbol {
            Some((start, end)) if end > start => {
                output.push((line_no, start + 1 - line_offset, &source[start..end]));
            }
            _ => {
                if char_no >= source.len() {
                    match new_state {
                        ParserState::Normal(start, end)
                            if char_no == source.len() && end > start =>
                        {
                            output.push((line_no, start - line_offset + 1, &source[start..end]));
                        }
                        ParserState::CommentStart(start, end) => {}
                        ParserState::MultiLineComment(start_at) => {
                            return Err(Error::Parse {
                                line_no,
                                source: source[start_at..].to_string(),
                                cause: "Unterminated comment".to_string(),
                            })
                        }
                        _ => {}
                    }
                }
            }
        }

        if sym == '\n' {
            line_no += 1;
            line_offset = char_no;
        }

        state = new_state;
    }

    Ok(output)
}
*/

enum ParserState {
    Whitespace,
    Symbol {
        offset: usize,
        len: usize,
        line_no: usize,
        line_offset: usize,
    },
    Comment,
    MultiLineComment {
        line_no: usize,
        line_offset: usize,
        sym: Option<char>,
    },
}

fn normalize_source(source: &str) -> Result<Source, Error> {
    let mut state = ParserState::Symbol {
        offset: 0,
        len: 0,
        line_no: 1,
        line_offset: 1,
    };
    let mut line_no = 1;
    let mut line_start = 0;
    let mut parsed_source = vec![];

    for (source_offset, ch) in source.chars().chain(iter::once('\n')).enumerate() {
        let (new_state, parsed_symbol) = match state {
            ps @ ParserState::Whitespace => match ch {
                s if s.is_whitespace() => (ps, None),
                _ => (
                    ParserState::Symbol {
                        offset: source_offset,
                        len: 1,
                        line_no,
                        line_offset: source_offset - line_start,
                    },
                    None,
                ),
            },
            ParserState::Symbol {
                offset,
                len,
                line_no,
                line_offset,
            } => {
                let parsed_symbol = if len > 0 {
                    Some((line_no, line_offset, &source[offset..offset + len]))
                } else {
                    None
                };
                match ch {
                    s if s.is_whitespace() => (ParserState::Whitespace, parsed_symbol),
                    '/' if len > 0 && &source[offset + len - 1..source_offset] == "/" => {
                        let parsed_symbol = if len > 1 {
                            Some((line_no, line_offset, &source[offset..offset + len - 1]))
                        } else {
                            None
                        };

                        (ParserState::Comment, parsed_symbol)
                    }
                    '*' if &source[offset + len - 1..source_offset] == "/" => {
                        let parsed_symbol = if len > 1 {
                            Some((line_no, line_offset, &source[offset..offset + len - 1]))
                        } else {
                            None
                        };
                        (
                            ParserState::MultiLineComment {
                                line_no,
                                line_offset: line_start + 1,
                                sym: None,
                            },
                            parsed_symbol,
                        )
                    }
                    _ => (
                        ParserState::Symbol {
                            offset,
                            len: len + 1,
                            line_no,
                            line_offset,
                        },
                        None,
                    ),
                }
            }
            ps @ ParserState::Comment => match ch {
                '\n' => (ParserState::Whitespace, None),
                _ => (ps, None),
            },
            ParserState::MultiLineComment {
                line_no,
                line_offset,
                sym,
            } => match ch {
                '/' if sym == Some('*') => {
                    let offset = source_offset + 1;
                    (
                        ParserState::Symbol {
                            offset,
                            len: 0,
                            line_no,
                            line_offset: offset - line_start,
                        },
                        None,
                    )
                }
                _ => (
                    ParserState::MultiLineComment {
                        line_no,
                        line_offset,
                        sym: Some(ch),
                    },
                    None,
                ),
            },
        };

        if let Some(symbol) = parsed_symbol {
            parsed_source.push(symbol);
        }

        if ch == '\n' {
            line_no += 1;
            line_start = source_offset;
        }

        state = new_state;
    }

    if let ParserState::MultiLineComment {
        line_no,
        line_offset,
        sym: _,
    } = state
    {
        Err(Error::Parse {
            line_no,
            source: source[line_offset..].to_string(),
            cause: "Unterminated multiline comment".to_string(),
        })
    } else {
        Ok(parsed_source)
    }
}

#[inline(always)]
fn try_label(line: &str) -> (Option<char>, Option<char>, &str) {
    let mut chars = line.chars();

    (chars.next(), chars.next_back(), chars.as_str())
}

/*
fn resolve_labels(
    mut symbol_table: SymbolTable,
    source: Source,
) -> Result<(SymbolTable, Source), Error> {
    let mut labeless_source = Vec::with_capacity(source.len());

    for (line_no, line) in source.into_iter() {
        match try_label(line) {
            (Some('('), Some(')'), label) => {
                let label = label.try_into().map_err(|err| Error::Parse {
                    line_no,
                    source: line.to_string(),
                    cause: err,
                })?;

                if symbol_table.contains(&label) {
                    return Err(Error::Parse {
                        line_no,
                        source: line.to_string(),
                        cause: format!("Duplicate label `{}`", label.0),
                    });
                }

                let addr = labeless_source.len();
                if addr > ROM_SIZE {
                    return Err(Error::Parse {
                        line_no,
                        source: line.to_string(),
                        cause: format!(
                            "The label `{}` is pointing to the address beyond the max ROM address",
                            label.0
                        ),
                    });
                }

                symbol_table = symbol_table.add_entry(label, addr as u16);
            }
            _ => labeless_source.push((line_no, line)),
        }
    }

    Ok((symbol_table, labeless_source))
}
*/

fn parse(source: &str) -> Result<String, Error> {
    let symbol_table = SymbolTable::default();

    let source = normalize_source(source)?;
    // let (symbol_table, source) = resolve_labels(symbol_table, source)?;
    let source = source
        .into_iter()
        .map(|(l, c, s)| format!("{s} // {l}:{c}"))
        .collect::<Vec<_>>()
        .join("\n");
    println!("{source}");

    // Ensure that the source code is not
    if source.len() > ROM_SIZE {
        return Err(Error::SourceTooLong(source.len()));
    }

    // let instructions = generate_instructions(symbol_table, source)?;

    Ok("".to_string())
}

/*
fn generate_instructions(
    mut symbol_table: SymbolTable,
    source: Source,
) -> Result<Vec<Instruction>, Error> {
    for (line_no, line) in source {
        println!("{line_no} {line}");
    }

    for (symbol, addr) in symbol_table.symbols {
        println!("{} -> {}", symbol, addr);
    }

    Ok(vec![])
}
*/

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parse_address_instruction() {
        let source = "@100
/";

        let binary = parse(source);

        assert_eq!("0000000001100100", binary.unwrap());
    }

    #[test]
    fn parse_comments() {
        let source = "// This is a comment
            @200 // This is also a comment
            (LABEL) // FOO /* This shouldn't start a multi-line comment
            D=A//Comment with no whitespace separator from the instruction
              // This is a tricky comment with some whitespace before it
            M=1\t//This is a comment after a tab
            /* Some comments
             * are very tricky*/
            @LABEL/*But they still
            should be handled properly*/
            0;JMP
            /*This is*/A=1/*a really*/@foo/*crazy
            situation*/M;JEQ//but still should be fine
        ";

        let binary = parse(source);

        assert_eq!("0000000000000000", binary.unwrap());
    }

    #[test]
    fn parse_labels() {
        let source = "@100
@foo 0;JEQ
M=1
@NEXT
0;JMP";

        let binary = parse(source);

        assert_eq!("0000000000000000", binary.unwrap());
    }

    #[test]
    fn parse_screen() {
        let source = "@300
/*this is unfinished comment";

        let binary = parse(source);

        assert_eq!("0000000000000000", binary.unwrap());
    }

    #[test]
    fn parse_keyboard() {
        let source = "@100
M=1/
@NE/XT
/
0;JMP";

        let binary = parse(source);

        assert_eq!("0000000000000000", binary.unwrap());
    }

    #[test]
    fn parse_jump_equal() {
        let source = "/*foo*/
@100 ";

        let binary = parse(source);

        assert_eq!("0000000000000000", binary.unwrap());
    }

    #[test]
    fn parse_jump_unconditional() {
        let source = "@100/* This is";

        let binary = parse(source);

        assert_eq!("0000000000000000", binary.unwrap());
    }

    #[test]
    fn parse_jump_less_than() {}

    #[test]
    fn parse_jump_less_than_or_equal() {}

    #[test]
    fn parse_jump_greater_than() {}

    #[test]
    fn parse_jump_greater_than_or_equal() {}

    #[test]
    fn parse_jump_not_equal() {}
}
