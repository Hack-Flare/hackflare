use std::fmt;

#[derive(Debug)]
pub enum ResolveError {
    TooManyConcurrentResolves,
    BindFailed(String),
    ResolutionFailed,
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyConcurrentResolves => {
                write!(f, "too many concurrent resolves")
            }
            Self::BindFailed(msg) => write!(f, "bind failed: {msg}"),
            Self::ResolutionFailed => write!(f, "resolution failed"),
        }
    }
}

impl std::error::Error for ResolveError {}
