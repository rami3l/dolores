use std::fmt::Display;

pub(crate) fn report(pos: (usize, usize), ctx: &str, msg: impl Display) -> String {
    format!("[L{}:{}] Error {}: {}", pos.0, pos.1, ctx, msg)
}

pub(crate) fn runtime_report(pos: (usize, usize), ctx: &str, msg: impl Display) -> String {
    format!("[L{}:{}] Runtime Error {}: {}", pos.0, pos.1, ctx, msg)
}

pub(crate) fn semantic_report(pos: (usize, usize), ctx: &str, msg: impl Display) -> String {
    format!("[L{}:{}] Semantic Error {}: {}", pos.0, pos.1, ctx, msg)
}

#[macro_export]
macro_rules! bail {
    ($pos:expr, $ctx:expr, $msg:expr $(,)?) => {
        anyhow::bail!("{}", $crate::error::report($pos, $ctx, $msg))
    };
    ($pos:expr, $ctx:expr, $msg:expr, $( $arg:expr ),+ $(,)?) => {
        anyhow::bail!("{}", $crate::error::report(
            $pos,
            $ctx,
            format!($msg, $( $arg ),+),
        ))
    };
}

#[macro_export]
macro_rules! runtime_bail {
    ($pos:expr, $ctx:expr, $msg:expr $(,)?) => {
        anyhow::bail!("{}", $crate::error::runtime_report($pos, $ctx, $msg))
    };
    ($pos:expr, $ctx:expr, $msg:expr, $( $arg:expr ),+ $(,)?) => {
        anyhow::bail!("{}", $crate::error::runtime_report(
            $pos,
            $ctx,
            format!($msg, $( $arg ),+),
        ))
    };
}

#[macro_export]
macro_rules! semantic_bail {
    ($pos:expr, $ctx:expr, $msg:expr $(,)?) => {
        anyhow::bail!("{}", $crate::error::semantic_report($pos, $ctx, $msg))
    };
    ($pos:expr, $ctx:expr, $msg:expr, $( $arg:expr ),+ $(,)?) => {
        anyhow::bail!("{}", $crate::error::semantic_report(
            $pos,
            $ctx,
            format!($msg, $( $arg ),+),
        ))
    };
}
