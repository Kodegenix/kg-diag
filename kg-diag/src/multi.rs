use super::*;

#[derive(Debug)]
pub struct Diags {
    diags: Vec<Box<dyn Diag>>,
    max_severity: Severity,
}

impl Diags {
    pub fn new() -> Diags {
        Diags {
            diags: Vec::new(),
            max_severity: Severity::Info,
        }
    }

    pub fn add_diag<D: Diag>(&mut self, diag: D) -> Result<(), Errors> {
        self.max_severity = std::cmp::max(self.max_severity, diag.detail().severity());
        let recover = diag.detail().severity().is_recoverable();
        self.diags.push(Box::new(diag));
        if recover {
            Ok(())
        } else {
            Err(Errors::new(self.max_severity))
        }
    }

    pub fn result<T>(&self, res: T) -> Result<T, Errors> {
        if self.max_severity.is_error() {
            Err(Errors::new(self.max_severity))
        } else {
            Ok(res)
        }
    }
}

pub trait ResultExt<T, E: Diag> {
    fn add_err(self, diags: &mut Diags) -> Result<T, Errors>;
}

impl<T, E: Diag> ResultExt<T, E> for Result<T, E> {
    fn add_err(self, diags: &mut Diags) -> Result<T, Errors> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(diags.add_diag(err).unwrap_err()),
        }
    }
}

#[derive(Debug)]
pub struct Errors {
    severity: Severity,
    stacktrace: Option<Box<Stacktrace>>,
}

impl Errors {
    pub fn new(severity: Severity) -> Errors {
        Errors {
            severity,
            stacktrace: None,
        }
    }

    pub fn with_stacktrace(severity: Severity, stacktrace: Stacktrace) -> Errors {
        Errors {
            severity,
            stacktrace: Some(box stacktrace),
        }
    }
}

impl Detail for Errors {
    fn severity(&self) -> Severity {
        self.severity
    }
}

impl Diag for Errors {
    fn stacktrace(&self) -> Option<&Stacktrace> {
        self.stacktrace.as_ref().map(|s| s.as_ref())
    }
}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "multiple errors\n")?;
        if let Some(ref s) = self.stacktrace {
            write!(f, "{:?}", s)?;
        }
        Ok(())
    }
}

/*


#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Operation {
    #[display("create", "creating")]
    Create,
    #[display("read", "reading")]
    Read,
    #[display("write", "writing")]
    Write,
    #[display("remove", "removing")]
    Remove,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum FileType {
    #[display("file")]
    File,
    #[display("dir")]
    Dir,
    #[display("link")]
    Link,
    #[display("device")]
    Device,
}


#[derive(Debug, Clone)]
struct IoErr {
    kind: std::io::ErrorKind,
    operation: Operation,
    file_type: FileType,
    path: Option<PathBuf>,
}

impl Detail for IoErr {
    fn code(&self) -> u32 {
        1 + self.kind as u32
    }
}

impl Display for IoErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::error::Error;
        let e: std::io::Error = self.kind.into();
        (self as &Detail).write_prefix(f)?;
        if let Some(ref p) = self.path {
            write!(f, "{}, while {:#} {} '{}'", e.description(), self.operation, self.file_type, p.display())
        } else {
            write!(f, "{}", e.description())
        }
    }
}






#[cfg(test)]
mod tests {
    use super::*;
    use test::*;

    #[test]
    fn test_1() {
        let e: BasicDiag = IoErr {
            path: Some("/home/dir/file.txt".into()),
            operation: Operation::Create,
            file_type: FileType::File,
            kind: std::io::ErrorKind::NotFound,
        }.into();

        if let Some(d) = e.detail().downcast_ref::<IoErr>() {
            println!("{}", d);
        }

    }


}
*/
