use crate::{HingeConsumer, Token, Result, HingeOutput, CollectionNode, NamedNode, AlwaysTrueNode, GreedyNode};

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
pub enum FlagType {
  Bool,
  Item,
  List(usize),
  RemainingItems
}

#[derive(Debug, Clone)]
pub struct HingeBuilder(CollectionNode);

impl HingeBuilder {
  pub fn new() -> Self {
    HingeBuilder(CollectionNode::new())
  }

  pub fn flag(
    mut self,
    name: impl AsRef<str>,
    long: Option<impl AsRef<str>>,
    short: Option<char>,
    flag_config: FlagType
  ) -> Self {
    let name_options = [long.map(|x| format!("--{}", x.as_ref())), short.map(|x| format!("-{}", x))];
    let names = name_options.iter().filter_map(|x| x.as_ref()).collect();
    let node: NamedNode = match flag_config {
      FlagType::Bool => NamedNode::new(names, AlwaysTrueNode),
      FlagType::Item => NamedNode::new(names, GreedyNode::new()),
      FlagType::List(count) => NamedNode::new(names, GreedyNode::new().accept_many(Some(count))),
      FlagType::RemainingItems => NamedNode::new(names, GreedyNode::new().accept_many(None)),
    };
    self.0 = self.0.put(name, node);
    self
  }

  pub fn require(mut self) -> Self {
    self.0 = self.0.require_last();
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