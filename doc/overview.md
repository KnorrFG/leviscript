# A Shell

- immutable
- statically typed
- infers everything
- vom prinzip her reference counted, aber eig wie rust

- kompiliert beim laden. Cached den bytecode irgendwo
- einfachheit kommt vor speed. Es muss fertig werden, und nen shell script
  betreibt kein heavy lifting


## Types 

All data types are immutable

### Tuples

can be created with `(v1, v2)`, a tuple type with `(t1, t2)` a one tuple can be
created like in python

### Strings

Strings are by nature utf-8 byte sequences. 
To define a string literal, there are multiple options:

- normal string literals like python, supporting '' and "", but they are f
  strings by default, and multiline by default
- rawstrings, r'' or r"".
- dedented strings with d.
- also rd strings
- x"" strings to interprete the string a binary name and execute it

I might notice that it is neccessary to have a more complex string end token,
but currently I can't think of a scenario that requires that.

### Numbers

Int and Float, both 64bit by default. Freely castable

### Lists

- defined via []
- statically typed, so all elements must have the same type
- dynamic dispatch is available by having list elements that are type unions or
  open records
- stringify by stringifying contents and joining with \n

### Sets

- defined via s[]
- all the typical mathematical actions
- + union
- `-` for difference
- & for intersection
- !& for xor
- also, obviously iterable

### Stream / Generator

TODO
What a generator in python is. I want iterators and generators to be
compatible. But since lists are basically iterators, I need some sort of
generator. 

Generator ohne for und yield is schwierig vorzustellen

### Keywords

A keyword is an constant that only equals itself. Has a to-string function
(like everything)

TODO: not sure if they need a from-str function. Since they're values they
should be storeable in functions, in which case from string would make sense.

### Dict

aender dict defs zu `=>` sodass es sich nicht mit : als type def in die quere
kommt.
```
# 3 ways of reading dict members:
# :foo is a keyword, :foo: is a keyword + a colon
let myDict = d[ :foo: bar, "string key": some_var, 5: "five"]
echo myDict.foo
echo myDict."string key"
echo myDict[5]
```

should be iterable by something like: 

```
map |(key, val)| "$key: $val" dict
```

a dict can be "open" which means querying a key that doesn't exist returns nil
instead of raising an exception

### Nil

The typical thing. Variables can only be Nil, if their type is <name>?, which
means <name> | nil

### Type synonyms

```
type ListElem = String | Int
```

### Records

Records are basically more formal named tuples, so:
```
type MyRec = ( 
  name: type,
  ...
 )

defRec MyRec (  
  elem: type,
  ...
)
```

Records sind im endeffekt ne type familie und mit type macht man nur nen
bestimmtes type synonym. defRec generiert noch zusaetzlich ne Konstruktor
funktion.

einen Record erzeugen:

```
(fieldname = value, fieldname2 = value)
```

Records koennen auseinander erzeugt werden:

```
let a = (foo = "bar")
echo a.foo  # bar

let a = a with (foo = "bar bar")
echo a.foo  # bar bar

let b = a with (baz: "qox")
echo "${b.foo} ${b.baz}"  # bar bar qox
```

removal of fields via kw sets, but I dont know when that would be needed.
Still, for completeness:

```
# b from above
let c = b without s[:foo]
echo c.foo  # compiler error
```

### ADTs

```
enum Foo {
  Variant1,
  Variant2 field1 field2,
  Var2 field,
}
```

this will generate all the Value Ctors, in the current namespace, as well as
functions for checking which whether a value is a specific variant: isVariant1,
isVariant2 and so on. Can be wrapped in a module:

```
let foo = mod {
  enum Foo {
    ...
  }
}
```

## Comments

- `#` for line comments
- `#{}` for scoped comments, may be within a line or multiline

## Program Execution

The Syntax for function calls and program calls is identical, e.g.
```
programm_or_func arg1 arg2
```

if a program with the name of a built in function or keyword should be called,
it can be prefixed with !

```
!programm_with_name_of_built_in arg1 arg2
```

in the above calls, arg1 and arg2 are variables, to pass strings, they need to
be strings, e.g.

```
let arg1 = "Foo"
prog arg1

#or

prog "arg1"
```

As this might be anoying, there is a special execution syntax:

```
x{prog arg1 -f}
```

here prog1 will be interpreted normally, but all further arguments will be
interpreted as strings. If a line begins with a $, that's the same as putting
the whole line into x{}.

```
let flag = "-f"
$ prog arg1 $flag ${flag + 2}
```

this will execute as

```
prog "arg1" -f -f2
```

to execute a program from a string to this:

```
x{$progname}

or

x"progname" *a_list
```
### Program invocations vs Function calls

