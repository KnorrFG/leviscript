open! Core
open Sedlexing
open Utils

open Parser

exception LexingError of string

let blank = [%sedlex.regexp? ' ' | '\t']
let newline_char = [%sedlex.regexp? '\r' | '\n']
let newline = [%sedlex.regexp? newline_char | "\r\n"]
let any_blank = [%sedlex.regexp? blank | newline]
let symbol = [%sedlex.regexp? (ll | lo), Star (ll | lu | lo | nd | pc | '\'')  ]

let xexpr_forbidden = [%sedlex.regexp? blank| newline_char | '}']
let xexpr_word = [%sedlex.regexp? Plus (Compl xexpr_forbidden)]


let rec nom buf =
  match%sedlex buf with
  | Plus any_blank -> nom buf
  | _ -> ()


type lexer_type = 
  | LtMain 
  | LtXExHead | LtXExTail


let unexpected buf =
  raise (LexingError (Utf8.lexeme buf))


let get_token_main buf =
  nom buf;
  match%sedlex buf with
  | eof -> LtMain, Eof
  | "x{" -> LtXExHead, XExStart
  | _ -> unexpected buf


let get_token_xexpr_head buf =
  nom buf;
  match%sedlex buf with
  | '$', symbol -> LtXExTail, Symbol (Utf8.lexeme buf |> str_tail)
  | xexpr_word -> LtXExTail, StrLit(Utf8.lexeme buf)
  | _ -> unexpected buf


let get_token_xexpr_tail buf =
  nom buf;
  match%sedlex buf with 
  | ('$', symbol) -> LtXExTail, Symbol (Utf8.lexeme buf |> str_tail)
  | xexpr_word -> LtXExTail, StrLit (Utf8.lexeme buf)
  | '}' -> LtMain, XExEnd
  | _ -> unexpected buf


let lexer_type_to_fun = function
  | LtMain -> get_token_main
  | LtXExHead -> get_token_xexpr_head
  | LtXExTail -> get_token_xexpr_tail


let mk_lexer lexbuf = 
  let current_lexer_type = ref LtMain in
  let lexer () =
    let get_token = lexer_type_to_fun !current_lexer_type in
    let next_type, token = get_token lexbuf in
    let start_p, curr_p = lexing_positions lexbuf in
    current_lexer_type := next_type;
    (token, start_p, curr_p)
  in
  lexer
