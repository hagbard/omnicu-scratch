#![no_std]

use core::str::Chars;

pub trait Normalizer {
  fn is_normalized(&self, t: Type) -> bool;
}

// Bind for "str" (which is most expected usage).
impl Normalizer for str {
  fn is_normalized(&self, t: Type) -> bool {
    t.is_normalized(self)
  }
}

// Bind for anything which can represent itself as a reference to a "str"
impl<'a, T> Normalizer for T where T: AsRef<&'a str> {
  fn is_normalized(&self, t: Type) -> bool {
    t.is_normalized(self.as_ref())
  }
}

pub enum Type {
  NFC, NFKC, NFD, NFKD
}

impl Type {
  pub fn is_normalized(&self, s: &str) -> bool {
    self.is_normalized_iter(&s.chars())
  }

  pub fn is_normalized_chars(&self, s: &Chars) -> bool {
    self.is_normalized_iter(s)
  }

  pub fn is_normalized_iter(&self, s: &dyn Iterator<Item=char>) -> bool {
    match self {
      Type::NFC => is_nfc_normalized(s),
      Type::NFKC => false,
      Type::NFD => false,
      Type::NFKD => false,
    }
  }
}

fn is_nfc_normalized(s: &dyn Iterator<Item=char>) -> bool {
  true
}

#[cfg(test)]
mod tests {
  use super::Type;
  use super::Normalizer;

  #[test]
  fn is_nfc_normalized() {
    assert_eq!(Type::NFC.is_normalized(&"Hello World"), true);
    assert_eq!("Hello World".is_normalized(Type::NFC), true);
  }
}
