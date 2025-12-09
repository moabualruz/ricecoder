" RiceCoder Hover Module
" Handles hover information requests and displays them

let s:hover_popup = -1

" Get hover information at cursor
function! ricecoder#hover#get_hover()
    let params = {
        \ 'language': &filetype,
        \ 'file_path': expand('%:p'),
        \ 'position': {
            \ 'line': line('.') - 1,
            \ 'character': col('.') - 1
        \ }
    \ }
    
    let response = ricecoder#rpc#call('hover/get_hover', params)
    
    if empty(response)
        return {}
    endif
    
    return response
endfunction

" Show hover information
function! ricecoder#hover#show()
    let hover = ricecoder#hover#get_hover()
    
    if empty(hover)
        call ricecoder#hover#hide()
        return
    endif
    
    let contents = hover.contents
    
    " Display hover information
    if has('nvim')
        call ricecoder#hover#show_nvim(contents)
    else
        call ricecoder#hover#show_vim(contents)
    endif
endfunction

" Show hover in neovim using floating window
function! ricecoder#hover#show_nvim(contents)
    " Create floating window
    let width = min([80, &columns - 4])
    let height = min([20, &lines - 4])
    let row = 1
    let col = 1
    
    let opts = {
        \ 'relative': 'cursor',
        \ 'width': width,
        \ 'height': height,
        \ 'row': row,
        \ 'col': col,
        \ 'style': 'minimal',
        \ 'border': 'rounded'
    \ }
    
    " Close previous popup if exists
    if s:hover_popup != -1
        call nvim_win_close(s:hover_popup, v:true)
    endif
    
    " Create buffer
    let buf = nvim_create_buf(v:false, v:true)
    call nvim_buf_set_lines(buf, 0, -1, v:false, split(a:contents, '\n'))
    
    " Create window
    let s:hover_popup = nvim_open_win(buf, v:false, opts)
    
    " Set window options
    call nvim_win_set_option(s:hover_popup, 'wrap', v:true)
    call nvim_win_set_option(s:hover_popup, 'number', v:false)
endfunction

" Show hover in vim using preview window
function! ricecoder#hover#show_vim(contents)
    " Use preview window for hover
    pclose
    
    " Create preview window
    execute 'pedit RicecoderHover'
    
    " Get preview window number
    let preview_win = -1
    for win in range(1, winnr('$'))
        if getwinvar(win, '&previewwindow')
            let preview_win = win
            break
        endif
    endfor
    
    if preview_win != -1
        " Set content in preview window
        call setwinvar(preview_win, '&number', 0)
        call setwinvar(preview_win, '&wrap', 1)
        
        " Set buffer content
        let lines = split(a:contents, '\n')
        call setbufline(winbufnr(preview_win), 1, lines)
    endif
endfunction

" Hide hover information
function! ricecoder#hover#hide()
    if has('nvim')
        if s:hover_popup != -1
            call nvim_win_close(s:hover_popup, v:true)
            let s:hover_popup = -1
        endif
    else
        pclose
    endif
endfunction

" Show hover on demand
function! ricecoder#hover#show_on_demand()
    call ricecoder#hover#show()
endfunction
