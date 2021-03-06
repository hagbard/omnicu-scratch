#![no_std]

use super::icu_str::IcuStrRef;

pub enum Type {
  NFC, NFKC, NFD, NFKD
}

pub trait Normalizer {
  fn is_normalized(&self, t: Type) -> bool;
}

// Bind for 'str' (which is most expected usage).
impl Normalizer for str {
  fn is_normalized(&self, t: Type) -> bool {
    t.is_normalized(self)
  }
}

// Bind for anything which can represent itself as a reference to a "str".
//
// Note that by doing _this_ we prohibit ourselves from _also_ binding to 'Chars', since it would
// create ambiguity for the compiler. It seems more useful to do this though, since the Chars
// struct is typically transient and not passed around or stored much (so any caller is expected
// to naturally have the 'str' available anyway). In the worst case, a caller with only a 'Chars'
// instance can just use 'as_str()' anyway:
impl<'a, T: AsRef<str>> Normalizer for T {
  fn is_normalized(&self, t: Type) -> bool {
    t.is_normalized_icu(&self)
  }
}

impl Type {
  pub fn is_normalized(&self, s: &str) -> bool {
    self.is_normalized_icu(&s)
  }

  pub fn is_normalized_icu<'a>(&self, s: impl IcuStrRef<'a>) -> bool {
    match self {
      Type::NFC => is_nfc_normalized(s),
      Type::NFKC => false,
      Type::NFD => false,
      Type::NFKD => false,
    }
  }
}

fn is_nfc_normalized<'a>(s: impl IcuStrRef<'a>) -> bool {
  s.icu_chars(0).for_each(|(i,c)| eprintln!("{:?}", c));
  true
}

#[cfg(test)]
mod tests {
  use super::{Normalizer, Type};

  #[test]
  fn is_nfc_normalized() {
    // Not opting into the bound trait; only use "normalizer::Type".
    assert_eq!(Type::NFC.is_normalized(&"Hello World"), true);

    // Using bound trait via "normalizer::Normalizer".
    assert_eq!("Hello World".is_normalized(Type::NFC), true);
    assert_eq!("Hello World".chars().as_str().is_normalized(Type::NFC), true);
  }
}
