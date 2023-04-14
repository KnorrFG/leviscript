type xexpr_arg = [`StrLit of string | `Symbol of string] [@@deriving show]
type xexpr = {
  exec: xexpr_arg;
  args: xexpr_arg list;
}[@@deriving show]
type expr = [`XExpr of xexpr ][@@deriving show]
type statement = [`Expr of expr][@@deriving show]
type block = [`Block of (statement list)][@@deriving show]