//! Indent guidelines support in Kakoune.

use std::fmt::{Display, Formatter, Write as _};

const INDENT_GUIDELINE_CHAR: char = '│';

/// Indent guidelines for a whole buffer.
///
/// The guidelines are sorted by lines and optimized to generate a single
/// highlighter item with spaces for several guidelines annotation on the
/// same line, instead of having several highlight items.
///
/// This is not a contiguous array of indent guidelines, as
#[derive(Debug)]
pub struct IndentGuidelines {
  lines: Vec<IndentGuideline>,
}

impl IndentGuidelines {
  pub fn new(lines: impl Into<Vec<IndentGuideline>>) -> Self {
    Self {
      lines: lines.into(),
    }
  }

  /// Display as a string recognized by the `ranges` or `replace-ranges` Kakoune
  /// highlighters.
  pub fn to_kak_replace_replace_ranges_str(&self, f: &mut Formatter) {
    self.to_kak_replace_hl_str(f, &INDENT_GUIDELINE_CHAR);
  }

  pub fn to_kak_ranges_str(&self, f: &mut Formatter) {
    self.to_kak_replace_hl_str(f, &"ts_indent_guideline");
  }

  fn to_kak_replace_hl_str(&self, f: &mut Formatter, s: &impl Display) {
    for (line1, line2) in self.lines.iter().zip(self.lines.iter().skip(1)) {
      // display the first line + gaps if any
      for line in line1.line..line2.line {
        for col in &line1.cols {
          write!(f, "{line}.{col}+1|{s} ").unwrap();
        }
      }

      // second line
      let line = line2.line;
      for col in &line2.cols {
        write!(f, "{line}.{col}+1|{s} ").unwrap();
      }
    }
  }
}

/// Indent guideline for a given line.
#[derive(Debug)]
pub struct IndentGuideline {
  line: usize,
  cols: Vec<usize>,
}

impl IndentGuideline {
  pub fn new(line: usize, cols: Vec<usize>) -> Self {
    Self { line, cols }
  }
}
