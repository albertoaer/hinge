use crate::{HingeConsumer, Token, Result, HingeOutput, CollectionNode};

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
pub struct HingeBuilder(CollectionNode);

impl HingeBuilder {
  pub fn build(self) -> Hinge {
    self.0.into()
  }
}