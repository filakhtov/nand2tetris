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
    Normal,
    MultiLineComment,
    AfterMultiLineComment,
}

fn strip_comments(line: &str, state: ParserState) -> Result<(&str, ParserState), Error> {
    match state {
        ParserState::Normal => {
            let line = if let Some((code, _)) = line.split_once("//") {
                code
            } else {
                line
            };

            if let Some((code, rest)) = line.split_once("/*") {
                let (_, state) = strip_comments(rest, ParserState::MultiLineComment)?;

                Ok((code, state))
            } else {
                Ok((line, state))
            }
        }
        ParserState::MultiLineComment => {
            if let Some((_, rest)) = line.split_once("*/
") {
                strip_comments(rest, ParserState::AfterMultiLineComment)
            } else {
                Ok(("", ParserState::MultiLineComment))
            }
        }
        ParserState::AfterMultiLineComment => todo!(),
    }
}
*/

enum ParserState {
    Normal(usize, usize),
    Comment,
    MultiLineComment,
    CommentStart(usize, usize),
    CommentEnd,
}

fn normalize_source(source: &str) -> Source {
    let mut line_no = 1;
    let mut line_offset = 0;
    let mut state = ParserState::Normal(0, 0);
    let mut output = vec![];

    for (char_no, sym) in source.chars().chain(iter::once('\n')).enumerate() {
        let char_no = char_no + 1;

        let (new_state, symbol) = match state {
            ParserState::CommentStart(start, end) => match sym {
                '/' => (ParserState::Comment, Some((start, end))),
                '*' => (ParserState::MultiLineComment, Some((start, end))),
                _ => (ParserState::Normal(start, char_no), None),
            },
            c @ ParserState::Comment => match sym {
                '\n' => (ParserState::Normal(char_no, char_no), None),
                _ => (c, None),
            },
            c @ ParserState::MultiLineComment => match sym {
                '*' => (ParserState::CommentEnd, None),
                _ => (c, None),
            },
            c @ ParserState::CommentEnd => match sym {
                '/' => (ParserState::Normal(char_no, char_no), None),
                '*' => (c, None),
                _ => (ParserState::MultiLineComment, None),
            },
            ParserState::Normal(start, end) => match sym {
                '/' => (ParserState::CommentStart(start, end), None),
                '\n' => (ParserState::Normal(char_no, char_no), Some((start, end))),
                c if c.is_whitespace() => {
                    (ParserState::Normal(char_no, char_no), Some((start, end)))
                }
                _ => (ParserState::Normal(start, char_no), None),
            },
        };

        match symbol {
            Some((start, end)) if end > start => {
                output.push((line_no, start - line_offset + 1, &source[start..end]));
            }
            _ => {}
        }

        if sym == '\n' {
            line_no += 1;
            line_offset = char_no;
        }

        state = new_state;
    }

    output
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

    let source = normalize_source(source);
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
        let source = "@100";

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
        let s = "hello".split_once("//");
        assert_eq!(s, Some(("???", "hello")));
    }

    #[test]
    fn parse_screen() {}

    #[test]
    fn parse_keyboard() {}

    #[test]
    fn parse_jump_equal() {}

    #[test]
    fn parse_jump_unconditional() {}

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
