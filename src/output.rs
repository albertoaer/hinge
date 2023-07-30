use std::collections;

use crate::error::Result;

pub type Atom = String;

const OUTPUT_IS_NOT_A_VALUE: &'static str = "output is not a value";
const INDEX_OUT_OF_BOUNDS: &'static str = "index out of bounds";
const OUTPUT_IS_NOT_A_COLLECTION: &'static str = "output is not a collection error";
const ITEM_DOES_NOT_EXISTS: &'static str = "item does not exists";
const OUTPUT_DOES_NOT_HAVE_INDEXED_ITEMS: &'static str = "output does not have indexed items";
const OUTPUT_DOES_NOT_HAVE_NAMED_ITEMS: &'static str = "output does not have named items";

#[derive(Debug, Clone)]
pub enum HingeOutput {
  Collection {
    named: Option<collections::HashMap<String, HingeOutput>>,
    tail: Option<Vec<HingeOutput>>
  },
  Value(Atom),
  True,
  Empty
}

impl HingeOutput {
  pub fn get_value(self) -> Result<Atom> {
    match self {
      Self::Value(atom) => Ok(atom),
      _ => Err(OUTPUT_IS_NOT_A_VALUE.to_string().into())
    }
  }

  pub fn is_true(&self) -> bool {
    matches!(self, Self::True)
  }

  pub fn is_empty(&self) -> bool {
    matches!(self, Self::Empty)
  }

  pub fn get_tail(&self) -> Result<&[HingeOutput]> {
    match self {
      Self::Collection { named: _, tail } => tail.as_ref().map(|x| x as &[HingeOutput]).ok_or(
        OUTPUT_DOES_NOT_HAVE_INDEXED_ITEMS.to_string().into()
      ),
      _ => Err(OUTPUT_IS_NOT_A_COLLECTION.to_string().into())
    }
  }

  pub fn get_tail_idx(&self, index: usize) -> Result<&HingeOutput> {
    self.get_tail().and_then(|vec| vec.get(index).ok_or(INDEX_OUT_OF_BOUNDS.to_string().into()))
  }

  pub fn get_named_items(&self) -> Result<&collections::HashMap<String, HingeOutput>> {
    match self {
      Self::Collection { named, tail: _ } => named.as_ref().ok_or(
        OUTPUT_DOES_NOT_HAVE_NAMED_ITEMS.to_string().into()
      ),
      _ => Err(OUTPUT_IS_NOT_A_COLLECTION.to_string().into())
    }
  }

  pub fn get_item(&self, name: impl AsRef<str>) -> Result<&HingeOutput> {
    self.get_named_items().and_then(|map| map.get(name.as_ref()).ok_or(ITEM_DOES_NOT_EXISTS.to_string().into()))
  }
}

#[derive(Debug, Clone)]
pub struct OutputCollectionBuilder {
  named: collections::HashMap<String, HingeOutput>,
  tail: Vec<HingeOutput>
}

impl OutputCollectionBuilder {
  pub fn new() -> Self {
    OutputCollectionBuilder { named: collections::HashMap::new(), tail: Vec::new() }
  }

  pub fn add_item(&mut self, name: impl AsRef<str>, value: HingeOutput) {
    self.named.insert(name.as_ref().to_string().clone(), value);
  }

  pub fn has_item(&self, name: impl AsRef<str>) -> bool {
    self.named.contains_key(name.as_ref())
  }

  pub fn add_value(&mut self, value: HingeOutput) {
    self.tail.push(value);
  }

  pub fn collect(self) -> HingeOutput {
    HingeOutput::Collection { named: Some(self.named), tail: Some(self.tail) }
  }
}