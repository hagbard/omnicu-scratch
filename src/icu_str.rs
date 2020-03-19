use std::str::CharIndices;

// Alternate non generic: Box<dyn Iterator<Item=(usize, char)> + 'a>

/// Represents a `char` sequence without reference to an underlying encoding, intended for use by
/// ICU functions.
///
/// ```
/// let v: Vec<(usize, char)> = "I💖🇷🇺🇸🇹".icu_chars(0).collect();
/// assert_eq!(v, vec!((0, 'I'), (1, '💖'), (5, '🇷'), (9, '🇺'), (13, '🇸'), (17, '🇹')))
/// ```
///
/// The iterated offset `n` represents the offset in the underlying encoding of the start of the
/// associated `char`. This value must only be used to "rewind" iteration by using it as input to
/// `icu_chars()` function (for the same `IcuStrRef`). The rules for handling offsets are:
///
/// * The only constant offset which can be used safely to start an iteration is `0`
/// * Offsets obtained during iteration are always relative to the start of that iteration (i.e.
///   the first iterated code point always has an offset of `0`).
/// * Offsets are unique in an iteration and always increase in value.
/// * An offset within an iteration can be added to the starting offset of that iteration to obtain
///   a new starting offset from that point (this permits calling code to store an offset in order
///   to "rewind" iteration back to a known position later).
///
/// # Implementation Notes
///
/// This type exists in order to present a minimal API for accessing string data which is not tied
/// to any specific underlying encoding and does not require data to be re-encoded or copied. It is
/// implemented by a lightweight wrapper for `&str` and also for `UTF-16` encoded values in a
/// `[u16]` (via the `utf16` module) and can easily be extended to support non-buffered data.
///
/// As such, this API does not support general purpose features for string handling and focusses
/// only on providing the APIs necessary to implement ICU functionality (e.g. string normalization
/// or comparison) in an efficient manner.
///
/// Public ICU methods designed for normal Rust usage should always accept `&str` or `T: AsRef<str>`
/// rather than forcing callers to instantiate one of these (while this type is public, it is not
/// intended as a way to manage string data in general Rust code).
pub trait IcuStrRef<'a> {
  type Iter: Iterator<Item=(usize, char)>;

  /// Iterates over a sequence of `(offset, code point)` pairs starting from the specified `start`
  /// offset (which must either be `0` or an offset derived from a previous iteration).
  ///
  /// This function will `Panic` if the given starting offset is invalid.
  // TODO: Consider making this return an Option or Result ??
  fn icu_chars(self, start: usize) -> Self::Iter;
}

/// Binding of `IcuStrRef` for string-like Rust types.
impl<'a, T: AsRef<str>> IcuStrRef<'a> for &'a T {
  type Iter = std::str::CharIndices<'a>;

  fn icu_chars(self, start: usize) -> Self::Iter {
    self.as_ref()[start..].char_indices()
  }
}

#[cfg(test)]
mod tests {
  use super::IcuStrRef;

  #[test]
  fn ascii_only() {
    let out: Vec<(usize, char)> = "ABCDE".icu_chars(0).collect();
    assert_eq!(out, vec!((0, 'A'), (1, 'B'), (2, 'C'), (3, 'D'), (4, 'E')));
  }

  #[test]
  fn multi_byte() {
    // UTF-8 chars obtained from: https://en.wikipedia.org/wiki/UTF-8#Examples (1,2,3 & 4 bytes)
    let out: Vec<(usize, char)> = "[$¢€𐍈]".icu_chars(0).collect();
    assert_eq!(out, vec!((0, '['), (1, '$'), (2, '¢'), (4, '€'), (7, '𐍈'), (11, ']')));
  }

  #[test]
  fn relative_offset() {
    let out: Vec<(usize, char)> = "💖🇷🇺🇸🇹".icu_chars(8).collect();
    assert_eq!(out, vec!((0, '🇺'), (4, '🇸'), (8, '🇹')));
  }

  #[test]
  fn empty() {
    let out: Vec<(usize, char)> = "".icu_chars(0).collect();
    assert_eq!(out, vec!());
  }

  #[test]
  fn start_at_end() {
    let out: Vec<(usize, char)> = "💖🇷🇺🇸🇹".icu_chars(20).collect();
    assert_eq!(out, vec!());
  }

  #[test]
  #[should_panic]
  fn bad_index() {
    "💖".icu_chars(2);
  }

  #[test]
  #[should_panic]
  fn bad_index_oob() {
    "💖".icu_chars(5);
  }
}
