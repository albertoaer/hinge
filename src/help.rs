#[derive(Clone, Debug)]
pub struct HingeHelp {
  names: Vec<String>,
  alt_name: Option<String>,
  description: Option<String>,
  childs: Vec<HingeHelp>
}

impl HingeHelp {
  pub fn new() -> Self {
    HingeHelp { names: Vec::new(), alt_name: None, description: None, childs: Vec::new() }
  }

  pub fn add_name(&mut self, name: impl AsRef<str>) {
    self.names.push(name.as_ref().to_string())
  }

  pub fn set_alternative_name(&mut self, name: impl AsRef<str>) {
    self.alt_name = Some(name.as_ref().into())
  }

  pub fn set_description(&mut self, description: impl AsRef<str>) {
    self.description = Some(description.as_ref().to_string())
  }

  pub fn get_new_child(&mut self) -> &mut HingeHelp {
    self.childs.push(HingeHelp::new());
    self.childs.last_mut().unwrap()
  }

  pub fn generate(&self) -> String {
    let header = match self.names.join(", ") {
        names if names.len() > 0 => names,
        _ => self.alt_name.clone().unwrap_or_default()
      } + self.description.as_ref().unwrap_or(&String::new());
    let body = self.childs.iter()
      .map(|c| c.generate().split("\n").map(|x| String::from("  ") + &x).collect::<Vec<String>>())
      .flatten().fold(String::new(), |a, b| format!("{}\n{}", a, b));
    match (header.len(), body.len()) {
      (0, 0) => String::new(),
      (_, 0) => header,
      (0, _) => body,
      (_, _) => header + &body
    }
  }
}