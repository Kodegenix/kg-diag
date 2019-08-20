use std::any::TypeId;
use std::fmt::{Debug, Display};
use std::raw::TraitObject;

use super::*;

pub trait Diag: Display + Debug + Send + Sync + 'static {
    fn detail(&self) -> &dyn Detail;

    fn detail_mut(&mut self) -> &mut dyn Detail;

    fn cause(&self) -> Option<&dyn Diag>;

    fn cause_mut(&mut self) -> Option<&mut dyn Diag>;

    fn stacktrace(&self) -> Option<&Stacktrace>;

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl dyn Diag {
    pub fn downcast_ref<T: Diag>(&self) -> Option<&T> {
        if self.type_id() == TypeId::of::<T>() {
            unsafe { Some(&*(self as *const dyn Diag as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: Diag>(&mut self) -> Option<&mut T> {
        if self.type_id() == TypeId::of::<T>() {
            unsafe { Some(&mut *(self as *mut dyn Diag as *mut T)) }
        } else {
            None
        }
    }

    fn display(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let d = self.detail();
        write!(
            f,
            "{} [{}{:04}]: {}\n",
            d.severity(),
            d.severity().code_char(),
            d.code(),
            d
        )?;
        if let Some(parse_diag) = self.downcast_ref::<ParseDiag>() {
            for q in parse_diag.quotes().iter() {
                std::fmt::Display::fmt(q, f)?;
            }
        }
        if let Some(c) = self.cause() {
            write!(f, "caused by: {}", c)?;
        }
        if let Some(s) = self.stacktrace() {
            write!(f, "{}", s)?;
        }
        Ok(())
    }
}

default impl<T: Detail> Diag for T {
    fn detail(&self) -> &dyn Detail {
        self
    }

    fn detail_mut(&mut self) -> &mut dyn Detail {
        self
    }

    fn cause(&self) -> Option<&dyn Diag> {
        None
    }

    fn cause_mut(&mut self) -> Option<&mut dyn Diag> {
        None
    }

    fn stacktrace(&self) -> Option<&Stacktrace> {
        None
    }
}

#[derive(Debug)]
pub struct BasicDiag {
    detail: DetailHolder,
    cause: Option<Box<dyn Diag>>,
    stacktrace: Option<Box<Stacktrace>>,
}

impl BasicDiag {
    pub fn new<T: Detail>(detail: T) -> BasicDiag {
        BasicDiag {
            cause: None,
            stacktrace: None,
            detail: DetailHolder::new(detail),
        }
    }

    pub fn with_cause<T: Detail, E: Diag>(detail: T, cause: E) -> BasicDiag {
        BasicDiag {
            cause: Some(Box::new(cause)),
            stacktrace: None,
            detail: DetailHolder::new(detail),
        }
    }

    pub fn with_stacktrace<T: Detail>(detail: T, stacktrace: Stacktrace) -> BasicDiag {
        BasicDiag {
            cause: None,
            stacktrace: Some(Box::new(stacktrace)),
            detail: DetailHolder::new(detail),
        }
    }

    pub fn with_cause_stacktrace<T: Detail, E: Diag>(
        detail: T,
        cause: E,
        stacktrace: Stacktrace,
    ) -> BasicDiag {
        BasicDiag {
            cause: Some(Box::new(cause)),
            stacktrace: Some(Box::new(stacktrace)),
            detail: DetailHolder::new(detail),
        }
    }
}

impl Diag for BasicDiag {
    fn detail(&self) -> &dyn Detail {
        self.detail.as_ref()
    }

    fn detail_mut(&mut self) -> &mut dyn Detail {
        self.detail.as_mut()
    }

    fn cause(&self) -> Option<&dyn Diag> {
        self.cause.as_ref().map(|d| d.as_ref())
    }

    fn cause_mut(&mut self) -> Option<&mut dyn Diag> {
        self.cause.as_mut().map(|d| d.as_mut())
    }

    fn stacktrace(&self) -> Option<&Stacktrace> {
        self.stacktrace.as_ref().map(|s| s.as_ref())
    }
}

impl<T: Detail> From<T> for BasicDiag {
    #[cfg(debug_assertions)]
    #[inline(always)]
    fn from(detail: T) -> Self {
        BasicDiag::with_stacktrace(detail, Stacktrace::new())
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn from(detail: T) -> Self {
        BasicDiag::new(detail)
    }
}

impl Display for BasicDiag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Diag).display(f)
    }
}

const INPLACE_SIZE: usize = 40;

enum DetailHolder {
    Inplace {
        vtable: *mut (),
        data: [u8; INPLACE_SIZE],
    },
    Ref(Box<dyn Detail>),
}

unsafe impl Send for DetailHolder {}

unsafe impl Sync for DetailHolder {}

impl DetailHolder {
    #[inline(always)]
    fn new<T: Detail>(detail: T) -> DetailHolder {
        if std::mem::size_of::<T>() <= INPLACE_SIZE {
            unsafe {
                let t: TraitObject = std::mem::transmute(&detail as &dyn Detail);
                let mut h = DetailHolder::Inplace {
                    vtable: t.vtable,
                    data: std::mem::zeroed(),
                };
                if let DetailHolder::Inplace { ref mut data, .. } = h {
                    let ptr: *mut T = std::mem::transmute(data);
                    std::ptr::write(ptr, detail);
                } else {
                    unreachable!();
                }
                h
            }
        } else {
            DetailHolder::Ref(Box::new(detail))
        }
    }
}

impl AsRef<dyn Detail> for DetailHolder {
    fn as_ref(&self) -> &dyn Detail {
        match self {
            &DetailHolder::Inplace { vtable, ref data } => unsafe {
                let ptr = TraitObject {
                    data: std::mem::transmute(data),
                    vtable,
                };
                std::mem::transmute(ptr)
            },
            &DetailHolder::Ref(ref detail) => detail.as_ref(),
        }
    }
}

impl AsMut<dyn Detail> for DetailHolder {
    fn as_mut(&mut self) -> &mut dyn Detail {
        match self {
            &mut DetailHolder::Inplace { vtable, ref data } => unsafe {
                let ptr = TraitObject {
                    data: std::mem::transmute(data),
                    vtable,
                };
                std::mem::transmute(ptr)
            },
            &mut DetailHolder::Ref(ref mut detail) => detail.as_mut(),
        }
    }
}

impl Drop for DetailHolder {
    fn drop(&mut self) {
        if let &mut DetailHolder::Inplace { .. } = self {
            let detail = self.as_mut() as *mut dyn Detail;
            unsafe {
                std::ptr::drop_in_place(detail);
            }
        }
    }
}

impl Debug for DetailHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DetailHolder::Inplace { .. } => f
                .debug_tuple("Inplace")
                .field(self.as_ref().as_fmt_debug())
                .finish(),
            DetailHolder::Ref(..) => f
                .debug_tuple("Ref")
                .field(self.as_ref().as_fmt_debug())
                .finish(),
        }
    }
}

