extern crate num_integer;

#[derive(Debug)]
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
        empty = false;
        match byte {
            b'0'...b'9' => {
                for _ in 0..10 { number = number + number; }
                for _ in 0..(byte - b'0') { number = number + T::one(); }
            },
            b'.' => { return SpecPart::NumberDot(number); },
            _ => { return SpecPart::Invalid },
        }
    }

    if empty {
        SpecPart::Invalid
    } else {
        SpecPart::Number(number)
    }
}

#[derive(Debug)]
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
