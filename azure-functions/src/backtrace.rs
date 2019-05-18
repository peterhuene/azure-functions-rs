use backtrace::BacktraceFrame;
use std::env;
use std::fmt;

pub struct Backtrace {
    inner: backtrace::Backtrace,
}

impl Backtrace {
    pub fn new() -> Backtrace {
        if !Backtrace::is_enabled() {
            return Backtrace {
                inner: Vec::<BacktraceFrame>::new().into(),
            };
        }

        let mut found_start = false;
        let mut found_end = false;

        // This attempts to filter to only the frames relevant to the Azure Function
        let frames: Vec<BacktraceFrame> = backtrace::Backtrace::new()
            .frames()
            .iter()
            .filter_map(|frame| {
                if found_end {
                    return None;
                }

                for symbol in frame.symbols() {
                    if let Some(name) = symbol.name() {
                        let name = format!("{}", name);

                        // Check for the start (i.e. where the panic starts)
                        if !found_start {
                            if name.starts_with("std::panicking::begin_panic::")
                                || name.starts_with("core::panicking::panic::")
                            {
                                found_start = true;
                            }
                            return None;
                        }

                        // Check for the end (the invoker frame)
                        if !found_end && name.contains("::__invoke_") {
                            found_end = true;
                            return None;
                        }
                    }
                }

                Some(frame.clone())
            })
            .collect();

        Backtrace {
            inner: frames.into(),
        }
    }

    pub fn is_enabled() -> bool {
        env::var("RUST_BACKTRACE").unwrap_or_else(|_| "0".to_owned()) == "1"
    }
}

impl fmt::Display for Backtrace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Debug;

        if !Backtrace::is_enabled() {
            return write!(
                f,
                "\nNote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace."
            );
        }

        if self.inner.frames().is_empty() {
            return Ok(());
        }

        writeln!(f)?;
        self.inner.fmt(f)
    }
}
