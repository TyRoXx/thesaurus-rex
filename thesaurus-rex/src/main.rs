enum RegularLanguage<A> {
    Empty,
    Singleton(A),
    // Kleene Star
    Repetition(Box<RegularLanguage<A>>),
    Union(Box<RegularLanguage<A>>, Box<RegularLanguage<A>>),
    Concatenation(Box<RegularLanguage<A>>, Box<RegularLanguage<A>>),
}

fn try_match<'t, A>(language: &RegularLanguage<A>, word: &'t [A]) -> Option<&'t [A]>
where
    A: core::cmp::PartialEq<A>,
{
    match language {
        RegularLanguage::Empty => None,
        RegularLanguage::Singleton(character) => match word {
            [head, ..] if (character == head) => Some(word.split_at(1).1),
            _ => None,
        },
        RegularLanguage::Repetition(element) => {
            let mut tail = word;
            loop {
                let element_matched = try_match(&element, tail);
                match element_matched {
                    Some(new_tail) => {
                        tail = new_tail;
                    }
                    None => {
                        break;
                    }
                }
            }
            Some(tail)
        }
        RegularLanguage::Union(first, second) => {
            let first_match = try_match(first, word);
            let second_match = try_match(second, word);
            match (first_match, second_match) {
                (None, None) => None,
                (None, Some(tail)) => Some(tail),
                (Some(tail), None) => Some(tail),
                (Some(first_tail), Some(second_tail)) => {
                    if second_tail.len() < first_tail.len() {
                        Some(&second_tail)
                    } else {
                        Some(&first_tail)
                    }
                }
            }
        }
        RegularLanguage::Concatenation(first, second) => {
            let first_match = try_match(&first, word);
            match first_match {
                Some(tail) => {
                    let second_match = try_match(&second, tail);
                    second_match
                }
                None => None,
            }
        }
    }
}

fn is_match<A>(language: &RegularLanguage<A>, word: &[A]) -> bool
where
    A: core::cmp::PartialEq<A>,
{
    match try_match(language, word) {
        Some(tail) => tail.is_empty(),
        None => false,
    }
}

#[test]
fn match_empty_language() {
    let language = RegularLanguage::<char>::Empty;
    assert!(!is_match(&language, &[]));
    assert!(!is_match(&language, &['a']));
    assert!(!is_match(&language, &['a', 'b']));
}

#[test]
fn match_empty_word_language() {
    let language = RegularLanguage::<char>::Repetition(Box::new(RegularLanguage::Empty));
    assert!(is_match(&language, &[]));
}

#[test]
fn match_singleton() {
    let language = RegularLanguage::<char>::Singleton('a');
    assert!(!is_match(&language, &[]));
    assert!(is_match(&language, &['a']));
    assert!(!is_match(&language, &['b']));
    assert!(!is_match(&language, &['a', 'a']));
}

#[test]
fn match_repeated_singleton() {
    let language =
        RegularLanguage::<char>::Repetition(Box::new(RegularLanguage::<char>::Singleton('a')));
    assert!(is_match(&language, &[]));
    assert!(is_match(&language, &['a']));
    assert!(is_match(&language, &['a', 'a']));
    assert!(is_match(&language, &['a', 'a', 'a']));
    assert!(!is_match(&language, &['b']));
    assert!(!is_match(&language, &['a', 'a', 'a', 'b']));
}

#[test]
fn match_union_simple() {
    let language = RegularLanguage::<char>::Union(
        Box::new(RegularLanguage::<char>::Singleton('a')),
        Box::new(RegularLanguage::<char>::Singleton('b')),
    );
    assert!(!is_match(&language, &[]));
    assert!(is_match(&language, &['a']));
    assert!(is_match(&language, &['b']));
    assert!(!is_match(&language, &['A']));
    assert!(!is_match(&language, &['z']));
    assert!(!is_match(&language, &['a', 'a']));
    assert!(!is_match(&language, &['b', 'a']));
    assert!(!is_match(&language, &['b', 'b']));
    assert!(!is_match(&language, &['b', 'b']));
}

#[test]
fn match_union_longer_match_wins() {
    let language = RegularLanguage::<char>::Union(
        Box::new(RegularLanguage::<char>::Singleton('a')),
        Box::new(RegularLanguage::<char>::Repetition(Box::new(
            RegularLanguage::<char>::Singleton('a'),
        ))),
    );
    assert!(is_match(&language, &['a', 'a']));
    assert!(is_match(&language, &[]));
    assert!(is_match(&language, &['a']));
    assert!(!is_match(&language, &['b']));
    assert!(!is_match(&language, &['a', 'b']));
    assert!(is_match(&language, &['a', 'a', 'a']));
}

#[test]
fn match_concatenation() {
    let language = RegularLanguage::<char>::Concatenation(
        Box::new(RegularLanguage::<char>::Singleton('a')),
        Box::new(RegularLanguage::<char>::Singleton('b')),
    );
    assert!(!is_match(&language, &[]));
    assert!(!is_match(&language, &['a']));
    assert!(!is_match(&language, &['b']));
    assert!(is_match(&language, &['a', 'b']));
    assert!(!is_match(&language, &['a', 'b', 'b']));
    assert!(!is_match(&language, &['a', 'a']));
    assert!(!is_match(&language, &['b', 'a']));
}

fn main() {
    println!("Hello, world!");
}
