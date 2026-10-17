use std::fmt::Display;

use crate::{
    error::CompilerError,
    source_map::{SourceFile, SourceMap},
};

const RED: &str = "\x1b[1;31m";
const BLUE: &str = "\x1b[1;34m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

pub struct Style {
    color: bool,
}

impl Style {
    pub fn new(color: bool) -> Self {
        Self { color }
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("{code}{text}{RESET}")
        } else {
            text.to_string()
        }
    }

    pub fn message(&self, message: impl Display) -> String {
        self.header("error", message)
    }

    fn header(&self, label: &str, message: impl Display) -> String {
        format!(
            "{}{}",
            self.paint(RED, label),
            self.paint(BOLD, &format!(": {message}"))
        )
    }

    pub fn render(&self, error: &CompilerError, sources: &SourceMap) -> String {
        let SourceFile { path, text: source } = sources.get(error.span.file);
        let line_index = error.span.line as usize;
        let col = error.span.col as usize;
        let line = source.lines().nth(line_index).unwrap_or("");
        let line_number = (line_index + 1).to_string();
        let gutter = " ".repeat(line_number.len());

        let padding: String = line
            .chars()
            .take(col)
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();
        let underline = "^".repeat(underline_len(line, col));

        format!(
            "{}\n{gutter}{} {}:{}\n{gutter} {}\n{} {} {line}\n{gutter} {} {padding}{}\n",
            self.header(&format!("error[{}]", error.kind.stage()), &error.kind),
            self.paint(BLUE, "-->"),
            path.display(),
            error.span,
            self.paint(BLUE, "|"),
            self.paint(BLUE, &line_number),
            self.paint(BLUE, "|"),
            self.paint(BLUE, "|"),
            self.paint(RED, &underline),
        )
    }
}

/// Spans only mark where something starts, so underline the word there, or one character
fn underline_len(line: &str, col: usize) -> usize {
    let is_word = |c: &char| c.is_alphanumeric() || *c == '_';
    line.chars().skip(col).take_while(is_word).count().max(1)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::{
        diagnostic::Style,
        error::{At, CompilerError},
        source_map::SourceMap,
        type_check::{Type, TypeCheckErrorKind},
        types::{IntType, Span},
    };

    fn render(source: &str, line: u32, col: u32) -> String {
        let error: CompilerError = TypeCheckErrorKind::IdentifierNotFound("foo".to_string())
            .at(Span::new(0, line, col))
            .into();
        Style::new(false).render(&error, &sources(source))
    }

    fn sources(source: &str) -> SourceMap {
        let mut sources = SourceMap::default();
        sources.add(PathBuf::from("main.tan"), source.to_string());
        sources
    }

    #[test]
    fn renders_source_line_with_caret() {
        assert_eq!(
            render("let a = 1;\nlet b = foo;", 1, 8),
            "error[type check]: Identifer: (foo) not found\n \
             --> main.tan:2:9\n  \
             |\n\
             2 | let b = foo;\n  \
             |         ^^^\n"
        );
    }

    #[test]
    fn underlines_one_character_for_symbols() {
        assert!(render("a = 1 $ 2;", 0, 6).ends_with("|       ^\n"));
    }

    #[test]
    fn widens_gutter_for_large_line_numbers() {
        let source = "x;\n".repeat(11);
        assert!(render(&source, 10, 0).contains("\n11 | x;\n   | ^\n"));
    }

    #[test]
    fn keeps_tabs_aligned() {
        assert!(render("\tfoo;", 0, 1).ends_with("| \t^^^\n"));
    }

    #[test]
    fn handles_span_past_end_of_source() {
        let error: CompilerError = TypeCheckErrorKind::TypeMismatch {
            expected: Type::Int(IntType::I32),
            got: Type::Boolean,
        }
        .at(Span::new(0, 5, 0))
        .into();
        Style::new(false).render(&error, &sources("a;"));
    }
}
