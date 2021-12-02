/// Given a source string and an index, returns its (line, column) numbers in
/// the text editor standard (index starting from 1).
///
/// *Shamelessly copied from https://stackoverflow.com/a/66443805.*
pub(crate) fn index_to_pos(src: &str, idx: usize) -> (usize, usize) {
    src[..idx + 1]
        .lines()
        .enumerate()
        .last()
        .map_or((1, 1), |(i, ln)| (i + 1, ln.len()))
}
