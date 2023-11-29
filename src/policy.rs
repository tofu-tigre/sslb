
pub trait Policy<T> {
  fn select(&mut self, choices: &[T]) -> T;
}

#[derive(Debug)]
pub struct SimpleRoundRobinPolicy {
  curr: usize,
}

impl SimpleRoundRobinPolicy {
  pub fn new() -> Self {
    SimpleRoundRobinPolicy { curr: 0 }
  }
}

impl<T: Clone> Policy<T> for SimpleRoundRobinPolicy {
  fn select(&mut self, choices: &[T]) -> T {
    let choice = choices[self.curr % choices.len()].clone();
    self.curr += 1;
    choice
  }
}