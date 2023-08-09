#[derive(Clone, Debug)]
pub struct HingeHelp {
  names: Vec<String>,
  alt_name: Option<String>,
  description: Option<String>,
  childs: Vec<HingeHelp>,
  tabulate_childs: bool
}

const TAB: &'static str = "  ";

impl HingeHelp {
  pub const DEFAULT_TABULATE: bool = false;

  pub fn new() -> Self {
    HingeHelp {
      names: Vec::new(),
      alt_name: None,
      description: None,
      childs: Vec::new(),
      tabulate_childs: HingeHelp::DEFAULT_TABULATE
    }
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

  pub fn get_new_child(&mut self) -> &mut Self {
    self.childs.push(HingeHelp::new());
    self.childs.last_mut().unwrap()
  }

  pub fn set_tabulate_childs(&mut self, tabulate: bool) {
    self.tabulate_childs = tabulate;
  }

  pub fn generate(&self) -> String {
    let header = match self.names.join(", ") {
      names if names.len() > 0 => names,
      _ => self.alt_name.clone().unwrap_or_default()
    } + self.description.as_ref().unwrap_or(&String::new());
    
    let body = self.childs.iter().map(|c| {
      if self.tabulate_childs {
        c.generate().split("\n").map(|x| String::from(TAB) + &x).collect::<Vec<String>>()
      } else {
        c.generate().split("\n").map(|x| x.to_string()).collect::<Vec<String>>()
      }
    }).flatten().reduce(|a, b| format!("{}\n{}", a, b)).unwrap_or_default();

    match (header.len(), body.len()) {
      (0, 0) => String::new(),
      (_, 0) => header,
      (0, _) => body,
      (_, _) => format!("{}\n{}", header, body)
    }
  }
}