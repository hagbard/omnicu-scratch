use std::str::CharIndices;

pub trait IcuStr {
  type Iter: Iterator<Item = (usize, char)>;

  fn icu_chars(&self, n: usize) -> Self::Iter;
}

impl<'a> IcuStr for &'a str {
  type Iter = CharIndices<'a>;

  #[inline]
  fn icu_chars(&self, n: usize) -> Self::Iter {
    self[n..].char_indices()
  }
}

#[cfg(test)]
mod tests {
  use super::IcuStr;

  #[test]
  fn do_stuff() {
    "abcdef".icu_chars(2).for_each(|(n,c)| println!("{:?}", c));
  }
}
