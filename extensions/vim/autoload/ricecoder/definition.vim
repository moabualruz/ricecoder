" RiceCoder Definition Module
" Handles go-to-definition requests

" Go to definition
function! ricecoder#definition#goto_definition()
    let params = {
        \ 'language': &filetype,
        \ 'file_path': expand('%:p'),
        \ 'position': {
            \ 'line': line('.') - 1,
            \ 'character': col('.') - 1
        \ }
    \ }
    
    let response = ricecoder#rpc#call('definition/get_definition', params)
    
    if empty(response)
        call ricecoder#error#report('No definition found')
        return
    endif
    
    let file_path = response.file_path
    let line = response.range.start.line + 1
    let col = response.range.start.character + 1
    
    " Open file and jump to location
    execute 'edit ' . fnameescape(file_path)
    call cursor(line, col)
endfunction

" Go to definition in split
function! ricecoder#definition#goto_definition_split()
    let params = {
        \ 'language': &filetype,
        \ 'file_path': expand('%:p'),
        \ 'position': {
            \ 'line': line('.') - 1,
            \ 'character': col('.') - 1
        \ }
    \ }
    
    let response = ricecoder#rpc#call('definition/get_definition', params)
    
    if empty(response)
        call ricecoder#error#report('No definition found')
        return
    endif
    
    let file_path = response.file_path
    let line = response.range.start.line + 1
    let col = response.range.start.character + 1
    
    " Open file in split and jump to location
    execute 'split ' . fnameescape(file_path)
    call cursor(line, col)
endfunction

" Go to definition in vertical split
function! ricecoder#definition#goto_definition_vsplit()
    let params = {
        \ 'language': &filetype,
        \ 'file_path': expand('%:p'),
        \ 'position': {
            \ 'line': line('.') - 1,
            \ 'character': col('.') - 1
        \ }
    \ }
    
    let response = ricecoder#rpc#call('definition/get_definition', params)
    
    if empty(response)
        call ricecoder#error#report('No definition found')
        return
    endif
    
    let file_path = response.file_path
    let line = response.range.start.line + 1
    let col = response.range.start.character + 1
    
    " Open file in vertical split and jump to location
    execute 'vsplit ' . fnameescape(file_path)
    call cursor(line, col)
endfunction
