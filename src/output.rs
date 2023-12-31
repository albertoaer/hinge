use std::collections;

use crate::{error::Result, HingeError};

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

  pub fn is_value(&self, concrete: Option<impl AsRef<str>>) -> bool {
    match (self, concrete) {
      (Self::Value(val), Some(concrete)) => *val == concrete.as_ref().to_string(),
      (Self::Value(_), None) => true,
      _ => false
    }
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

impl TryInto<Vec<HingeOutput>> for HingeOutput {
  type Error = HingeError;

  fn try_into(self) -> std::result::Result<Vec<HingeOutput>, Self::Error> {
    match self {
      Self::List(list) | Self::MapList(_, list) => Ok(list),
      _ => Err(OUTPUT_IS_NOT_A_MAP.to_string().into())
    }
  }
}

impl TryInto<collections::HashMap<String, HingeOutput>> for HingeOutput {
  type Error = HingeError;

  fn try_into(self) -> std::result::Result<collections::HashMap<String, HingeOutput>, Self::Error> {
    match self {
      Self::Map(map) | Self::MapList(map, _) => Ok(map),
      _ => Err(OUTPUT_IS_NOT_A_MAP.to_string().into())
    }
  }
}

impl TryInto<String> for HingeOutput {
  type Error = HingeError;

  fn try_into(self) -> std::result::Result<String, Self::Error> {
    match self {
      Self::Value(atom) => Ok(atom),
      _ => Err(OUTPUT_IS_NOT_A_VALUE.to_string().into())
    }
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

  pub fn new_from(list: Vec<HingeOutput>, map: collections::HashMap<String, HingeOutput>) -> Self {
    HingeCollectionBuilder { list, map }
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

impl AsRef<Vec<HingeOutput>> for HingeCollectionBuilder {
  fn as_ref(&self) -> &Vec<HingeOutput> {
    &self.list
  }
}

impl AsRef<collections::HashMap<String, HingeOutput>> for HingeCollectionBuilder {
  fn as_ref(&self) -> &collections::HashMap<String, HingeOutput> {
    &self.map
  }
}

impl TryInto<HingeCollectionBuilder> for HingeOutput {
  type Error = HingeError;

  fn try_into(self) -> std::result::Result<HingeCollectionBuilder, Self::Error> {
    Ok(match self {
      HingeOutput::Map(map) => HingeCollectionBuilder::new_from(Vec::new(), map),
      HingeOutput::List(list) => HingeCollectionBuilder::new_from(list, collections::HashMap::new()),
      HingeOutput::MapList(map, list) => HingeCollectionBuilder::new_from(list, map),
      _ => return Err("result is not a collection".to_string().into())
    })
  }
}