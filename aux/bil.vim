" Vim syntax file
" Language: Billig DSL
" Maintainer: Neven Villani (Vanille-N)
" Lastest Revision: 21 April 2021

if exists("b:current_syntax")
    finish
endif

syn keyword bilKeyword val type span tag
syn keyword bilCategory Food Clean Home Pay Pro Tech Mov
syn keyword bilWindow Post Curr Ante Pred Succ
syn keyword bilDuration Day Week Month Year

syn match bilMoneyAmount '-\?\d\+\(.\d\d\?\)\?'
syn match bilBuiltin '@[[:alpha:]]\+'
syn match bilArgExpand '*\([[:alpha:]]\|_\)\+'
syn match bilTemplate '!\([[:alpha:]]\|_\|-\)\+'
syn match bilMarker '\([[:alpha:]]\+\|[[:digit:]]\+\):'

syn region bilString start=/"/ end=/"/
syn region bilComment start=/\/\// end=/\n/

let b:current_syntax = "bil"

hi def link bilKeyword Statement
hi def link bilMoneyAmount Constant
hi def link bilCategory Type
hi def link bilDuration Type
hi def link bilWindow StorageClass
hi def link bilBuiltin Macro
hi def link bilString String
hi def link bilArgExpand StorageClass
hi def link bilTemplate Identifier
hi def link bilComment Comment
hi def link bilMarker Todo
