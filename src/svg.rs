//! Minimal SVG path parser for converting outlines to 2D point rings.
//!
//! Handles absolute commands: **M**, **L**, **H**, **V**, **C**, **Z**.
//! Cubic Béziers are flattened to line segments via [`crate::math::flatten_cubic`].
//!
//! ```rust,ignore
//! use iced3d::svg::parse_path;
//!
//! let ring = parse_path("M0 0 L10 0 L10 10 L0 10 Z", 8);
//! assert_eq!(ring.len(), 4);
//! ```

use crate::math::flatten_cubic;

/// Parse an SVG path `d` attribute into a 2D polygon ring.
///
/// Only absolute commands are supported: `M`, `L`, `H`, `V`, `C`, `Z`.
/// Cubic Bézier curves (`C`) are flattened into `curve_segments` line
/// segments each using [`crate::math::flatten_cubic`].
///
/// Returns a `Vec` of `[x, y]` points forming the polygon outline.
#[allow(clippy::many_single_char_names)]
pub fn parse_path(d: &str, curve_segments: usize) -> Vec<[f32; 2]> {
    let tokens = tokenize(d);
    let mut pts = Vec::new();
    let mut x = 0.0_f32;
    let mut y = 0.0_f32;
    let mut cmd = 'M';
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Cmd(c) => {
                cmd = *c;
                i += 1;
            }
            Token::Num(_) => match cmd {
                'M' => {
                    x = expect_num(&tokens[i]);
                    y = expect_num(&tokens[i + 1]);
                    pts.push([x, y]);
                    i += 2;
                    cmd = 'L';
                }
                'L' => {
                    x = expect_num(&tokens[i]);
                    y = expect_num(&tokens[i + 1]);
                    pts.push([x, y]);
                    i += 2;
                }
                'H' => {
                    x = expect_num(&tokens[i]);
                    pts.push([x, y]);
                    i += 1;
                }
                'V' => {
                    y = expect_num(&tokens[i]);
                    pts.push([x, y]);
                    i += 1;
                }
                'C' => {
                    let x1 = expect_num(&tokens[i]);
                    let y1 = expect_num(&tokens[i + 1]);
                    let x2 = expect_num(&tokens[i + 2]);
                    let y2 = expect_num(&tokens[i + 3]);
                    let x3 = expect_num(&tokens[i + 4]);
                    let y3 = expect_num(&tokens[i + 5]);
                    flatten_cubic(
                        [x, y],
                        [x1, y1],
                        [x2, y2],
                        [x3, y3],
                        curve_segments,
                        &mut pts,
                    );
                    x = x3;
                    y = y3;
                    i += 6;
                }
                _ => i += 1,
            },
        }
    }
    pts
}

// ──────────────────── Tokenizer ────────────────────

/// Token from SVG path data.
enum Token {
    Cmd(char),
    Num(f32),
}

/// Tokenize an SVG path `d` attribute into commands and numbers.
fn tokenize(d: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = d.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphabetic() {
            tokens.push(Token::Cmd(c));
            chars.next();
        } else if c == '-' || c == '.' || c.is_ascii_digit() {
            tokens.push(Token::Num(parse_float(&mut chars)));
        } else {
            chars.next(); // skip whitespace / comma
        }
    }
    tokens
}

/// Parse a floating-point number from a character iterator.
fn parse_float(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> f32 {
    let mut s = String::with_capacity(12);
    if chars.peek() == Some(&'-') {
        s.push('-');
        chars.next();
    }
    let mut has_dot = false;
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            s.push(c);
            chars.next();
        } else if c == '.' && !has_dot {
            has_dot = true;
            s.push(c);
            chars.next();
        } else {
            break;
        }
    }
    s.parse().unwrap_or(0.0)
}

/// Extract a number from a token (panics if not a `Num`).
fn expect_num(tok: &Token) -> f32 {
    match tok {
        Token::Num(n) => *n,
        Token::Cmd(c) => panic!("expected number, got command '{c}'"),
    }
}

#[cfg(test)]
#[path = "svg_tests.rs"]
mod tests;