#[derive(Debug)]
pub struct SimpleDiag {
    detail: Box<dyn Detail>,
    cause: Option<Box<dyn Diag>>,
    stacktrace: Option<Box<Stacktrace>>,
}

impl SimpleDiag {
    pub fn new<T: Detail>(detail: T) -> SimpleDiag {
        SimpleDiag {
            detail: box detail,
            cause: None,
            stacktrace: None,
        }
    }

    pub fn with_cause<T: Detail, E: Diag>(detail: T, cause: E) -> SimpleDiag {
        SimpleDiag {
            detail: box detail,
            cause: Some(Box::new(cause)),
            stacktrace: None,
        }
    }

    pub fn with_stacktrace<T: Detail>(detail: T, stacktrace: Stacktrace) -> SimpleDiag {
        SimpleDiag {
            detail: box detail,
            cause: None,
            stacktrace: Some(Box::new(stacktrace)),
        }
    }

    pub fn with_cause_stacktrace<T: Detail, E: Diag>(
        detail: T,
        cause: E,
        stacktrace: Stacktrace,
    ) -> SimpleDiag {
        SimpleDiag {
            detail: box detail,
            cause: Some(Box::new(cause)),
            stacktrace: Some(Box::new(stacktrace)),
        }
    }
}

impl Diag for SimpleDiag {
    fn detail(&self) -> &dyn Detail {
        self.detail.as_ref()
    }

