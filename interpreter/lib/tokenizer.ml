type token_type = 
  (* misc *)
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
  | StrEnd
  | RStrStart
  | RStrEnd
  | DStrStart
  | DStrEnd
  | DRStrStart
  | DRStrEnd
  | StrSubExprStart
  | StrSubExprEnd
  | ListStart
  | ListEnd
  | SetStart
  | SetEnd
  | DictStart
  | DictEnd
  | RecStart
  | RecEnd
  | FExStart
  | FExEnd
  | XExStart
  | XExEnd
  | AmpExStart
  | AmpExEnd
  | GExStart
  | GExEnd

