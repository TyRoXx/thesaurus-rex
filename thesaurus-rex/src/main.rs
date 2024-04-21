#![feature(coroutines, coroutine_trait)]
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Coroutine;
use std::ops::CoroutineState;
use std::pin::Pin;

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

pub fn gen_to_iter<A, G: Coroutine<Return = (), Yield = A> + Unpin>(
    gen2: G,
) -> impl Iterator<Item = A> {
    CoroutineIter {
        state: CoroutineIterState::Pending,
        gen2,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CoroutineIter<G: Coroutine<Return = ()>> {
    state: CoroutineIterState,
    gen2: G,
}

#[derive(Debug, PartialEq, Eq)]
enum CoroutineIterState {
    Pending,
    Empty,
}

impl<G: Coroutine<Return = ()> + Unpin> Iterator for CoroutineIter<G> {
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            CoroutineIterState::Empty => None,
            CoroutineIterState::Pending => match Pin::new(&mut self.gen2).resume(()) {
                CoroutineState::Yielded(value) => Some(value),
                CoroutineState::Complete(_) => {
                    self.state = CoroutineIterState::Empty;
                    None
                }
            },
        }
    }
}

fn try_match<'t, A: Hash>(
    language: &'t RegularLanguage<A>,
    word: &'t [A],
) -> impl Iterator<Item = &'t [A]> + 't
where
    A: core::cmp::Eq,
{
    gen_to_iter(move || {
        match language {
            RegularLanguage::Empty => {}
            RegularLanguage::Singleton(character) => match word {
                [head, ..] if (character == head) => {
                    yield word.split_at(1).1;
                }
                _ => {}
            },
            RegularLanguage::Repetition(element) => {
                let mut results = HashSet::new();
                let mut stack = vec![vec![word]];
                loop {
                    match &mut stack.last_mut() {
                        Some(last_element) => {
                            let next_match = last_element.pop();
                            match next_match {
                                Some(matched) => {
                                    if results.insert(matched) {
                                        yield matched;
                                    }
                                    let inner_generator = Box::new(try_match(&element, matched));
                                    let element_matches: Vec<&'t [A]> = inner_generator
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
            }
            RegularLanguage::Union(first, second) => {
                let first_matches = Box::new(try_match(first, word));
                let mut already_yielded = HashSet::new();
                for first_match in first_matches {
                    yield first_match;
                    already_yielded.insert(first_match);
                }
                let second_matches = Box::new(try_match(second, word));
                for second_match in second_matches {
                    if already_yielded.contains(second_match) {
                        continue;
                    }
                    yield second_match;
                }
            }
            RegularLanguage::Concatenation(first, second) => {
                let first_matches = Box::new(try_match(&first, word));
                for first_match in first_matches {
                    let second_matches = Box::new(try_match(&second, first_match));
                    for second_match in second_matches {
                        yield second_match;
                    }
                }
            }
        }
    })
}

fn is_match<A: Hash>(language: &RegularLanguage<A>, word: &[A]) -> bool
where
    A: core::cmp::Eq,
{
    let mut matches = try_match(language, word);
    matches.find(|element| element.is_empty()).is_some()
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

#[test]
fn repeated_ambiguity() {
    let language = RegularLanguage::<char>::Repetition(Box::new(RegularLanguage::<char>::Union(
        Box::new(RegularLanguage::<char>::Concatenation(
            Box::new(RegularLanguage::<char>::Singleton('a')),
            Box::new(RegularLanguage::<char>::Singleton('a')),
        )),
        Box::new(RegularLanguage::<char>::Singleton('a')),
    )));
    assert!(is_match(&language, &[]));
    assert!(is_match(&language, &['a']));
    assert!(!is_match(&language, &['b']));
    assert!(is_match(&language, &['a', 'a']));
    assert!(is_match(&language, &['a'; 10]));
    assert!(is_match(&language, &['a'; 20]));
    assert!(is_match(&language, &['a'; 30]));
    assert!(is_match(&language, &['a'; 100]));
    assert!(is_match(&language, &['a'; 1000]));
    // TODO: make this fast:
    // assert!(is_match(&language, &['a'; 10000]));
}

fn main() {
    println!("Hello, world!");
}
