#![no_std]

#![feature(unicode)]

extern crate either;
extern crate std_unicode;

#[cfg(test)]
#[macro_use]
extern crate std;

use core::iter::{Filter, FlatMap, Once, once};
use either::Either;
use std_unicode::char::{ToLowercase, ToUppercase};

use self::Either::{Left, Right};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Case {
    Camel(bool),
    Snake(bool),
}

use self::Case::*;

#[derive(Clone, Debug)]
pub struct Convert<Xs>(Either<FlatMap<Filter<ToCamelCase<Xs>, fn(&(Option<bool>, char)) -> bool>,
                                      Either<Once<char>, ToUppercase>,
                                      fn((Option<bool>, char)) -> Either<Once<char>, ToUppercase>>,
                              FlatMap<ToSnakeCase<Xs>,
                                      Either<ToLowercase, ToUppercase>,
                                      fn(char) -> Either<ToLowercase, ToUppercase>>>);

impl<Xs: Iterator<Item = char>> Convert<Xs> {
    #[inline]
    pub fn new(xs: Xs, c: Case) -> Self {
        Convert(match c {
            Camel(u) => Left (ToCamelCase { xs, last_x_opt: None }
                                  .filter({ fn f(&(_, x): &(Option<bool>, char)) -> bool { '_' != x }; f as _ })
                                  .flat_map(if u { (|(v, x): (Option<bool>, char)|
                                                    if v.unwrap_or(true)  { Right(x.to_uppercase()) }
                                                    else { Left(once(x)) }) as _ }
                                            else { (|(v, x): (Option<bool>, char)|
                                                    if v.unwrap_or(false) { Right(x.to_uppercase()) }
                                                    else { Left(once(x)) }) as _ })),
            Snake(u) => Right(ToSnakeCase { xs, x_opt: Right(false) }.flat_map(if u { (|x: char| Right(x.to_uppercase())) as _ }
                                                                               else { (|x: char| Left (x.to_lowercase())) as _ })),
        })
    }
}

impl<Xs: Iterator<Item = char>> Iterator for Convert<Xs> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> { self.0.next() }
}

#[derive(Clone, Debug)]
struct ToSnakeCase<Xs> {
    xs: Xs,
    x_opt: Either<char, bool>,
}

impl<Xs: Iterator<Item = char>> Iterator for ToSnakeCase<Xs> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        match self.x_opt {
            Left(x) => { self.x_opt = Right(true); Some(x) },
            Right(b) => self.xs.next().map(|x| if x.is_uppercase() && b { self.x_opt = Left(x); '_' }
                                               else { self.x_opt = Right(x.is_lowercase()); x }),
        }
    }
}

#[derive(Clone, Debug)]
struct ToCamelCase<Xs> {
    xs: Xs,
    last_x_opt: Option<char>,
}

impl<Xs: Iterator<Item = char>> Iterator for ToCamelCase<Xs> {
    type Item = (Option<bool>, char);

    #[inline]
    fn next(&mut self) -> Option<(Option<bool>, char)> {
        let last_x_opt = self.last_x_opt;
        self.last_x_opt = self.xs.next();
        self.last_x_opt.map(|x| (last_x_opt.map(|x| '_' == x), x))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert!(Iterator::eq("camelCase".chars(), Convert::new("camel_case".chars(), Camel(false))));
        assert!(Iterator::eq("camelCase".chars(), Convert::new("camelCase".chars(), Camel(false))));
        assert!(Iterator::eq("CamelCase".chars(), Convert::new("camel_case".chars(), Camel(true))));
        assert!(Iterator::eq("CamelCase".chars(), Convert::new("camelCase".chars(), Camel(true))));
        assert!(Iterator::eq("CamelCase".chars(), Convert::new("CamelCase".chars(), Camel(true))));
        assert!(Iterator::eq("snake_case".chars(), Convert::new("snake_case".chars(), Snake(false))));
        assert!(Iterator::eq("snake_case".chars(), Convert::new("SNAKE_CASE".chars(), Snake(false))));
        assert!(Iterator::eq("snake_case".chars(), Convert::new("snakeCase".chars(), Snake(false))));
        assert!(Iterator::eq("snake_case".chars(), Convert::new("SnakeCase".chars(), Snake(false))));
        assert!(Iterator::eq("SNAKE_CASE".chars(), Convert::new("snake_case".chars(), Snake(true))));
        assert!(Iterator::eq("SNAKE_CASE".chars(), Convert::new("SNAKE_CASE".chars(), Snake(true))));
        assert!(Iterator::eq("SNAKE_CASE".chars(), Convert::new("snakeCase".chars(), Snake(true))));
        assert!(Iterator::eq("SNAKE_CASE".chars(), Convert::new("SnakeCase".chars(), Snake(true))));
    }
}
