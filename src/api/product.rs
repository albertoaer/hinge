use crate::{HingeConsumer, Token, HingeOutput, HingeHelp, Result};

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

  pub fn apply_args(&self) -> Result<HingeOutput> {
    self.apply_tokens(std::env::args().skip(1))
  }

  pub fn extract(self) -> Box<dyn HingeConsumer> {
    self.0
  }

  pub fn help(&self) -> String {
    let mut help = HingeHelp::new();
    self.0.apply_help_info(&mut help);
    help.generate()
  }
}

impl<T : HingeConsumer + 'static> From<T> for Hinge {
  fn from(value: T) -> Self {
    Self::new(value)
  }
}