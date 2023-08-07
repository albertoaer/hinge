use std::{rc::Rc, fmt::Debug, iter, mem::swap, collections};

use crate::{HingeOutput, Result, HingeCollectionBuilder, HingeHelp};

pub type Token = String;

pub trait HingeConsumer: Debug {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput>;

  fn apply_help_info(&self, _: &mut HingeHelp) { }
}

impl<T : HingeConsumer + ?Sized> HingeConsumer for Box<T> {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput> {
    (**self).consume(iterator)
  }

  fn apply_help_info(&self, help: &mut HingeHelp) {
    (**self).apply_help_info(help)
  }
}

#[derive(Debug, Clone)]
pub struct AlwaysTrueNode;

impl HingeConsumer for AlwaysTrueNode {
  fn consume(&self, _: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput> {
    Ok(HingeOutput::True)
  }
}

#[derive(Debug, Clone)]
pub struct OneTokenNode;

impl HingeConsumer for OneTokenNode {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput> {
    match iterator.next() {
      Some(val) => Ok(HingeOutput::Value(val)),
      None => Err("expecting a value".to_string().into())
    }
  }
}

#[derive(Debug, Clone)]
pub struct OptionalTokenNode;

impl HingeConsumer for OptionalTokenNode {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput> {
    Ok(iterator.next().map(|val| HingeOutput::Value(val)).unwrap_or(HingeOutput::Empty))
  }
}

#[derive(Debug, Clone)]
pub struct ListNode {
  count: Option<usize>
}

impl ListNode {
  pub fn new(count: Option<usize>) -> Self {
    ListNode { count }
  }
}

impl HingeConsumer for ListNode {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput> {
    let result: Vec<_> = {
      if let Some(count) = self.count {
        let collected: Vec<_> = iterator.take(count).map(|x| HingeOutput::Value(x)).collect();
        if collected.len() < count {
          return Err("not enough elements".to_string().into());
        }
        collected
      } else {
        iterator.map(|x| HingeOutput::Value(x)).collect()
      }
    };
    return Ok(HingeOutput::List(result))
  }
}

#[derive(Debug, Clone)]
pub struct NamedNode {
  names: Vec<String>,
  wrapped: Rc<Box<dyn HingeConsumer>>
}

impl NamedNode {
  pub fn new(names: Vec<impl AsRef<str>>, child_consumer: impl HingeConsumer + 'static) -> Self {
    NamedNode {
      names: names.into_iter().map(|x| x.as_ref().to_string()).collect(),
      wrapped: Rc::new(Box::new(child_consumer))
    }
  }
}

impl HingeConsumer for NamedNode {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput> {
    let first_token = match iterator.next() {
      Some(n) => n,
      None => return Ok(HingeOutput::Empty),
    };
    if !self.names.iter().any(|x| *x == first_token) {
      let mut tmp: Box<dyn Iterator<Item = Token>> = Box::new(iter::empty());
      swap(&mut tmp, iterator);
      *iterator = Box::new(iter::once(first_token).chain(tmp));
      return Ok(HingeOutput::Empty)
    }
    return self.wrapped.consume(iterator)
  }

  fn apply_help_info(&self, help: &mut HingeHelp) {
    for name in self.names.iter() {
      help.add_name(name);
    }
    self.wrapped.apply_help_info(help);
  }
}

#[derive(Debug, Clone)]
pub struct CollectionNodeEntry {
  name: Option<String>,
  consumer: Rc<Box<dyn HingeConsumer>>,
  require: bool
}

impl CollectionNodeEntry {
  pub fn new(consumer: impl HingeConsumer + 'static) -> Self {
    CollectionNodeEntry { name: None, consumer: Rc::new(Box::new(consumer)), require: false }
  }

  pub fn new_named(name: String, consumer: impl HingeConsumer + 'static) -> Self {
    CollectionNodeEntry { name: Some(name), consumer: Rc::new(Box::new(consumer)), require: false }
  }

  pub fn require(&mut self, require: bool) {
    self.require = require
  }

  pub fn rename(&mut self, name: impl AsRef<str>) {
    self.name = Some(name.as_ref().to_string())
  }

  pub fn remove_name(&mut self) {
    self.name = None
  }
}

#[derive(Debug, Clone)]
pub struct CollectionNode {
  items: Vec<CollectionNodeEntry>
}

impl CollectionNode {
  pub fn new() -> Self {
    CollectionNode { items: Vec::new() }
  }

  pub fn add(mut self, value: impl HingeConsumer + 'static) -> Self {
    self.items.push(CollectionNodeEntry::new(value));
    self
  }

  pub fn put(mut self, id: impl AsRef<str>, value: impl HingeConsumer + 'static) -> Self {
    self.items.push(CollectionNodeEntry::new_named(id.as_ref().to_string(), value));
    self
  }

  pub fn update_last(mut self, operation: impl FnOnce(&mut CollectionNodeEntry)) -> Self {
    if let Some(n) = self.items.last_mut() {
      operation(n);
    }
    self
  }

