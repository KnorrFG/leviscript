use pest::error::Error;
use pest::{Parser, Span};
use pest_derive::Parser;

use crate::ast::*;
use crate::utils;

use std::matches;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct LsParser;

pub type SpanVec<'a> = Vec<Span<'a>>;
pub type ParseResult<T> = Result<T, Error<Rule>>;

pub type Pair<'a> = pest::iterators::Pair<'a, Rule>;
pub type Pairs<'a> = pest::iterators::Pairs<'a, Rule>;

pub fn to_ast(mut pairs: Pairs) -> ParseResult<(Block, SpanVec)> {
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

    let phrases = utils::sequence_result(pair.into_inner().map(|p| parse_phrase(p, span_vec)))?;
    Ok(Block(id, phrases))
}

fn parse_phrase<'a>(pair: Pair<'a>, span_vec: &mut SpanVec<'a>) -> ParseResult<Phrase> {
    let id = span_vec.len();
    span_vec.push(pair.as_span());
    assert!(matches!(pair.as_rule(), Rule::phrase));

    let child = get_single_child(pair.into_inner());
    match child.as_rule() {
        Rule::expression => Ok(Phrase::Expr(id, parse_expression(child, span_vec)?)),
        _ => unreachable!(),
    }
}

fn parse_expression<'a>(pair: Pair<'a>, span_vec: &mut SpanVec<'a>) -> ParseResult<Expr> {
    span_vec.push(pair.as_span());
    assert!(matches!(pair.as_rule(), Rule::expression));

    let child = get_single_child(pair.into_inner());
    match child.as_rule() {
        Rule::x_expression => parse_x_expression(child, span_vec),
        Rule::str_lit => parse_str_lit(child, span_vec),
        Rule::let_expr => parse_let_expr(child, span_vec),
        _ => unreachable!(),
    }
}

fn parse_let_expr<'a>(pair: Pair<'a>, span_vec: &mut SpanVec<'a>) -> ParseResult<Expr> {
    let id = span_vec.len();
    span_vec.push(pair.as_span());
    assert!(matches!(pair.as_rule(), Rule::let_expr));

    let mut children = pair.into_inner();
    Ok(Expr::Let {
        id,
        symbol_name: children.next().unwrap().as_str().into(),
        value_expr: Box::new(parse_expression(children.next().unwrap(), span_vec)?),
    })
}

fn parse_str_lit<'a>(pair: Pair<'a>, span_vec: &mut SpanVec<'a>) -> ParseResult<Expr> {
    let id = span_vec.len();
    span_vec.push(pair.as_span());
    assert!(matches!(pair.as_rule(), Rule::str_lit));

    // A strlit is either quoted or single_quoted, that makes a different in the grammar,
    // but the structure is the same, so we just go one lvl deeper
    let pair = get_single_child(pair.into_inner());
    let children = pair
        .into_inner()
        .map(|p| parse_str_lit_elem(p, span_vec))
        .collect::<Result<_, _>>()?;
    Ok(Expr::StrLit(id, children))
}

fn parse_str_lit_elem<'a>(pair: Pair<'a>, span_vec: &mut SpanVec<'a>) -> ParseResult<StrLitElem> {
    let id = span_vec.len();
    span_vec.push(pair.as_span());
    assert!(matches!(pair.as_rule(), Rule::quoted_str_lit_elem));

    let child = get_single_child(pair.into_inner());
    use StrLitElem::*;
    Ok(match child.as_rule() {
        Rule::pure_quoted_str_lit => PureStrLit(id, child.as_str().into()),
        Rule::symbol => Symbol(id, child.as_str().into()),
        Rule::sub_expr => SubExpr(id, parse_expression(child, span_vec)?),
        _ => unreachable!(),
    })
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
