open Core

let str_tail str = 
  String.sub str ~pos:1 ~len:((String.length str) - 1)