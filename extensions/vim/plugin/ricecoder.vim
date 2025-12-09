" RiceCoder Vim/Neovim Plugin
" Provides IDE integration with ricecoder for vim and neovim
" Supports: completion, diagnostics, hover information

if exists('g:ricecoder_loaded')
    finish
endif
let g:ricecoder_loaded = 1

" Configuration defaults
let g:ricecoder_host = get(g:, 'ricecoder_host', 'localhost')
let g:ricecoder_port = get(g:, 'ricecoder_port', 9000)
let g:ricecoder_timeout = get(g:, 'ricecoder_timeout', 5000)
let g:ricecoder_enabled = get(g:, 'ricecoder_enabled', 1)

" Initialize RiceCoder connection
function! ricecoder#init()
    if !g:ricecoder_enabled
        return
    endif
    
    " Initialize JSON-RPC client
    call ricecoder#rpc#init(g:ricecoder_host, g:ricecoder_port)
    
    " Set up autocommands for completion and diagnostics
    augroup ricecoder
        autocmd!
        autocmd CompleteDone * call ricecoder#completion#on_complete_done()
        autocmd TextChanged,TextChangedI * call ricecoder#diagnostics#update()
        autocmd CursorMoved,CursorMovedI * call ricecoder#hover#show()
    augroup END
    
    " Set up key mappings
    call ricecoder#keybinds#setup()
endfunction

" Completion function for omnifunc
function! ricecoder#complete(findstart, base)
    if a:findstart
        " Find the start of the word
        let line = getline('.')
        let start = col('.') - 1
        while start > 0 && line[start - 1] =~ '\w'
            let start -= 1
        endwhile
        return start
    else
        " Get completions from ricecoder
        return ricecoder#completion#get_completions(a:base)
    endif
endfunction

" Get diagnostics for current buffer
function! ricecoder#get_diagnostics()
    return ricecoder#diagnostics#get_diagnostics()
endfunction

" Get hover information at cursor
function! ricecoder#get_hover()
    return ricecoder#hover#get_hover()
endfunction

" Go to definition
function! ricecoder#goto_definition()
    call ricecoder#definition#goto_definition()
endfunction

" Initialize on plugin load
call ricecoder#init()
