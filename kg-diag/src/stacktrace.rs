use backtrace::Backtrace;

use std::sync::Mutex;
use std::path::Path;


struct Inner {
    backtrace: Option<Backtrace>,
    resolved: bool,
    skip: usize,
}

impl Inner {
    fn backtrace(&mut self) -> &Backtrace {
        if !self.resolved {
            let this_file: &Path = Path::new(file!());
            let mut b = self.backtrace.take().unwrap();
            b.resolve();
            let mut frames: Vec<_> = b.into();
            let mut first = None;
            for (i, f) in frames.iter().enumerate() {
                if first.is_none() {
                    if f.symbols().iter().find(|s| s.filename() == Some(this_file)).is_some() {
                        first = Some(i);
                    }
                } else {
                    if f.symbols().iter().find(|s| s.filename() == Some(this_file)).is_some() {
                        first = Some(i);
                    } else {
                        break;
                    }
                }
            }
            if let Some(i) = first {
                frames.drain(0..=(i + self.skip));
            }
            let mut last = None;
            for (mut i, f) in frames.iter().enumerate() {
                if f.symbols().iter().find(|s| if let Some(n) = s.name().map(|n| n.as_str().unwrap_or("")) {
                    if n.starts_with("_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$") {
                        true
                    } else if n.starts_with("_ZN4test8run_test28_$u7b$$u7b$closure$u7d$$u7d$") {
                        i -= 1;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }).is_some() {
                    last = Some(i);
                    break;
                }
            }
            if let Some(i) = last {
                frames.drain(i..);
            }
            assert!(!frames.is_empty());
            self.backtrace = Some(frames.into());
            self.resolved = true;
        }
        self.backtrace.as_ref().unwrap()
    }
}


pub struct Stacktrace(Mutex<Inner>);

impl Stacktrace {
    pub fn new_skip(skip: usize) -> Self {
        Stacktrace(Mutex::new(Inner {
            backtrace: Some(Backtrace::new_unresolved()),
            resolved: false,
            skip,
        }))
    }

    pub fn new() -> Self {
        Self::new_skip(0)
    }
}

impl std::fmt::Display for Stacktrace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut inner = self.0.lock().unwrap();
        write!(f, "{:?}", inner.backtrace())
    }
}

impl std::fmt::Debug for Stacktrace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        struct BacktraceDebug<'a>(&'a Backtrace);

        impl<'a> std::fmt::Debug for BacktraceDebug<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "Backtrace [{} frames...]", self.0.frames().len())
            }
        }

        let inner = self.0.lock().unwrap();
        f.debug_struct("Stacktrace")
            .field("backtrace", &inner.backtrace.as_ref().map(|b| BacktraceDebug(b)))
            .field("resolved", &inner.resolved)
            .field("skip", &inner.skip)
            .finish()
    }
}

