open Leviscript
open Stdio
open Base
open Cmdliner

let main path = 
  let src = In_channel.read_all path in
  let tokens = Tokenizer.tokenize src path in
  printf "token tree:\n%s\n" @@ Sexp.to_string_hum @@ List.sexp_of_t Tokenizer.TokenType.sexp_of_t tokens

let script_path = Arg.(required & pos 0 (some file) None & info [] ~docv:"SCRIPT")
let main_t = Term.(const main $ script_path)
let cmd = Cmd.v (Cmd.info "levs") main_t
let () = Caml.exit (Cmd.eval cmd)