    fn detail_mut(&mut self) -> &mut dyn Detail {
        self.detail.as_mut()
    }

    fn cause(&self) -> Option<&dyn Diag> {
        self.cause.as_ref().map(|d| d.as_ref())
    }

    fn cause_mut(&mut self) -> Option<&mut dyn Diag> {
        self.cause.as_mut().map(|d| d.as_mut())
    }

    fn stacktrace(&self) -> Option<&Stacktrace> {
        self.stacktrace.as_ref().map(|s| s.as_ref())
    }
}

impl<T: Detail> From<T> for SimpleDiag {
    #[cfg(debug_assertions)]
    #[inline(always)]
    fn from(detail: T) -> Self {
        SimpleDiag::with_stacktrace(detail, Stacktrace::new_skip(1))
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn from(detail: T) -> Self {
        SimpleDiag::new(detail)
    }
}

impl Display for SimpleDiag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Diag).display(f)
    }
}

#[derive(Debug)]
pub struct ParseDiag {
    detail: Box<dyn Detail>,
    quotes: Vec<Quote>,
    cause: Option<Box<dyn Diag>>,
    stacktrace: Option<Box<Stacktrace>>,
}

impl ParseDiag {
    pub fn new<T: Detail>(detail: T) -> ParseDiag {
        ParseDiag {
            detail: box detail,
            quotes: Vec::new(),
            cause: None,
            stacktrace: None,
        }
    }

    pub fn with_cause<T: Detail, E: Diag>(detail: T, cause: E) -> ParseDiag {
        ParseDiag {
            detail: box detail,
            quotes: Vec::new(),
            cause: Some(Box::new(cause)),
            stacktrace: None,
        }
    }

    pub fn with_stacktrace<T: Detail>(detail: T, stacktrace: Stacktrace) -> ParseDiag {
        ParseDiag {
            detail: box detail,
            quotes: Vec::new(),
            cause: None,
            stacktrace: Some(Box::new(stacktrace)),
        }
    }

    pub fn with_cause_stacktrace<T: Detail, E: Diag>(
        detail: T,
        cause: E,
        stacktrace: Stacktrace,
    ) -> ParseDiag {
        ParseDiag {
            detail: box detail,
            quotes: Vec::new(),
            cause: Some(Box::new(cause)),
            stacktrace: Some(Box::new(stacktrace)),
        }
    }

    pub fn quotes(&self) -> &[Quote] {
        &self.quotes
    }

    pub fn add_quote(&mut self, quote: Quote) {
        self.quotes.push(quote)
    }
}

impl Diag for ParseDiag {
    fn detail(&self) -> &dyn Detail {
        self.detail.as_ref()
    }

    fn detail_mut(&mut self) -> &mut dyn Detail {
        self.detail.as_mut()
    }

    fn cause(&self) -> Option<&dyn Diag> {
        self.cause.as_ref().map(|d| d.as_ref())
    }

    fn cause_mut(&mut self) -> Option<&mut dyn Diag> {
        self.cause.as_mut().map(|d| d.as_mut())
    }

    fn stacktrace(&self) -> Option<&Stacktrace> {
        self.stacktrace.as_ref().map(|s| s.as_ref())
    }
}

impl<T: Detail> From<T> for ParseDiag {
    #[cfg(debug_assertions)]
    #[inline(always)]
    fn from(detail: T) -> Self {
        ParseDiag::with_stacktrace(detail, Stacktrace::new_skip(1))
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn from(detail: T) -> Self {
        ParseDiag::new(detail)
    }
}

impl Display for ParseDiag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Diag).display(f)
    }
}
