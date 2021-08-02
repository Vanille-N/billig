" Vim syntax file
" Language: Billig DSL
" Maintainer: Neven Villani (Vanille-N)
" Lastest Revision: 21 April 2021
" License: MIT or Apache 2.0

if exists("b:current_syntax")
    finish
endif

syn match pestRange +'.'\.\.'.'+
syn match pestOperator '(\|)\|+\|*\|?\|!\||\|\~'
syn match pestRule '\([[:alpha:]]\|_\)\+'
syn match pestRepeat '{\([[:digit:]]\|,\)\+}'
syn match pestEscape contained '\\.'
syn match pestWrapper '\(_\|\$\|@\|\){\|}'
syn region pestPattern start=/"/ skip=+\\\\\|\\"+ end=/"/ contains=pestEscape
syn keyword pestPredefs ANY COMMENT WHITESPACE SOI EOI
syn region pestComment start='//' end='\n'

let b:current_syntax = "pest"

hi def link pestEscape Special
hi def link pestPattern String
hi def link pestRange String
hi def link pestOperator Type
hi def link pestRule Operator
hi def link pestPredefs Todo
hi def link pestRepeat Type
hi def link pestComment Comment
hi def link pestWrapper Statement

