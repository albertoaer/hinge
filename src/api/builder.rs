use std::collections::HashSet;

use crate::{OrNode, ClassificationNode, Hinge, NamedNode, AlwaysTrueNode, OneTokenNode, ListNode, OptionalTokenNode, HingeConsumer, HelpNode, KeyWrapNode, MandatoryItemsNode};

#[derive(Debug, Clone)]
pub enum FlagName {
  Short(char),
  Long(String),
  Both(char, String)
}

impl FlagName {
  fn collect(&self) -> Vec<String> {
    match self {
      FlagName::Short(s) => vec![format!("-{}", s)],
      FlagName::Long(l) => vec![format!("--{}", l)],
      FlagName::Both(s, l) => vec![format!("-{}", s), format!("--{}", l)],
    }
  }
}

impl From<char> for FlagName {
  fn from(value: char) -> Self {
    Self::Short(value)
  }
}

impl From<&str> for FlagName {
  fn from(value: &str) -> Self {
    Self::Long(value.to_string())
  }
}

impl From<String> for FlagName {
  fn from(value: String) -> Self {
    Self::Long(value)
  }
}

impl From<(char, &str)> for FlagName {
  fn from(value: (char, &str)) -> Self {
    Self::Both(value.0, value.1.to_string())
  }
}

impl From<(char, String)> for FlagName {
  fn from(value: (char, String)) -> Self {
    Self::Both(value.0, value.1)
  }
}

pub trait RequirableField {
  fn required_id(&self) -> &str;
}

pub struct RequirableItem(String);

impl RequirableField for RequirableItem {
  fn required_id(&self) -> &str {
    &self.0
  }
}

pub struct Default;

#[derive(Debug, Clone)]
pub struct HingeBuilder<T> {
  subcommands: OrNode,
  node: ClassificationNode,
  mandatory: HashSet<String>,
  state: T
}

impl<T> Into<Hinge> for HingeBuilder<T> {
  fn into(self) -> Hinge {
    self.build()
  }
}

impl HingeBuilder<Default> {
  pub fn new() -> Self {
    HingeBuilder {
      subcommands: OrNode::new(),
      node: ClassificationNode::new(),
      mandatory: HashSet::new(),
      state: Default
    }
  }
}

impl<T> HingeBuilder<T> {
  fn fork<K>(self, state: K) -> HingeBuilder<K> {
    HingeBuilder { subcommands: self.subcommands, node: self.node, mandatory: self.mandatory, state }
  }

  pub fn bool(
    self,
    id: impl AsRef<str>,
    name: impl Into<FlagName>
  ) -> HingeBuilder<Default> {
    let names: FlagName = name.into();
    self.include(&id, NamedNode::new(names.collect(), AlwaysTrueNode), true)
  }
  
  pub fn item(
    self,
    id: impl AsRef<str>,
    name: impl Into<FlagName>
  ) -> HingeBuilder<RequirableItem> {
    let names: FlagName = name.into();
    self.include(&id, NamedNode::new(names.collect(), OneTokenNode), true)
      .fork(RequirableItem(id.as_ref().to_string()))
  }

  pub fn list(
    self,
    id: impl AsRef<str>,
    name: impl Into<FlagName>,
    count: Option<usize>,
  ) -> HingeBuilder<RequirableItem> {
    let names: FlagName = name.into();
    self.include(&id, NamedNode::new(names.collect(), ListNode::new(count)), true)
      .fork(RequirableItem(id.as_ref().to_string()))
  }

  pub fn catch_tail(
    self,
    id: impl AsRef<str>
  ) -> HingeBuilder<Default> {
    self.include(id, NamedNode::new(vec!["--"], ListNode::new(None)), true)
  }

  pub fn arg(
    self,
    id: impl AsRef<str>,
  ) -> HingeBuilder<RequirableItem> {
    self.include(&id, OptionalTokenNode, false)
      .fork(RequirableItem(id.as_ref().to_string()))
  }

  pub fn include(
    mut self,
    id: impl AsRef<str>,
    node: impl HingeConsumer + 'static,
    prioritary: bool
  ) -> HingeBuilder<Default> {
    self.node.put(&id, node, prioritary);
    self.fork(Default)
  }

  pub fn subcommand(
    mut self,
    id: impl AsRef<str>,
    name: impl AsRef<str>,
    hinge: impl Into<Hinge>
  ) -> Self {
    let hinge: Hinge = hinge.into();
    self.subcommands.put(HelpNode::new(
      KeyWrapNode::new(id.as_ref().to_string(), NamedNode::new(vec![name], hinge.extract()))
    ).tabulate());
    self
  }

  pub fn build(
    self
  ) -> Hinge {
    let core = MandatoryItemsNode::new(self.node, self.mandatory.into_iter().collect());
    if self.subcommands.len() > 0 {
      self.subcommands.or(core).into()
    } else {
      core.into()
    }
  }
}

impl<T : RequirableField> HingeBuilder<T> {
  pub fn require(mut self) -> Self {
    self.mandatory.insert(self.state.required_id().to_string());
    self
  }
}