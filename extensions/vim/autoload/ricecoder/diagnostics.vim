" RiceCoder Diagnostics Module
" Handles diagnostic requests and displays errors/warnings

let s:diagnostic_signs = {}
let s:diagnostic_highlights = {}

" Get diagnostics for current buffer
function! ricecoder#diagnostics#get_diagnostics()
    let params = {
        \ 'language': &filetype,
        \ 'file_path': expand('%:p'),
        \ 'source': join(getline(1, '$'), '\n')
    \ }
    
    let response = ricecoder#rpc#call('diagnostics/get_diagnostics', params)
    
    if empty(response)
        return []
    endif
    
    return response
endfunction

" Update diagnostics for current buffer
function! ricecoder#diagnostics#update()
    let diagnostics = ricecoder#diagnostics#get_diagnostics()
    
    if empty(diagnostics)
        call ricecoder#diagnostics#clear()
        return
    endif
    
    " Clear previous diagnostics
    call ricecoder#diagnostics#clear()
    
    " Display new diagnostics
    for diagnostic in diagnostics
        call ricecoder#diagnostics#display(diagnostic)
    endfor
endfunction

" Display a single diagnostic
function! ricecoder#diagnostics#display(diagnostic)
    let line = a:diagnostic.range.start.line + 1
    let col = a:diagnostic.range.start.character + 1
    let severity = a:diagnostic.severity
    let message = a:diagnostic.message
    
    " Determine sign and highlight based on severity
    let sign_name = severity == 1 ? 'RicecoderError' : 'RicecoderWarning'
    let hl_name = severity == 1 ? 'RicecoderErrorHL' : 'RicecoderWarningHL'
    
    " Place sign
    if has('signs')
        execute printf('sign place %d line=%d name=%s file=%s',
            \ line,
            \ line,
            \ sign_name,
            \ expand('%:p'))
    endif
    
    " Store diagnostic for later reference
    if !has_key(s:diagnostic_signs, line)
        let s:diagnostic_signs[line] = []
    endif
    call add(s:diagnostic_signs[line], {
        \ 'message': message,
        \ 'severity': severity,
        \ 'col': col
    \ })
endfunction

" Clear all diagnostics
function! ricecoder#diagnostics#clear()
    " Clear signs
    if has('signs')
        execute 'sign unplace * file=' . expand('%:p')
    endif
    
    let s:diagnostic_signs = {}
endfunction

" Show diagnostic message at cursor
function! ricecoder#diagnostics#show_message()
    let line = line('.')
    
    if has_key(s:diagnostic_signs, line)
        let messages = []
        for diagnostic in s:diagnostic_signs[line]
            call add(messages, diagnostic.message)
        endfor
        
        if !empty(messages)
            echo join(messages, '\n')
        endif
    endif
endfunction

" Define signs and highlights
function! ricecoder#diagnostics#setup_signs()
    if has('signs')
        " Define error sign
        execute 'sign define RicecoderError text=>> texthl=RicecoderErrorSign'
        execute 'sign define RicecoderWarning text=>> texthl=RicecoderWarningSign'
    endif
    
    " Define highlights
    highlight RicecoderErrorSign ctermfg=1 guifg=#ff0000
    highlight RicecoderWarningSign ctermfg=3 guifg=#ffff00
    highlight RicecoderErrorHL ctermbg=1 guibg=#ff0000
    highlight RicecoderWarningHL ctermbg=3 guibg=#ffff00
endfunction

" Initialize diagnostics
call ricecoder#diagnostics#setup_signs()