a program call is NOT a function invocation, because you never know how many
args a program takes, but a program call can be wrapped in a function.

A program call that does not return successfully, raises an exception.

functions and programs both have result as well as std(in|out|err).
The result value from a function call or an invocation can be postfixed with !
to get its stdout and !! to get its stderr.

programs must always be prefixed with ! Because otherwise a function invocation
typo ends up as program invocation. Also if a fullpath is used, without the !
it can be missinterpreted. If the path has a space, a string can be used too.
!symbol reads the var, or the result of the function call, if its written as
!(getCommand ()) !/bin/bash executes it, !"/path with/spaces/a binary" works
too

then i probably need run

### Environment

Ein program kann mit dem with keyword gestartet werden um ihm ein anderes
environment zu geben als das eigene. Das with-kw nimmt als rechten operand ein
dict, das als keys keywords hat, und als values strings

### Background processing

ist wirklich nur nen thread im hintergrund starten. vermutlich mit nem keyword,
sagen wir mal `bg` (braucht ne bessere alternative). Nimmt ne Program
invocation as arg. gibt ne pid zurueck.

bessere syntax: `&progname` or `&x{...}`

man kann pruefen ob der noch laeuft. (zum beenden reich die kill-bin) und wenn
ers nicht mehr tut an seinen rueckgabewert kommen. Wahrscheinlich mit compiler
magic functions

isRunning, getResult, getResultNil oder so


### Streams

Streams can be redirected like this:

```
foo @> NULL  # redirect stdout to /dev/null
foo !> OUT # redirect stderr to stdout
foo !@> "some_file"  # redirect both to some file
foo <@ "some_file" #  read stdin from file

foo @> NULL !> OUT  # redirect process out to /dev/null and process stderr to
the scripts out
```

ein Prozessaufruf x{} der mit @ gepostfixt wird, captured autmatisch seinen
stdout ohne ein vorgesetztes <@. Selbes gillt fuer stderr. In dem fall ist das
identisch zu `f{ let res = <@ x{...}; res@}`

### Capturing

`<@` VOR einem process captured stdout
`<!` Captured stderr
`<@!` Captured beide

Ein result kann mit den postfix operatoren @ und ! versehen werden, um die
streams auszulesen

## if expr

Grundsyntax:

```
if <predExpr> <thenExpr> [else <elseExpr>]
```

Since it's an expression and has a return value, that means, if a else is
skipped the result type is <thenExprType>?

It also means braces are necessary to distinguish between the first and the
second expression:

```
if getCond arg thenAction arg1 args2
```

will be interpreted as `getCond` having 4 args, and a missing then block, so
this must be written as:

```
if (getCond arg) thenAction arg1 arg2
```

however, this works:

```
if someBoolVar thenAction arg1 arg2
```

## Casting

- casting happens implicitely as far as possible.
- EVERYTHING is castable to string.
- everything is castable to bool: everything is true except for false and nil
- casting happens automatically between number types

- num to string needs something explicit
- compatible record type casts happen automatically
- container X to list happens automatically

## Functions

- there will be at least single dispatch
- no currying, f{} is easy enough.
- every function must store the 3 streams, input and result types, as well as
  raisable exceptions
- calling a function without arguments requires the syntax `aFunction()`, which
  is different from `aFunction ()` by the space. The first means call the
  zero-arg function aFunction, the second says call the one-arg function
  aFunction and pass unit
- calling programs does not need this distinguation
- no except via fne
- functions always take at least one argument, which can be the empty tuple


### Syntax

- static func syntax? haskell like. Including the value overloading

```
fn func arg1 arg2 *args = <expr>
```

```
fn func = {
  <expr1>
  <expr2>
}
```

- explicit lambda: like rust

- short lambda

  ```
  f{ echo $1 }
  ```

defines a one argument function. Besides $n, $@ is allowed and provides the
*args., can be sliced `*$@`. 
The `f{}` should only be neccessary, when the default rules for detecting the
function boundries would yield something else thatn what is desired. E.g. this
should work

```
map $1 + 3 container
map len $1 container
```
that means we have to have varargs

### Splicing

- lists can be spliced like in python
- there is no dictsplicing, because that would require named args, which are
  out of scope
- but you can do this: `someProg *(mapcat id dict)` which will map a dict to a
  list like this: `[key0, val0, key1, val1]` althoug that requires them to be
  of the same type. Then again, if a string list is needed, and key and val are
  stringable, it might work again
- splicing tuples can be type checked, splicing lists can throw an exception

## Pattern Matching

used in var assignments, function arguments, a match statement

