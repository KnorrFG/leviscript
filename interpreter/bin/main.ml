open Leviscript
open Stdio
open Base
open Cmdliner

let to_list f = 
  let rec loop res =
    let (token, _, _) = f () in
    if Token.(token = Eof) then
      List.rev res
    else
      loop @@ token :: res
  in
  loop []
    

let main path = 
  let lexer = Lexer.mk_lexer @@ Sedlexing.Utf8.from_channel @@ In_channel.create path  in
  printf "token tree:\n%s\n" 
    @@ Sexp.to_string_hum 
    @@ List.sexp_of_t Token.sexp_of_t 
    @@ to_list lexer
    

let script_path = Arg.(required & pos 0 (some file) None & info [] ~docv:"SCRIPT")
let main_t = Term.(const main $ script_path)
let cmd = Cmd.v (Cmd.info "levs") main_t
let () = Caml.exit (Cmd.eval cmd)
