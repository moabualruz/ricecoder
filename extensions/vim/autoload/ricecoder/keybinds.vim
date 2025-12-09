" RiceCoder Keybinds Module
" Sets up default keybindings for ricecoder commands

" Setup default keybindings
function! ricecoder#keybinds#setup()
    " Completion - use Ctrl+X Ctrl+O for omnifunc
    " This is vim's default for omnifunc completion
    
    " Go to definition - Ctrl+]
    nnoremap <buffer> <C-]> :call ricecoder#goto_definition()<CR>
    
    " Go to definition in split - Ctrl+W ]
    nnoremap <buffer> <C-w>] :call ricecoder#definition#goto_definition_split()<CR>
    
    " Go to definition in vsplit - Ctrl+W Ctrl+]
    nnoremap <buffer> <C-w><C-]> :call ricecoder#definition#goto_definition_vsplit()<CR>
    
    " Show hover - K (standard vim key for help)
    nnoremap <buffer> K :call ricecoder#hover#show_on_demand()<CR>
    
    " Show diagnostics - Ctrl+E
    nnoremap <buffer> <C-e> :call ricecoder#diagnostics#show_message()<CR>
    
    " Enable omnifunc completion
    call ricecoder#completion#enable()
endfunction

" Allow user to customize keybindings
function! ricecoder#keybinds#map(key, command)
    execute 'nnoremap <buffer> ' . a:key . ' :' . a:command . '<CR>'
endfunction
