use std::str::CharIndices;

// Alternate non generic: Box<dyn Iterator<Item=(usize, char)> + 'a>

pub enum Utf16Type {
  Normal(char),
  HighSurrogate(u32),
  LowSurrogate(u32),
}

fn as_utf16_type(v: u16) -> Utf16Type {
  match v {
    // Trap 8-bit values first (which have a safe no-op cast to char).
    0x0000..=0x00FF => Utf16Type::Normal(v as u8 as char),
    // Trap surrogate code values (non code points)
    0xD800..=0xDBFF => Utf16Type::HighSurrogate(v as u32),
    0xDC00..=0xDFFF => Utf16Type::LowSurrogate(v as u32),
    // DON'T PANIC: Safe to "unwrap()" here since no longer a surrogate code value from above.
    _ => Utf16Type::Normal(std::char::from_u32(v as u32).unwrap()),
  }
}

pub trait IcuStrRef<'a> {
  type Iter: Iterator<Item=(usize, char)>;
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

impl<'a> Utf16CharIndices<'a> {
  // Decode the next char from the UTF-16 sequence, updating internal state accordingly.
  // The caller must have already checked that we are not already at the end of the sequence
  // (so this must be a private method).
  fn decode_next_utf16(&mut self) -> char {
    // DON'T PANIC: n < len from explicit caller check in Utf16CharIndices.
    let v = self.utf16[self.n];
    self.n += 1;
    match as_utf16_type(v) {
      Utf16Type::Normal(c) => c,
      Utf16Type::HighSurrogate(hi) if self.n < self.utf16.len() => {
        // DON'T PANIC: n < len from above match guard.
        match as_utf16_type(self.utf16[self.n]) {
          Utf16Type::LowSurrogate(lo) => {
            self.n += 1;
            // DON'T PANIC: This should always produce a valid code point from a surrogate pair.
            // https://en.wikipedia.org/wiki/UTF-16
            std::char::from_u32(((hi - 0xD800) << 10) + (lo - 0xDC00) + 0x10000).unwrap()
          }
          // High surrogate not followed by a low surrogate.
          _ => std::char::REPLACEMENT_CHARACTER,
        }
      }
      // Low surrogate, or high surrogate at end of range.
      _ => std::char::REPLACEMENT_CHARACTER,
    }
  }
}

impl<'a> Iterator for Utf16CharIndices<'a> {
  type Item = (usize, char);

  fn next(&mut self) -> Option<Self::Item> {
    if self.n < self.utf16.len() {
      Some((self.n, self.decode_next_utf16()))
    } else {
      None
    }
  }
}

impl<'a> IcuStrRef<'a> for &'a [u16] {
  type Iter = Utf16CharIndices<'a>;

  fn icu_chars(self, n: usize) -> Self::Iter {
    // !!! MUST NOT ALLOW SPLIT BETWEEN SURROGATE PAIRS !!!
    Utf16CharIndices { utf16: &self[n..], n: 0 }
  }
}

#[cfg(test)]
mod tests {
  use super::IcuStrRef;

  #[test]
  fn do_stuff() {
    &"abcdef".icu_chars(2).for_each(|(n, c)| println!("{:?}", c));
  }

  #[test]
  fn do_stuff_utf16() {
    let v: Vec<u16> = vec!(65, 66, 67, 68, 69, 0xD801, 0xDC37, 0xD852, 0xDF62, 0xDC37, 0xD801);
    v.as_slice().icu_chars(2).for_each(|(n, c)| println!("{:?}", c));
    assert!(false)
  }
}
