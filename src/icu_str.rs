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

pub struct Utf16CharIndices<'a> {
  utf16: &'a [u16],
  n: usize,
}

impl <'a> Iterator for Utf16CharIndices<'a> {
  type Item = (usize, char);

  fn next(&mut self) -> Option<Self::Item> {
    if self.n < self.utf16.len() {
      let v = self.utf16[self.n];
      let c: (usize, char) = (self.n, unsafe { std::char::from_u32_unchecked(v as u32) });
      self.n += 1;
      return Some(c)
    }
    None
  }
}

impl<'a> IcuStrRef<'a> for &'a [u16] {
  type Iter = Utf16CharIndices<'a>;

  fn icu_chars(self, n: usize) -> Self::Iter {
    Utf16CharIndices{ utf16: &self[n..], n: 0 }
  }
}

#[cfg(test)]
mod tests {
  use super::IcuStrRef;

  #[test]
  fn do_stuff() {
    &"abcdef".icu_chars(2).for_each(|(n,c)| println!("{:?}", c));
  }

  #[test]
  fn do_stuff_utf16() {
    let v: Vec<u16> = vec!(65, 66, 67, 68, 69);
    v.as_slice().icu_chars(2).for_each(|(n,c)| println!("{:?}", c));
    assert!(false)
  }
}
