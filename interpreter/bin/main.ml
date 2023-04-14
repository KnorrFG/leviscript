open! Core
open Leviscript
open Stdio
open Cmdliner

(*let is_eof = function
  | Parser.Eof -> true
  | _ -> false

let to_list f =
  let rec loop res =
    let (token, _, _) = f () in
    if is_eof token then
      List.rev res
    else
      loop @@ token :: res
  in
  loop []*)

let main path = 
  let lexbuf = Sedlexing.Utf8.from_channel @@ In_channel.create path in
  let lexer = Lexer.mk_lexer lexbuf in
  (*let tokens = to_list lexer in*)
  let parser = MenhirLib.Convert.Simplified.traditional2revised Parser.prog in
  try
    let program = parser lexer in
    printf "AST:\n%s\n" 
      @@ [%derive.show: Ast.block option]
      @@ program
  with Parser.Error -> 
    let start, _ = Sedlexing.lexing_positions lexbuf in
    printf "Compiler Error at: line: %d, col: %d\n" start.pos_lnum start.pos_cnum
    

let script_path = Arg.(required & pos 0 (some file) None & info [] ~docv:"SCRIPT")
let main_t = Term.(const main $ script_path)
let cmd = Cmd.v (Cmd.info "levs") main_t
let () = Caml.exit (Cmd.eval cmd)