- lists: `[elem, tail ...]`, `[init ..., last]`, `[first, middle ..., last]`
  sub expressions should be matchable too. What happens on match fail depends
  on context. Also `[a, b, c]` is fine, and `[]` obviously too
- sets are only matchable with literals, to see if they're contained, or as
  emty
- dicts: `d[ key: val, ... ]`, `d[ "foo": foo_val, :kw ]` the second form binds
  the value of the key `:kw` to the variable kw

- records:
    - `r(field: int, field2 = "foo")` matches a record exactly
    - `r(field: int, ...)` matches any record that has a field named field of
      type int
    - `r(field, ...): r(field: string, field2: int)` matches only the specified
      record type and only binds the field val

in these examples, all symbols can be replaced with _ to say it must be there,
but don't bind it, and symbols can be replaced with literals

a match that goes wrong produces an exception

## Piping

There are two kind of pipes:
  - | pipes the stdout of its left operand to stdin of its right operand
  - |> pipes the result part of the return value to the argument of it's right
    operand
  - |> only works with function that have one operand, so we add |*> to splice
    the tuple or the list on the left to the function on the right
  - |> and |*> abort on null. Their return type is either <lastExprType> or 
    <lastExprType>? depending on whether any function in the pipe returns nil

## Glob Expressions

`g{<expr>}` defines a glob expression, which will return a list of all matches

## try/catch/finally

```
try <expr> [finally <expr>]
```

there is no catch block, because using try will modify the expression result to
be `<exprResultType> | <PossibleExceptionTypes>`

ein raise <x> nimmt einen Wert, und die gecatchte exception is dann
Exception <x>. Aber <x> kann ein tuple sein

man kann exceptions matchen

## match

- matches types, as lightweight multiple overloaded function
```
match <expr> {
  pat1 -> <expr>
  pat2 -> <expr>
  default -> <expr>
}
```

- pattern :: ConstrPattern [: type] | glob

## Arg parsing

```
# no commands
main args d'
    Here can optionally be a string as second arg, that is the big help str.

    The first paragraph is the short help, the long help is everything.
    Btw, this script takes any amount of arguments'
  options:
    -p --print str d'
      The last argument is optional, and can be a string, which 
      will end up as help'
    -r --repetitions int d'
      The third arg is the type to which the arg should be casted'
  flags:
    -s --short d'
      obligatory help str'
    --long -l 'order doesn't matter'
= {
  # function code goes here. Flags and options are available by their long
  # names. - will be converted to _. The value of a flag is the number of it's
  # occurences.
}

# 2 main sections aren't allowed, this is an alternative:
main [src, target] d'
    this is a script that want's exactly two arguments. Normal list pattern
    matching applies'
```

alternatively, a script can have one or multiple modes:

```
all 'The whole script help string'
  options: ...
  flags: ...

mode clone args 'mode help str'
  options: ...
  flags: ...
= ...

mode commit [] 'mode help str' # does not take args
  options: ...
  flags: ...
= ...
```

Die all section definiert den Helpstring fuers ganze script, und option und
flags die in allen modes verfuegbar sind. Die all section hat keinen
funktionsbody

ich sollte schauen, dass die syntax nicht indention based wird hier.

Ein alternativer mode, bei dem der mode durch ein flag gegeben wird und no-mode
eine option ist, waere auch nice

## Keywords

- and, or for boolean
- in for __contains__ (also for globs)
- with for envs
- main, all, mode for arg parsing
- import, as, open
- try, finally
- if, else

## Modules

Das uebliche. Objekte die Values und types enthalten. Muessen nur einmal
geladen werden. Side effects beim laden sind verboten.

Es wird modulpath und sowas geben.

besides a file being a module, modules can also be defined like this:

```
let name = mod {
...
}
```

modules can be opened (unqualified)
or be imported [as] (qualified, optional renaming)

## Global Vars

- `FILE` path to current file. Ist eigentlich ehr eine art Macro
- `LINE` current line. Ist eigentlich ehr eine art Macro
- `ARGV` arg values
  0 is the executable path, 1 is the first argument
- `ENV` the current environment.
- `IN`, `OUT`, `ERR` scripts streams

## Buildin Functions and operators

### Operators

- ~= for glob matching
- $ at line begin: special form of x{}
- $ somewhere else: same as haskell (nope, not needed without currying, doesn't
  even work)
- `*` for splicing
- << and >> for function chaining
- arithmetic stuff

- <@ und <! muessen eine noch geringere prazedenz haben als |, damit man das
  ohne klammern vor ne pipe schreiben kann

### Path

Paths are string, but we need a lot of manipulation functions. 

- exists
- isDir
- isFile
- name
- stem
- extension
- parent
- isRelative
- /
- ...

### Other

