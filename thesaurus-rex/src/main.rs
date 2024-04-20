enum RegularLanguage<A> {
    Empty,
    Singleton(A),
    // Kleene Star
    Repetition(Box<RegularLanguage<A>>),
    Union(Box<RegularLanguage<A>>, Box<RegularLanguage<A>>),
    Concatenation(Box<RegularLanguage<A>>, Box<RegularLanguage<A>>),
}

fn new_empty_word<A>() -> RegularLanguage<A> {
    RegularLanguage::<A>::Repetition(Box::new(RegularLanguage::Empty))
}

fn try_match<'t, A>(language: &RegularLanguage<A>, word: &'t [A]) -> Vec<&'t [A]>
where
    A: core::cmp::PartialEq<A>,
{
    match language {
        RegularLanguage::Empty => Vec::new(),
        RegularLanguage::Singleton(character) => match word {
            [head, ..] if (character == head) => vec![word.split_at(1).1],
            _ => Vec::new(),
        },
        RegularLanguage::Repetition(element) => {
            // TODO: deduplicate results
            let mut results = Vec::new();
            let mut stack = vec![vec![word]];
            loop {
                match &mut stack.last_mut() {
                    Some(last_element) => {
                        let next_match = last_element.pop();
                        match next_match {
                            Some(matched) => {
                                results.push(matched);
                                let element_matches: Vec<&'t [A]> = try_match(&element, matched)
                                    .drain(..)
                                    // avoid infinite loop by discarding empty word matches
                                    .filter(|matched_tail| matched_tail.len() < matched.len())
                                    .collect();
                                if !element_matches.is_empty() {
                                    stack.push(element_matches);
                                }
                            }
                            None => {
                                stack.pop();
                            }
                        }
                    }
                    None => break,
                }
            }
            results
        }
        RegularLanguage::Union(first, second) => {
            let mut first_match = try_match(first, word);
            let mut second_match = try_match(second, word);
            first_match.append(&mut second_match);
            first_match
        }
        RegularLanguage::Concatenation(first, second) => {
            let mut results = Vec::new();
            let first_matches = try_match(&first, word);
            for first_match in first_matches {
                let mut second_matches = try_match(&second, first_match);
                results.append(&mut second_matches);
            }
            results
        }
    }
}

fn is_match<A>(language: &RegularLanguage<A>, word: &[A]) -> bool
where
    A: core::cmp::PartialEq<A>,
{
    let matches = try_match(language, word);
    matches.iter().find(|element| element.is_empty()).is_some()
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
    let language: RegularLanguage<char> = new_empty_word();
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
fn match_repeated_empty_word() {
    let language = RegularLanguage::<char>::Concatenation(
        Box::new(RegularLanguage::<char>::Repetition(Box::new(
            new_empty_word(),
        ))),
        Box::new(RegularLanguage::<char>::Singleton('a')),
    );
    assert!(!is_match(&language, &[]));
    assert!(is_match(&language, &['a']));
    assert!(!is_match(&language, &['b']));
    assert!(!is_match(&language, &['a', 'a']));
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

#[test]
fn consider_empty_word_match() {
    let language = RegularLanguage::<char>::Concatenation(
        Box::new(RegularLanguage::<char>::Union(
            Box::new(RegularLanguage::<char>::Singleton('a')),
            Box::new(new_empty_word()),
        )),
        Box::new(RegularLanguage::<char>::Singleton('a')),
    );
    assert!(!is_match(&language, &[]));
    assert!(is_match(&language, &['a']));
    assert!(!is_match(&language, &['b']));
    assert!(is_match(&language, &['a', 'a']));
    assert!(!is_match(&language, &['a', 'b']));
    assert!(!is_match(&language, &['a', 'a', 'a']));
}

#[test]
fn consider_non_empty_match() {
    let language = RegularLanguage::<char>::Concatenation(
        Box::new(RegularLanguage::<char>::Union(
            Box::new(RegularLanguage::<char>::Concatenation(
                Box::new(RegularLanguage::<char>::Singleton('a')),
                Box::new(RegularLanguage::<char>::Singleton('a')),
            )),
            Box::new(RegularLanguage::<char>::Singleton('a')),
        )),
        Box::new(RegularLanguage::<char>::Singleton('a')),
    );
    assert!(!is_match(&language, &[]));
    assert!(!is_match(&language, &['a']));
    assert!(!is_match(&language, &['b']));
    assert!(is_match(&language, &['a', 'a']));
    assert!(is_match(&language, &['a', 'a', 'a']));
    assert!(!is_match(&language, &['a', 'a', 'b']));
    assert!(!is_match(&language, &['a', 'a', 'a', 'a']));
}

fn main() {
    println!("Hello, world!");
}
