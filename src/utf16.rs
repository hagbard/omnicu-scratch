use super::icu_str::IcuStrRef;

// Comparing enum variants directly is harder than expected in Rust, the only really supported way
// to do it is via a pattern match (e.e. "match" or "if let"). We could also just have a helper for
// "is_high_surrogate()" and "is_low_surrogate()" but they would only be used once at the moment.
macro_rules! is_enum {
    ($v:expr, $p:pat) => (
        if let $p = $v { true } else { false }
    );
}

/// Binding of `IcuStrRef` for UTF-16 slices.
impl<'a> IcuStrRef<'a> for &'a [u16] {
  type Iter = Utf16CharIndices<'a>;

  fn icu_chars(self, n: usize) -> Self::Iter {
    // Similarly to `[n..]` on `&str` we want to avoid anyone splitting UTF-16 data inside a
    // surrogate pair. If this were allowed then you could get different results for a string
    // depending on how you iterated it (e.g. as one string or as two sub-strings) because the
    // surrogate pair would appear as .
    //
    // This will also panic "naturally" if the index is too big when the element is read.
    if (n > 0)
        && (self.len() > 1)
        && is_enum!(as_utf16_type(self[n - 1]), Utf16Type::HighSurrogate(_))
        && is_enum!(as_utf16_type(self[n]), Utf16Type::LowSurrogate(_)) {
      panic!("offset {} is not a UTF-16 boundary (it splits a surrogate pair)", n);
    }
    Utf16CharIndices { utf16: &self[n..], n: 0 }
  }
}

/// Iterator for UTF-16 sequences from `IcuStrRef`.
pub struct Utf16CharIndices<'a> {
  utf16: &'a [u16],
  n: usize,
}

impl<'a> Iterator for Utf16CharIndices<'a> {
  type Item = (usize, char);

  fn next(&mut self) -> Option<Self::Item> {
    // This follows the same (strict) rules regarding decoding as Rust strings and always uses the
    // replacement character when it encounters issues. This means there's always a code-point to
    // be decoded unless we've reached the end of the string.
    if self.n < self.utf16.len() {
      Some((self.n, self.decode_next_utf16()))
    } else {
      None
    }
  }
}

impl<'a> Utf16CharIndices<'a> {
  // Private helper to decode next char from the UTF-16 sequence and update internal state.
  // The caller must have already checked that we are not already at the end of the sequence
  // (so this must be a private method since we could not trust arbitrary callers).
  fn decode_next_utf16(&mut self) -> char {
    // DON'T PANIC: n < len from explicit caller check in Utf16CharIndices::next().
    let v = self.utf16[self.n];
    self.n += 1;
    match as_utf16_type(v) {
      // Non-surrogate pair (BMP) code point.
      Utf16Type::Normal(c) => c,
      // Found high surrogate and ensure there's at least one more code value.
      Utf16Type::HighSurrogate(hi) if self.n < self.utf16.len() => {
        // DON'T PANIC: n < len from above match guard.
        match as_utf16_type(self.utf16[self.n]) {
          // High surrogate followed by low surrogate.
          Utf16Type::LowSurrogate(lo) => {
            self.n += 1;
            // DON'T PANIC: This should always produce a valid code point from a surrogate pair.
            // https://en.wikipedia.org/wiki/UTF-16
            std::char::from_u32(((hi - 0xD800) << 10) + (lo - 0xDC00) + 0x10000).unwrap()
          }
          // High surrogate followed by something that's not a low surrogate.
          _ => std::char::REPLACEMENT_CHARACTER,
        }
      }
      // Unpaired low surrogate, or high surrogate at end of range.
      _ => std::char::REPLACEMENT_CHARACTER,
    }
  }
}

// Classification of UTF-16 code values into their surrogate types. Private since there's no need
// to expose how UTF-16 encodes anything.
enum Utf16Type {
  Normal(char),
  HighSurrogate(u32),
  LowSurrogate(u32),
}

// Helper to classify UTF-16 values.
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

#[cfg(test)]
mod tests {
  use super::IcuStrRef;

  #[test]
  fn bmp_only() {
    let v: Vec<u16> = vec!(65, 66, 67, 68, 69);
    let out: Vec<(usize, char)> = v.icu_chars(0).collect();
    assert_eq!(out, vec!((0, 'A'), (1, 'B'), (2, 'C'), (3, 'D'), (4, 'E')));
  }

  #[test]
  fn surrogate_pairs() {
    // UTF-16 values obtained from: https://en.wikipedia.org/wiki/UTF-16#Examples
    let v: Vec<u16> = vec!(0x5B, 0xD801, 0xDC37, 0xD852, 0xDF62, 0x5D);
    let out: Vec<(usize, char)> = v.icu_chars(0).collect();
    assert_eq!(out, vec!((0, '['), (1, '𐐷'), (3, '𤭢'), (5, ']')));
  }

  #[test]
  fn surrogate_pairs_unpaired_high() {
    let v: Vec<u16> = vec!(0x5B, 0xD801, 0x5D);
    let out: Vec<(usize, char)> = v.icu_chars(0).collect();
    assert_eq!(out, vec!((0, '['), (1, '�'), (2, ']')));
  }

  #[test]
  fn surrogate_pairs_trailing_high() {
    let v: Vec<u16> = vec!(0x5B, 0xD801);
    let out: Vec<(usize, char)> = v.icu_chars(0).collect();
    assert_eq!(out, vec!((0, '['), (1, '�')));
  }

  #[test]
  fn surrogate_pairs_unpaired_low() {
    let v: Vec<u16> = vec!(0x5B, 0xDC37, 0x5D);
    let out: Vec<(usize, char)> = v.icu_chars(0).collect();
    assert_eq!(out, vec!((0, '['), (1, '�'), (2, ']')));
  }

  #[test]
  fn surrogate_pairs_trailing_low() {
    let v: Vec<u16> = vec!(0x5B, 0xDC37);
    let out: Vec<(usize, char)> = v.icu_chars(0).collect();
    assert_eq!(out, vec!((0, '['), (1, '�')));
  }

  #[test]
  fn relative_offset() {
    let v: Vec<u16> = vec!(65, 66, 67, 68, 69);
    let out: Vec<(usize, char)> = v.icu_chars(2).collect();
    assert_eq!(out, vec!((0, 'C'), (1, 'D'), (2, 'E')));
  }

  #[test]
  fn empty() {
    let v: Vec<u16> = vec!();
    let out: Vec<(usize, char)> = v.icu_chars(0).collect();
    assert_eq!(out, vec!());
  }

  #[test]
  fn start_at_end() {
    let v: Vec<u16> = vec!(65, 66, 67, 68, 69);
    let out: Vec<(usize, char)> = v.icu_chars(5).collect();
    assert_eq!(out, vec!());
  }

  #[test]
  #[should_panic]
  fn bad_index() {
    let v: Vec<u16> = vec!(0x5B, 0xD801, 0xDC37, 0x5D);
    v.icu_chars(2);
  }

  #[test]
  #[should_panic]
  fn bad_index_oob() {
    let v: Vec<u16> = vec!(65, 66, 67, 68, 69);
    v.icu_chars(6);
  }
}
