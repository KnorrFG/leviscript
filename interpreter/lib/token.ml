open Core

type t = 
  (* misc *)
  | Eof
  | XExStart
  | XExEnd
  | PathLit of string
  | StrLit of string
  | Symbol of string
[@@deriving sexp, eq]


let (=) = equal
