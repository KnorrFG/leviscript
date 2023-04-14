%{
  open Ast	
%} 

%token Eof
%token XExStart
%token XExEnd
%token <string> PathLit
%token <string> StrLit
%token <string> Symbol

%start <Ast.block option> prog
%%

prog:
  | Eof { None }
  | b = block; Eof { Some b }
;

block:
  | stmts = rev_statements { `Block (List.rev stmts) }
;

rev_statements:
  | (*empty*) { [] }
  | stmts = rev_statements; xexpr = xexpression { `Expr xexpr :: stmts }
;

xexpression: XExStart; exe = xexpr_arg; children = rev_xexpr_children; XExEnd 
	{ `XExpr { exec = exe; args = List.rev children } }
;

xexpr_arg:
  | a = PathLit { `StrLit a }
  | a = Symbol { `Symbol a }
;

rev_xexpr_children:
  | (*empty*) { [] }
  | rev_children = rev_xexpr_children; arg = xexpr_arg { arg :: rev_children }
;
