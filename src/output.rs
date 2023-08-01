use std::collections;

use crate::error::Result;

pub type Atom = String;

const OUTPUT_IS_NOT_A_VALUE: &'static str = "output is not a value";
const INDEX_OUT_OF_BOUNDS: &'static str = "index out of bounds";
const ITEM_DOES_NOT_EXISTS: &'static str = "item does not exists";
const OUTPUT_IS_NOT_A_LIST: &'static str = "output is not a list";
const OUTPUT_IS_NOT_A_MAP: &'static str = "output is not a map";

#[derive(Debug, Clone)]
pub enum HingeOutput {
  Map(collections::HashMap<String, HingeOutput>),
  List(Vec<HingeOutput>),
  MapList(collections::HashMap<String, HingeOutput>, Vec<HingeOutput>),
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

  pub fn get_list(&self) -> Result<&[HingeOutput]> {
    match self {
      Self::List(list) | Self::MapList(_, list) => Ok(list.as_slice()),
      _ => Err(OUTPUT_IS_NOT_A_LIST.to_string().into())
    }
  }

  pub fn get_list_idx(&self, index: usize) -> Result<&HingeOutput> {
    self.get_list().and_then(|vec| vec.get(index).ok_or(INDEX_OUT_OF_BOUNDS.to_string().into()))
  }

  pub fn get_map(&self) -> Result<&collections::HashMap<String, HingeOutput>> {
    match self {
      Self::Map(map) | Self::MapList(map, _) => Ok(map),
      _ => Err(OUTPUT_IS_NOT_A_MAP.to_string().into())
    }
  }

  pub fn get_item(&self, name: impl AsRef<str>) -> Result<&HingeOutput> {
    self.get_map().and_then(|map| map.get(name.as_ref()).ok_or(ITEM_DOES_NOT_EXISTS.to_string().into()))
  }
}

#[derive(Debug, Clone)]
pub struct HingeCollectionBuilder {
  list: Vec<HingeOutput>,
  map: collections::HashMap<String, HingeOutput>
}

impl HingeCollectionBuilder {
  pub fn new() -> Self {
    HingeCollectionBuilder { list: Vec::new(), map: collections::HashMap::new() }
  }

  pub fn add_value(&mut self, value: HingeOutput) {
    self.list.push(value);
  }

  pub fn add_item(&mut self, name: impl AsRef<str>, value: HingeOutput) {
    self.map.insert(name.as_ref().to_string().clone(), value);
  }

  pub fn has_item(&self, name: impl AsRef<str>) -> bool {
    self.map.contains_key(name.as_ref())
  }

  pub fn collect(self) -> HingeOutput {
    if self.list.is_empty() {
      HingeOutput::Map(self.map)
    } else if self.map.is_empty() {
      HingeOutput::List(self.list)
    } else {
      HingeOutput::MapList(self.map, self.list)
    }
  }
}