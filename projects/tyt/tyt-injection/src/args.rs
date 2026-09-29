use std::{
    ffi::{OsStr, OsString},
    vec::IntoIter,
};

/// A builder for command arguments, accepting mixed `AsRef<OsStr>` types.
#[derive(Clone, Debug, Default)]
pub struct Args {
    inner: Vec<OsString>,
}

impl Args {
    /// An empty argument list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one argument.
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.inner.push(arg.as_ref().to_os_string());
        self
    }

    /// Appends every argument `args` yields.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.inner
            .extend(args.into_iter().map(|s| s.as_ref().to_os_string()));
        self
    }
}

impl IntoIterator for Args {
    type Item = OsString;

    type IntoIter = IntoIter<OsString>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
