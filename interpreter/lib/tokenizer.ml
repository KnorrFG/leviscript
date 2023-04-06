open! Core
open Sedlexing

module TokenType = struct
type t = 
  (* misc *)
  | Eof
  | Symbol of string
  | Typename of string
  | Newline
  | EscapedNewline
  | Operator of string
  | DollarSymbol of string
  | DollarNum of int32
  | ParOperator of string
  | ParPrefixOperator of string
  | ParPostfixOperator of string
  | Magic of string
  (* literals *)
  | IntLit of int64
  | FloatLit of float
  | BoolLit of bool
  | KeywordLit of string
  | UnitLit
  | StrLitPart of string
  | EscapedChar of string
  (* special symbols *)
  | Comma
  | Colon
  | Dot
  | Eq
  | DoubleDot
  | SAt
  | Bar
  | Semicolon
  | Arrow
  | FPipe
  | StarPipe
  | Questionmark
  | DoubleColon
  | StarPrefix
  | LMDollar (*LM is short for Line magic*)
  | DollarStar
  | BangPrefix
  | BangPostfix
  | AtPostfix
  | AtBangPostfix
  | CaptureErr
  | CaptureOut
  | CaptureBoth
  | PipeOut
  | PipeErr
  | PipeBoth
  | AmpPrefix
  (* Keywords *)
  | KWLet
  | KWType
  | KWRecord
  | KWMod
  | KWRecordMod
  | KWWith
  | KWWithout
  | KWNewtype
  | KWFn
  | KWFne
  | KWFna
  | KWRestrict
  | KWTo
  | KWIf
  | KWThen
  | KWElse
  | KWTry
  | KWAnd
  | KWOr
  | KWNot
  | KWMatch
  | KWRaise
  | KWInfix
  | KWPrefix
  | KWPostfix
  | KWOut
  | KWErr
  | KWNull
  | KWLine
  | KWFile
  | KWImport
  | KWIn
  | KWAs
  | KWOpen
  | KWException
  (* Scope Tokens *)
  | LBracket
  | RBracket
  | LParen
  | RParen
  | LBrace
  | RBrace
  | StrStart
  | RStrStart
  | DStrStart
  | DRStrStart
  | StrSubExprStart
  | ListStart
  | SetStart
  | DictStart
  | RecStart
  | FExStart
  | XExStart
  | AmpExStart
  | GExStart
[@@deriving sexp, eq]

let (=) = equal
end



let digit = [%sedlex.regexp? '0' .. '9']

let number = [%sedlex.regexp? Plus digit]

let blank = [%sedlex.regexp? ' ' | '\t']

let newline = [%sedlex.regexp? '\r' | '\n' | "\r\n"]

let any_blank = [%sedlex.regexp? blank | newline]

let letter = [%sedlex.regexp? 'a' .. 'z' | 'A' .. 'Z']

let decimal_ascii = [%sedlex.regexp? Plus ('0' .. '9')]

let octal_ascii = [%sedlex.regexp? "0o", Plus ('0' .. '7')]

let hex_ascii = [%sedlex.regexp? "0x", Plus (('0' .. '9' | 'a' .. 'f' | 'A' .. 'F'))]

let symbol = [%sedlex.regexp? (ll | lo), Star (ll | lu | lo | nd | pc | '\'')  ]

let rec nom buf =
  match%sedlex buf with
  | Plus any_blank -> nom buf
  | _ -> ()

let token buf =
  nom buf;
  let open TokenType in
  match%sedlex buf with
  | eof -> Eof
  | "x{" -> XExStart
  | "}" -> RBrace
  | symbol -> Symbol (Utf8.lexeme buf)
  | _ -> assert false

let tokenize buf = Sedlexing.with_tokenizer token buf

