use std::cmp::{max, min};
use std::collections::BTreeMap;

use miette::{Diagnostic, Result};
use pest::error::InputLocation;
use pest::Parser;

use crate::data::program::InputProgram;
use crate::data::value::DataValue;
use crate::parse::query::parse_query;
use crate::parse::sys::{parse_sys, SysOp};

pub(crate) mod expr;
pub(crate) mod query;
pub(crate) mod sys;
pub(crate) mod schema;

#[derive(pest_derive::Parser)]
#[grammar = "cozoscript.pest"]
pub(crate) struct CozoScriptParser;

pub(crate) type Pair<'a> = pest::iterators::Pair<'a, Rule>;
pub(crate) type Pairs<'a> = pest::iterators::Pairs<'a, Rule>;

pub(crate) enum CozoScript {
    Query(InputProgram),
    Sys(SysOp),
}

#[derive(Eq, PartialEq, Debug, serde_derive::Serialize, serde_derive::Deserialize, Copy, Clone)]
pub(crate) struct SourceSpan(pub(crate) usize, pub(crate) usize);

impl SourceSpan {
    pub(crate) fn merge(self, other: Self) -> Self {
        let s1 = self.0;
        let e1 = self.0 + self.1;
        let s2 = other.0;
        let e2 = other.0 + other.1;
        let s = min(s1, s2);
        let e = max(e1, e2);
        Self(s, e - s)
    }
}

impl From<&'_ SourceSpan> for miette::SourceSpan {
    fn from(s: &'_ SourceSpan) -> Self {
        miette::SourceSpan::new(s.0.into(), s.1.into())
    }
}

impl From<SourceSpan> for miette::SourceSpan {
    fn from(s: SourceSpan) -> Self {
        miette::SourceSpan::new(s.0.into(), s.1.into())
    }
}

#[derive(thiserror::Error, Diagnostic, Debug)]
#[error("The query parser has encountered unexpected input / end of input")]
#[diagnostic(code(parser::pest))]
pub(crate) struct ParseError {
    #[label]
    pub(crate) span: SourceSpan,
}

pub(crate) fn parse_script(
    src: &str,
    param_pool: &BTreeMap<String, DataValue>,
) -> Result<CozoScript> {
    let parsed = CozoScriptParser::parse(Rule::script, src)
        .map_err(|err| {
            let span = match err.location {
                InputLocation::Pos(p) => SourceSpan(p, 0),
                InputLocation::Span((start, end)) => SourceSpan(start, end - start),
            };
            ParseError { span }
        })?
        .next()
        .unwrap();
    Ok(match parsed.as_rule() {
        Rule::query_script => CozoScript::Query(parse_query(parsed.into_inner(), param_pool)?),
        Rule::sys_script => CozoScript::Sys(parse_sys(parsed.into_inner())?),
        _ => unreachable!(),
    })
}

trait ExtractSpan {
    fn extract_span(&self) -> SourceSpan;
}

impl ExtractSpan for Pair<'_> {
    fn extract_span(&self) -> SourceSpan {
        let span = self.as_span();
        let start = span.start();
        let end = span.end();
        SourceSpan(start, end - start)
    }
}
