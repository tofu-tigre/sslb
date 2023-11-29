use std::{collections::{HashMap, HashSet}, ops::Index, hash::Hash};
use rand::{thread_rng, Rng};

pub trait Policy {
  fn select(&mut self, client_addr: &str) -> String;
  fn remove(&mut self, el: &str) -> ();
}

struct VectorSet<T> {
  vec_elems: Vec<T>,
  set_elems: HashMap<T, usize>,
}

// Note: Elems must not contain duplicates
impl<T> VectorSet<T>
where T: Clone + PartialEq + Eq + Hash + FromIterator<T> {
  pub fn new(elems: HashSet<T>) -> Self {
    let mut i = 0;
    VectorSet {
      vec_elems: elems.clone().into_iter().collect(),
      set_elems: elems.into_iter().map(|el| {
        let pair = (el, i);
        i += 1;
        return pair
      }).collect()
    }
  }

  pub fn is_empty(&self) -> bool {
    assert_eq!(self.vec_elems.is_empty(), self.set_elems.is_empty());
    return self.vec_elems.is_empty()
  }

  pub fn len(&self) -> usize {
    assert_eq!(self.vec_elems.len(), self.set_elems.len());
    return self.vec_elems.len()
  }

  pub fn remove(&mut self, el: &T) {
    // Get index of element in the vector.
    let index_in_vec = match self.set_elems.remove(el) {
      Some(removed) => removed,
      None => return,
    };
    self.vec_elems.swap_remove(index_in_vec);

    // Edge case: `el` was the last element.
    if self.vec_elems.is_empty() {
      return
    }
    let swapped_el = &self.vec_elems[index_in_vec];
    *self.set_elems.get_mut(swapped_el).unwrap() = index_in_vec;
  }
}

impl<T> Index<usize> for VectorSet<T> {
  type Output = T;
  fn index(&self, index: usize) -> &Self::Output {
    &self.vec_elems[index]
  }
}

pub struct SimpleRoundRobinPolicy {
  curr: usize,
  choices: VectorSet<String>,
}

unsafe impl Send for SimpleRoundRobinPolicy {}

impl SimpleRoundRobinPolicy {
  pub fn new(choices: HashSet<String>) -> Self {
    assert!(choices.len() > 0);
    SimpleRoundRobinPolicy { curr: 0, choices: VectorSet::new(choices) }
  }
}

impl Policy for SimpleRoundRobinPolicy {
  fn select(&mut self, _client_addr: &str) -> String {
    assert!(!self.choices.is_empty());
    let choice = self.choices[self.curr % self.choices.len()].clone();
    self.curr += 1;
    choice
  }

  fn remove(&mut self, el: &str) {
    self.choices.remove(&el.to_owned());
  }
}

pub struct RandomPolicy {
  choices: VectorSet<String>,
}

unsafe impl Send for RandomPolicy {}

impl RandomPolicy {
  pub fn new(choices: HashSet<String>) -> Self {
    assert!(choices.len() > 0);
    RandomPolicy { choices: VectorSet::new(choices) }
  }
}

impl Policy for RandomPolicy {
  fn select(&mut self, _client_addr: &str) -> String {
    assert!(!self.choices.is_empty());
    let i = thread_rng().gen_range(0..self.choices.len());
    self.choices[i].clone()
  }

  fn remove(&mut self, el: &str) {
    self.choices.remove(&el.to_owned());
  }
}