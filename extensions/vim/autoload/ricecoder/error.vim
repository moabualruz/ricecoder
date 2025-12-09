" RiceCoder Error Handling Module
" Handles error reporting and logging

" Report an error to the user
function! ricecoder#error#report(message)
    echohl ErrorMsg
    echo 'RiceCoder: ' . a:message
    echohl None
endfunction

" Report a warning to the user
function! ricecoder#error#warn(message)
    echohl WarningMsg
    echo 'RiceCoder: ' . a:message
    echohl None
endfunction

" Report info to the user
function! ricecoder#error#info(message)
    echohl None
    echo 'RiceCoder: ' . a:message
endfunction

" Log a message (for debugging)
function! ricecoder#error#log(message)
    if get(g:, 'ricecoder_debug', 0)
        echom 'RiceCoder DEBUG: ' . a:message
    endif
endfunction
