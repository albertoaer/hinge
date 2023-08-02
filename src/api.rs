use crate::{HingeConsumer, Token, Result, HingeOutput, CollectionNode, NamedNode, GreedyNode, AlwaysTrueNode};

#[derive(Debug)]
pub struct Hinge(Box<dyn HingeConsumer>);

impl Hinge {
  pub fn new(consumer: impl HingeConsumer + 'static) -> Hinge {
    Hinge(Box::new(consumer))
  }

  pub fn apply_tokens(&self, tokens: impl Iterator<Item = Token> + 'static) -> Result<HingeOutput> {
    let mut tokens: Box<dyn Iterator<Item = Token>> = Box::new(tokens);
    let result = self.0.consume(&mut tokens)?;
    if result.is_empty() {
      return Err("expecting consumer to consume the tokens".to_string().into())
    }
    match tokens.next() {
      Some(token) => Err(format!("not every token could be processed, next is: {}", token).into()),
      None => Ok(result),
    }
  }
}

impl<T : HingeConsumer + 'static> From<T> for Hinge {
  fn from(value: T) -> Self {
    Self::new(value)
  }
}

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

#[derive(Debug, Clone)]
pub struct HingeBuilder(CollectionNode);

impl HingeBuilder {
  pub fn new() -> Self {
    HingeBuilder(CollectionNode::new())
  }

  pub fn bool_flag(
    mut self,
    id: impl AsRef<str>,
    name: impl Into<FlagName>
  ) -> Self {
    let names: FlagName = name.into();
    self.0 = self.0.put(id, NamedNode::new(names.collect(), AlwaysTrueNode));
    self
  }
  
  pub fn item_flag(
    mut self,
    id: impl AsRef<str>,
    name: impl Into<FlagName>,
    required: bool
  ) -> Self {
    let names: FlagName = name.into();
    self.0 = self.0.put(id, NamedNode::new(names.collect(), GreedyNode::new()));
    if required {
      self.0 = self.0.require_last()
    }
    self
  }

  pub fn list_flag(
    mut self,
    id: impl AsRef<str>,
    name: impl Into<FlagName>,
    count: Option<usize>,
    required: bool
  ) -> Self {
    let names: FlagName = name.into();
    let node: NamedNode = NamedNode::new(names.collect(), match count {
      Some(count) => GreedyNode::new().accept_many(Some(count)),
      None => GreedyNode::new().accept_many(None),
    });
    self.0 = self.0.put(id, node);
    if required {
      self.0 = self.0.require_last();
    }
    self
  }

  pub fn catch_tail(mut self, name: impl AsRef<str>) -> Self {
    self.0 = self.0.put(name, NamedNode::new(vec!["--"], GreedyNode::new().accept_many(None)));
    self
  }

  pub fn build(self) -> Hinge {
    self.0.into()
  }
}