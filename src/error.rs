use core::fmt;
use std::{result, error::Error, rc::Rc};

#[derive(Debug, Clone)]
pub enum HingeError {
  Wrapper(Rc<&'static dyn Error>),
  String(String)
}

impl fmt::Display for HingeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Wrapper(wrapped) => wrapped.fmt(f),
      Self::String(string) => string.fmt(f)
    }
  }
}

impl Error for HingeError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::Wrapper(wrapped) => Some(wrapped.as_ref()),
      _ => None
    }
  }
}

impl Into<HingeError> for String {
  fn into(self) -> HingeError {
    return HingeError::String(self)
  }
}

pub type Result<T> = result::Result<T, HingeError>;