use std::str::CharIndices;

// Alternate non generic: Box<dyn Iterator<Item=(usize, char)> + 'a>

pub trait IcuStrRef<'a> {
  type Iter: Iterator<Item = (usize, char)>;
  fn icu_chars(self, n: usize) -> Self::Iter;
}

impl<'a, T: AsRef<str>> IcuStrRef<'a> for &'a T {
  type Iter = std::str::CharIndices<'a>;

  fn icu_chars(self, n: usize) -> Self::Iter {
    self.as_ref()[n..].char_indices()
  }
}

//pub trait IcuStr {
//  type Iter: Iterator<Item = (usize, char)>;
//
//  fn icu_chars(&self, n: usize) -> Self::Iter;
//}
//
//impl<'a> IcuStr for &'a str {
//  type Iter = CharIndices<'a>;
//
//  #[inline]
//  fn icu_chars(&self, n: usize) -> Self::Iter {
//    self[n..].char_indices()
//  }
//}

#[cfg(test)]
mod tests {
  use super::IcuStrRef;

  #[test]
  fn do_stuff() {
    &"abcdef".icu_chars(2).for_each(|(n,c)| println!("{:?}", c));
  }
}
