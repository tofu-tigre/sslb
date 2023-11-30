use std::{collections::{HashMap, HashSet}, ops::Index, hash::Hash};
use core::fmt::Debug;
use rand::{thread_rng, Rng};

pub trait Policy: Send {
  fn select(&mut self, client_addr: &str) -> Option<String>;
  fn remove(&mut self, el: &str) -> ();
}

#[derive(Debug)]
struct VectorSet<T> {
  vec_elems: Vec<T>,
  set_elems: HashMap<T, usize>,
}

impl<T> VectorSet<T>
where T: Clone + PartialEq + Eq + Hash + Debug {
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
    self.vec_elems.len()
  }

  pub fn contains(&self, key: &T) -> bool {
    self.set_elems.contains_key(key)
  }

  pub fn remove(&mut self, el: &T) {
    // Get index of element in the vector.
    let index_in_vec = match self.set_elems.remove(el) {
      Some(removed) => removed,
      None => return,
    };
    self.vec_elems.swap_remove(index_in_vec);

    // Edge case: We swapped the last element with itself.
    if index_in_vec == self.vec_elems.len() {
      return
    }

    // Edge case: `el` was the last element or one element left.
    if self.vec_elems.len() <= 0 {
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
  fn select(&mut self, _client_addr: &str) -> Option<String> {
    if self.choices.is_empty() {
      return None
    }
    let choice = self.choices[self.curr % self.choices.len()].clone();
    self.curr += 1;
    Some(choice)
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
  fn select(&mut self, _client_addr: &str) -> Option<String> {
    if self.choices.is_empty() {
      return None
    }
    let i = thread_rng().gen_range(0..self.choices.len());
    Some(self.choices[i].clone())
  }

  fn remove(&mut self, el: &str) {
    self.choices.remove(&el.to_owned());
  }
}

// Tests

#[test]
fn values_persist() {
  let vals = vec![1, 2, 3, 4];
  let vecset = VectorSet::new(vals.into_iter().collect());
  assert!(vecset.contains(&1));
  assert!(vecset.contains(&2));
  assert!(vecset.contains(&3));
  assert!(vecset.contains(&4));
}

#[test]
fn remove_elements() {
  let vals = vec![1, 2, 3, 4];
  let mut vecset = VectorSet::new(vals.into_iter().collect());
  assert!(vecset.contains(&1));
  assert!(vecset.contains(&2));
  assert!(vecset.contains(&3));
  assert!(vecset.contains(&4));

  vecset.remove(&1);
  assert!(!vecset.contains(&1));
  assert!(vecset.contains(&2));
  assert!(vecset.contains(&3));
  assert!(vecset.contains(&4));

  vecset.remove(&2);
  assert!(!vecset.contains(&1));
  assert!(!vecset.contains(&2));
  assert!(vecset.contains(&3));
  assert!(vecset.contains(&4));

  vecset.remove(&3);
  assert!(!vecset.contains(&1));
  assert!(!vecset.contains(&2));
  assert!(!vecset.contains(&3));
  assert!(vecset.contains(&4));

  vecset.remove(&4);
  assert!(!vecset.contains(&1));
  assert!(!vecset.contains(&2));
  assert!(!vecset.contains(&3));
  assert!(!vecset.contains(&4));
}