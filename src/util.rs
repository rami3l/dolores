use std::fmt::Display;

use itertools::Itertools;

/// Given a source string and an index, returns its (line, column) numbers in
/// the text editor standard (index starting from 1).
///
/// *Shamelessly copied from <https://stackoverflow.com/a/66443805.>*
pub(crate) fn index_to_pos(src: &str, idx: usize) -> (usize, usize) {
    src[..=idx]
        .lines()
        .enumerate()
        .last()
        .map_or((1, 1), |(i, ln)| (i + 1, ln.len()))
}

pub(crate) fn disp_slice(xs: &[impl Display]) -> String {
    xs.iter().map(|x| format!("{}", x)).join(" ")
}