- eval
- (from|to)_json
- some sort of handy text parsing to parse ll output or smth like that
- shl, shr, binAnd, binOr, xor, binNot
- mod / c_mod
- ne art scanf function

## Typesystem

- das ganze ist statically typed, aber es sollte vollstaendige type inference
  geben.
- primitive types: Int, Float, Bool, String, 
- container: [<Type>], s[<Type>], d[key, val]. 
- functions: fn(t1, t2, *t3) -> res
- records.
- tuple
- ADT
- Union type

## Memory model

- references and a heap are only needed when mutation is at play.
- I think an evergrowing "stack" would work. A stack has frames, for the
  scopes, and everything only ever references downward (under the hood)
- if a var is shadowed, the orig var is still on the stack, and the new is
  above it.
- This might not be memory optimal, but it's easy, and probably works

- on the stack there will be a lot of shifting. We don't want to shift big data
  types all the time. There will be a heap (which will preallocate some memory,
  so not every heap allocation is slow). So when stuff is shifted around on the
  stack, less memory gets moved. 
- Der compiler sollte in der Lage sein rauszufinden was wann, bzw ob ueberhaupt
  etwas gemoved wird. Wenn etwas nur einmal erzeugt und dann zerstoert wird,
  landet es auch auf dem Stack, und wenn die ref tot ist (sie hat eige am ende
  ihre scopes nur die wahl zwischen sterben und als arg gemoved werden), wird
  der heap eintrag gefreed. Der Heap is dann vermutlich ne HashMap. Das ist
  weniger stress. Oder ehr nen Set. Mal schauen was ocaml hergibt

## Generics

- I need generics. And then I want HKTs by means of "Incomplete types".
- I also want generics on variables. symbol + type is unique, so I should be
  able to overload values. Also something like this should be possible:

---
fn getDefault[T] = someVal: T
---

The colon after val casts someVal to T, which might be neccessary, if someVal
is not of type T.

ill also want variadic generics, to be able to describe function chaining

## Notes to self

- modules can't be values, because then they need a type, and that's a problem
- i don't need finally, because execution continues after a try anyway
- module names cannot contain minuses, because those will be passed as
  operators. And module names must be symbols therefore. Everything contained
  in a Scope must be namable by a symbol
- there should be dead code elimination in the linker step, or maybe already in
  the compile step.
- stdlib should be precompiled

## Stuff to still think about

- maybe |> shouldn't abort on (), and postfixing a function with a `?` could be
  syntax sugar for creating a wrapper that just returns () if the input is (),
  and executes the function otherwise
- der anfang muss nochmal ueberarbeitet werden, da es jetzt statements und so
  gibt
- i need a std-lib
- lists. E.g. of all tokens
- rename String to Str
- zero arg funcs not yet documented
- byte type
- ranges
- correct all '...' operators to '..'
- io functions
- blocks aren't documented yet
- das erzeugen eines sets mit einem element laesst sich syntaktisch nicht
  unterscheiden von dem zugriff auf einen container
    - entweder anderen access operator, oder andere konstruktor syntax
- assert
- I really want generators
- also maybe return for early return?
- rename s[] to set[] d to dict[] and insert vec[]. And then have those as
  reserved keywords, so you cant have a symbol of that name, which would be
  ambigous syntax
- longterm: full pl. Missing feats:
  - generics/macros
  - async,
  - threads
  - libc
  - cffi

## Impl notes

- immer bevor man ein token parsed, sollte man white space skippen. In einem
  token kann keiner vorkommen, d.h. der Beginn von parseToken() is die
  richtige Stelle. Wenn man in einer multiline expression ist, und deshalb ne
  newline als whitespace weg frisst, muss man die aktuelle zeilen nummer
  erhoehen. Allerdings weiß ein tokenizer nicht ob man gerade in einer
  expression is, also braucht es ein newline token und ein escapedNewline
  token
- comments koennen vom tokenizer aussortiert werden. Theoretisch in der
  selben funktion in der er whitespace aussortiert. 
- Format sind erst mal ein token, und werden spaeter geparsed. Oder man
  parsed sie als Fstringbegin, VanillaString, Identifier, VanillaString, 
  SubexprBegin, NormalToken, ..., subexprEnd, VanillaString, linebreak,
  vanillaString, FStringEnd.
  - Do some research on this
- fn for def and nil for null. Shortness is king
- wenn man es dem user erlaub bytecode zu erzeugen, und den als interpreter
  input zu nehmen, macht es das leichter programme zu verteilen, so wegen
  package manager und so. Auch wenn das nicht der primaerzweck ist.
- Can I have a shebang in a bytecode file?
- haskell like type annotations should be allowed
- casting rules
