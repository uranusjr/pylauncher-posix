extern crate num_integer;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum SpecPart<T> {
    Number(T),
    NumberDot(T),
    Invalid,
}

pub fn parse_spec_part<'a, T: num_integer::Integer + Copy>(
    bytes: &mut Iterator<Item = &'a u8>) -> SpecPart<T> {

    let mut empty = true;
    let mut number = T::zero();

    for byte in bytes {
        match byte {
            b'0'...b'9' => {
                // Times 10, and add new digit.
                let mut n = number;
                for _ in 0..9 { number = number + n; }
                for _ in 0..(byte - b'0') { number = number + T::one(); }
            },
            b'.' => if empty {
                return SpecPart::Invalid;
            } else {
                return SpecPart::NumberDot(number);
            },
            _ => { return SpecPart::Invalid },
        }
        empty = false;
    }

    if empty {
        SpecPart::Invalid
    } else {
        SpecPart::Number(number)
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Spec {
    Major(u8),      // Specifies only the major version.
    Minor(u8, u8),  // Specifiers major and minor version.
}

pub fn parse_spec(bytes: &[u8]) -> Option<Spec> {
    let mut bytes = bytes.iter();

    // The spec options should be either "-X" or "-X.Y".
    if bytes.next() != Some(&b'-') {
        return None;
    }

    // Parse the X part. Return it unless a trailing dot follows.
    let major = match parse_spec_part(&mut bytes) {
        SpecPart::Invalid => { return None; },
        SpecPart::Number(n) => { return Some(Spec::Major(n)); },
        SpecPart::NumberDot(n) => n,
    };

    // Parse the Y part.
    match parse_spec_part(&mut bytes) {
        SpecPart::Invalid | SpecPart::NumberDot(_) => None,
        SpecPart::Number(n) => Some(Spec::Minor(major, n)),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec_part_number() {
        assert_eq!(parse_spec_part(&mut b"2".iter()), SpecPart::Number(2));
        assert_eq!(parse_spec_part(&mut b"12".iter()), SpecPart::Number(12));
    }

    #[test]
    fn test_parse_spec_part_number_dot() {
        assert_eq!(parse_spec_part(&mut b"3.".iter()),
                   SpecPart::NumberDot(3));
        assert_eq!(parse_spec_part(&mut b"23.".iter()),
                   SpecPart::NumberDot(23));
        assert_eq!(parse_spec_part(&mut b"3..".iter()),
                   SpecPart::NumberDot(3));
        assert_eq!(parse_spec_part(&mut b"23..".iter()),
                   SpecPart::NumberDot(23));
    }

    #[test]
    fn test_parse_spec_part_only_dot() {
        assert_eq!(parse_spec_part::<u8>(&mut b"...".iter()), SpecPart::Invalid);
    }

    #[test]
    fn test_parse_spec_major() {
        assert_eq!(parse_spec(b"-3"), Some(Spec::Major(3)));
        assert_eq!(parse_spec(b"-0"), Some(Spec::Major(0)));
    }

    #[test]
    fn test_parse_spec_minor() {
        assert_eq!(parse_spec(b"-3.4"), Some(Spec::Minor(3, 4)));
        assert_eq!(parse_spec(b"-3.0"), Some(Spec::Minor(3, 0)));
    }

    #[test]
    fn test_parse_spec_trailing_garbage() {
        assert_eq!(parse_spec(b"-3.4-32"), None);
    }

    #[test]
    fn test_parse_spec_wrong_dash() {
        assert_eq!(parse_spec(b"--3.4"), None);
        assert_eq!(parse_spec(b"3.4"), None);
        assert_eq!(parse_spec(b"-"), None);
    }

    #[test]
    fn test_parse_spec_trailing_dot() {
        assert_eq!(parse_spec(b"-3.4."), None);
        assert_eq!(parse_spec(b"-3."), None);
        assert_eq!(parse_spec(b"-."), None);
    }
}
