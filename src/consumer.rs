use std::{rc::Rc, fmt::Debug, iter, mem::swap};

use crate::{HingeOutput, Result, HingeCollectionBuilder};

pub type Token = String;

pub trait HingeConsumer: Debug {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput>;
}

#[derive(Debug, Clone)]
pub struct GreedyNode {
  many: bool,
  count: Option<usize>
}

impl GreedyNode {
  pub fn new() -> Self {
    GreedyNode { many: false, count: None }
  }

  pub fn accept_many(mut self, count: Option<usize>) -> Self {
    self.many = true;
    self.count = count;
    self
  }
}

impl HingeConsumer for GreedyNode {
  fn consume(&self, iterator: &mut Box<dyn Iterator<Item = Token>>) -> Result<HingeOutput> {
    if self.many {
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
      return Ok(
        HingeOutput::List(result)
      )
    }
    match iterator.next() {
      Some(val) => Ok(HingeOutput::Value(val)),
      None => Ok(HingeOutput::Empty)
    }
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
}