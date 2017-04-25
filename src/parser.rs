#![allow(missing_docs)]

//! Parser for the OWNERS file format
//!
//! See https://chromium.googlesource.com/chromium/src/+/master/docs/code_reviews.md
//!
//! lines      := (\s* line? \s* "\n")*
//!
//! line       := directive | comment
//!
//! directive  := "set noparent"
//!               |  identifier (\s* glob)*
//!               |  "*"
//!
//! identifier := email_address
//!              | username
//!              | team
//!
//! username   := @[a-zA-Z0-9-]+
//!
//! team       := @[a-zA-Z0-9-]+/[a-zA-Z0-9-]+
//!
//! glob       := [a-zA-Z0-9_-*?]+
//!
//! comment    := "#" [^"\n"]*

use std::str;
use nom::{is_alphanumeric, space, multispace};

named!(pub lines<Vec<Directive>>,
  do_parse!(
    opt!(multispace) >>
    directives: separated_list!(multispace, directive) >>
    opt!(multispace) >>
    eof!() >>
    (directives)
  )
);

#[derive(Debug, PartialEq)]
pub enum Directive<'a> {
  SetNoParent,
  Identifier(Identifier<'a>, Option<Vec<&'a str>>),
  Star
}

named!(pub directive<Directive>,
  alt_complete!(
    identifier_plus_globs => { |(i, g)| Directive::Identifier(i, g) }
    | tag!("set noparent") => { |_| Directive::SetNoParent }
    | tag!("*") => { |_| Directive::Star }
  )
);

named!(identifier_plus_globs<(Identifier, Option<Vec<&str>>)>,
  tuple!(identifier, opt!(glob_list))
);

named!(glob_list<Vec<&str>>,
  do_parse!(
    space >>
    gs: separated_list!(space, glob_pattern) >>
    (gs)
  )
);

named!(glob_pattern<&str>,
  map_res!(
    is_not!(" \t\r\n"),
    str::from_utf8
  )
);

#[derive(Debug, PartialEq)]
pub enum Identifier<'a> {
  Username(&'a str),
  Team(&'a str, &'a str)
}

named!(identifier<Identifier>,
  alt_complete!(
    team => { |(o, t)| Identifier::Team(o, t) }
    | username => { |u| Identifier::Username(u) }
  )
);

named!(username<&str>,
  do_parse!(
    tag!("@") >>
    login: github_identifier >>
    (login)
  )
);

named!(team<(&str, &str)>,
  do_parse!(
    tag!("@") >>
    org: github_identifier >>
    tag!("/") >>
    team: github_identifier >>
    (org, team)
  )
);

named!(github_identifier<&str>,
  map_res!(
    take_while!(is_alphanumeric),
    str::from_utf8
  )
);

#[cfg(test)]
mod tests {
  use nom::IResult;
  use super::*;

  #[test]
  fn test_identifier_parses_username() {
    let input = &b"@aergonaut"[..];

    let result = identifier(input);
    let output = match result {
      IResult::Done(_, o) => o,
      _ => unreachable!()
    };

    assert_eq!(Identifier::Username("aergonaut"), output);
  }

  #[test]
  fn test_identifier_parses_team() {
    let input = &b"@testorg/testteam"[..];

    let result = identifier(input);
    let output = match result {
      IResult::Done(_, o) => o,
      _ => unreachable!()
    };

    assert_eq!(Identifier::Team("testorg", "testteam"), output);
  }

  #[test]
  fn test_lines() {
    let input = &b"
set noparent

@aergonaut *.rb

@testorg/testteam
"[..];

    let result = lines(input);

    let output = match result {
      IResult::Done(_, o) => o,
      _ => unreachable!()
    };

    assert_eq!(
      vec![
        Directive::SetNoParent,
        Directive::Identifier(Identifier::Username("aergonaut"), Some(vec!["*.rb"])),
        Directive::Identifier(Identifier::Team("testorg", "testteam"), None)
      ],
      output
    );
  }
}
