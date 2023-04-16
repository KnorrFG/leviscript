use pest::error::Error;
use pest::{Parser, Span};
use pest_derive::Parser;

use crate::ast::*;
use crate::utils;

use std::matches;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LsParser;

pub type SpanVec<'a> = Vec<Span<'a>>;
pub type ParseResult<T> = Result<T, Error<Rule>>;

pub type Pair<'a> = pest::iterators::Pair<'a, Rule>;
pub type Pairs<'a> = pest::iterators::Pairs<'a, Rule>;

pub fn parse(src: &str) -> ParseResult<(Block, SpanVec)> {
    let mut pairs = LsParser::parse(Rule::file, src)?;
    let mut span_vec = vec![];
    let block_pair = pairs.next().unwrap();
    assert!(matches!(pairs.next().unwrap().as_rule(), Rule::EOI));
    let res = parse_block(block_pair, &mut span_vec)?;
    Ok((res, span_vec))
}

fn parse_block<'a>(pair: Pair<'a>, span_vec: &mut SpanVec<'a>) -> ParseResult<Block> {
    let id = span_vec.len();
    span_vec.push(pair.as_span());
    assert!(matches!(pair.as_rule(), Rule::block));

    let terms = utils::sequence_result(pair.into_inner().map(|p| parse_term(p, span_vec)))?;
    Ok(Block(id, terms))
}

fn parse_term<'a>(pair: Pair<'a>, span_vec: &mut SpanVec<'a>) -> ParseResult<Term> {
    let id = span_vec.len();
    span_vec.push(pair.as_span());
    assert!(matches!(pair.as_rule(), Rule::term));

    let child = get_single_child(pair.into_inner());
    match child.as_rule() {
        Rule::expression => Ok(Term::Expr(id, parse_expression(child, span_vec)?)),
        _ => unreachable!(),
    }
}

fn parse_expression<'a>(pair: Pair<'a>, span_vec: &mut SpanVec<'a>) -> ParseResult<Expr> {
    span_vec.push(pair.as_span());
    assert!(matches!(pair.as_rule(), Rule::expression));

    let child = get_single_child(pair.into_inner());
    match child.as_rule() {
        Rule::x_expression => parse_x_expression(child, span_vec),
        _ => unreachable!(),
    }
}

fn parse_x_expression<'a>(pair: Pair<'a>, span_vec: &mut SpanVec<'a>) -> ParseResult<Expr> {
    let id = span_vec.len();
    span_vec.push(pair.as_span());
    assert!(matches!(pair.as_rule(), Rule::x_expression));

    let children =
        utils::sequence_result(pair.into_inner().map(|p| parse_xexpr_atom(p, span_vec)))?;

    Ok(Expr::XExpr {
        exe: children[0].clone(),
        args: children[1..].to_vec(),
        id,
    })
}

fn parse_xexpr_atom<'a>(pair: Pair<'a>, span_vec: &mut SpanVec<'a>) -> ParseResult<XExprAtom> {
    let id = span_vec.len();
    span_vec.push(pair.as_span());
    assert!(matches!(pair.as_rule(), Rule::xexpr_elem));

    let child = get_single_child(pair.into_inner());
    Ok(match child.as_rule() {
        Rule::symbol => XExprAtom::Ref(id, child.as_str().into()),
        Rule::xexpr_str => XExprAtom::Str(id, child.as_str().into()),
        _ => unreachable!(),
    })
}

fn get_single_child(p: Pairs) -> Pair {
    let children: Vec<Pair> = p.collect();
    assert!(
        children.len() == 1,
        "get_single_child found {} children in {:#?}",
        children.len(),
        children
    );
    children[0].clone()
}