  pub fn require_last(self) -> Self {
    self.update_last(|x| x.require(true))
  }
}

impl HingeConsumer for CollectionNode {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput> {
    let mut builder = HingeCollectionBuilder::new();
    loop {
      let results: Result<Vec<_>> = self.items.iter().map(|item| item.consumer.consume(iterator).map(|x| (item, x))).collect();
      let non_empty: Vec<_> = results?.into_iter().filter(|(_, x)| !x.is_empty()).collect();
      if non_empty.is_empty() {
        break;
      }
      for (item, result) in non_empty {
        match &item.name {
          Some(name) => builder.add_item(name, result),
          None => builder.add_value(result)
        }
      }
    }
    for item in self.items.iter().filter(|x| x.name.is_some()) {
      let name = item.name.as_ref().unwrap();
      match (builder.has_item(name), item.require) {
        (false, true) => return Err(format!("Expecting item with name: {}", name).into()),
        (false, false) => builder.add_item(name, HingeOutput::Empty),
        _ => (),
      }
    }
    Ok(builder.collect())
  }

  fn apply_help_info(&self, help: &mut HingeHelp) {
    for item in self.items.iter() {
      item.consumer.apply_help_info(help.get_new_child())
    }
  }
}

#[derive(Debug, Clone)]
pub struct ClassificationNode {
  entries: (
    Vec<(String, Rc<Box<dyn HingeConsumer>>)>,
    Vec<(String, Rc<Box<dyn HingeConsumer>>)>
  )
}

impl ClassificationNode {
  pub fn new() -> Self {
    ClassificationNode { entries: (Vec::new(), Vec::new()) }
  }

  pub fn put(&mut self, id: impl AsRef<str>, value: impl HingeConsumer + 'static, prioritary: bool) {
    if prioritary {
      &mut self.entries.0
    } else {
      &mut self.entries.1
    }.push((id.as_ref().to_string(), Rc::new(Box::new(value))));
  }

  pub fn all_entries(&self) -> impl Iterator<Item = &(String, Rc<Box<dyn HingeConsumer>>)> {
    self.entries.0.iter().chain(self.entries.1.iter())
  }
}

impl HingeConsumer for ClassificationNode {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput> {
    let mut builder = HingeCollectionBuilder::new();
    loop {
      let results: Result<Vec<_>> = self.all_entries()
        .filter(|item| !builder.has_item(&item.0))
        .map(|item| item.1.consume(iterator).map(|x| (item, x)))
        .collect();
      let non_empty: Vec<_> = results?.into_iter().filter(|(_, x)| !x.is_empty()).collect();
      if non_empty.is_empty() {
        break;
      }
      for (item, result) in non_empty {
        builder.add_item(&item.0, result);
      }
    }
    for item in self.all_entries() {
      if !builder.has_item(&item.0) {
        builder.add_item(&item.0, HingeOutput::Empty);
      }
    }
    Ok(builder.collect())
  }

  fn apply_help_info(&self, help: &mut HingeHelp) {
    for item in self.all_entries() {
      let child = help.get_new_child();
      child.set_alternative_name(format!("<{}>", item.0));
      item.1.apply_help_info(child);
    }
  }
}

#[derive(Debug, Clone)]
pub struct MandatoryItemsNode {
  child: Rc<Box<dyn HingeConsumer>>,
  names: Vec<String>
}

impl MandatoryItemsNode {
  pub fn new(child: impl HingeConsumer + 'static, names: Vec<String>) -> Self {
    MandatoryItemsNode { child: Rc::new(Box::new(child)), names }
  }
}

impl HingeConsumer for MandatoryItemsNode {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput> {
    let builder: HingeCollectionBuilder = self.child.consume(iterator)?.try_into()?;
    for name in &self.names {
      let map: &collections::HashMap<_, _> = builder.as_ref();
      if !map.contains_key(name) || map.get(name).unwrap().is_empty() {
        return Err(format!("Expecting item with name: {}", name).into());
      }
    }
    Ok(builder.collect())
  }

  fn apply_help_info(&self, help: &mut HingeHelp) {
    self.child.apply_help_info(help);
  }
}

#[derive(Debug, Clone)]
pub struct KeyWrapNode {
  key: String,
  wrapped: Rc<Box<dyn HingeConsumer>>
}

impl KeyWrapNode {
  pub fn new(key: impl AsRef<str>, wrapped: impl HingeConsumer + 'static) -> Self {
    KeyWrapNode { key: key.as_ref().to_string(), wrapped: Rc::new(Box::new(wrapped)) }
  }
}

impl HingeConsumer for KeyWrapNode {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput> {
    match self.wrapped.consume(iterator) {
      res @ Ok(HingeOutput::Empty) | res @ Err(_) => res,
      Ok(value) => Ok(HingeOutput::Map(
        collections::HashMap::from_iter(iter::once((self.key.clone(), value)))
      ))
    }
  }

  fn apply_help_info(&self, help: &mut HingeHelp) {
    self.wrapped.apply_help_info(help);
  }
}

#[derive(Debug, Clone)]
pub struct OrNode(Vec<Rc<Box<dyn HingeConsumer>>>);

impl OrNode {
  pub fn new() -> Self {
    OrNode(Vec::new())
  }

  pub fn put(&mut self, value: impl HingeConsumer + 'static) {
    self.0.push(Rc::new(Box::new(value)));
  }
  
  pub fn or(mut self, value: impl HingeConsumer + 'static) -> Self {
    self.put(value);
    OrNode(self.0)
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }
}

impl HingeConsumer for OrNode {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput> {
    let mut options = self.0.iter();
    while let Some(output) = options.next().map(|x| x.consume(iterator)) {
      if !matches!(output, Ok(HingeOutput::Empty)) {
        return output;
      }
    }
    Ok(HingeOutput::Empty)
  }

  fn apply_help_info(&self, help: &mut HingeHelp) {
    for item in self.0.iter() {
      item.apply_help_info(help.get_new_child())
    }
  }
}