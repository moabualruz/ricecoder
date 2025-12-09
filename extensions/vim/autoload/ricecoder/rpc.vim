" RiceCoder JSON-RPC Client
" Handles communication with ricecoder backend via JSON-RPC

let s:rpc_id = 1
let s:rpc_host = 'localhost'
let s:rpc_port = 9000
let s:rpc_timeout = 5000
let s:pending_requests = {}

" Initialize RPC connection
function! ricecoder#rpc#init(host, port)
    let s:rpc_host = a:host
    let s:rpc_port = a:port
endfunction

" Send a JSON-RPC request to ricecoder
function! ricecoder#rpc#call(method, params)
    let request = {
        \ 'jsonrpc': '2.0',
        \ 'id': s:rpc_id,
        \ 'method': a:method,
        \ 'params': a:params
        \ }
    
    let s:rpc_id += 1
    
    " For neovim, use built-in RPC
    if has('nvim')
        return ricecoder#rpc#call_nvim(request)
    else
        " For vim, use job API
        return ricecoder#rpc#call_vim(request)
    endif
endfunction

" Neovim RPC call
function! ricecoder#rpc#call_nvim(request)
    try
        let response = json_decode(system(printf(
            \ 'curl -s -X POST http://%s:%d/rpc -H "Content-Type: application/json" -d %s',
            \ s:rpc_host,
            \ s:rpc_port,
            \ shellescape(json_encode(a:request))
        \ )))
        
        if has_key(response, 'result')
            return response.result
        elseif has_key(response, 'error')
            call ricecoder#error#report('RPC Error: ' . response.error.message)
            return {}
        endif
    catch
        call ricecoder#error#report('RPC Connection Error: ' . v:exception)
        return {}
    endtry
endfunction

" Vim RPC call using job API
function! ricecoder#rpc#call_vim(request)
    try
        let response = json_decode(system(printf(
            \ 'curl -s -X POST http://%s:%d/rpc -H "Content-Type: application/json" -d %s',
            \ s:rpc_host,
            \ s:rpc_port,
            \ shellescape(json_encode(a:request))
        \ )))
        
        if has_key(response, 'result')
            return response.result
        elseif has_key(response, 'error')
            call ricecoder#error#report('RPC Error: ' . response.error.message)
            return {}
        endif
    catch
        call ricecoder#error#report('RPC Connection Error: ' . v:exception)
        return {}
    endtry
endfunction

" Async RPC call (for long-running operations)
function! ricecoder#rpc#call_async(method, params, callback)
    let request = {
        \ 'jsonrpc': '2.0',
        \ 'id': s:rpc_id,
        \ 'method': a:method,
        \ 'params': a:params
        \ }
    
    let request_id = s:rpc_id
    let s:rpc_id += 1
    
    " Store callback for later
    let s:pending_requests[request_id] = a:callback
    
    " Send request asynchronously
    if has('nvim')
        call ricecoder#rpc#call_async_nvim(request, request_id)
    else
        call ricecoder#rpc#call_async_vim(request, request_id)
    endif
endfunction

" Async RPC call for neovim
function! ricecoder#rpc#call_async_nvim(request, request_id)
    " Use jobstart for async execution
    let cmd = printf(
        \ 'curl -s -X POST http://%s:%d/rpc -H "Content-Type: application/json" -d %s',
        \ s:rpc_host,
        \ s:rpc_port,
        \ shellescape(json_encode(a:request))
    \ )
    
    call jobstart(cmd, {
        \ 'on_stdout': function('ricecoder#rpc#handle_response', [a:request_id]),
        \ 'on_stderr': function('ricecoder#rpc#handle_error', [a:request_id])
        \ })
endfunction

" Async RPC call for vim
function! ricecoder#rpc#call_async_vim(request, request_id)
    let cmd = printf(
        \ 'curl -s -X POST http://%s:%d/rpc -H "Content-Type: application/json" -d %s',
        \ s:rpc_host,
        \ s:rpc_port,
        \ shellescape(json_encode(a:request))
    \ )
    
    call job_start(cmd, {
        \ 'out_cb': function('ricecoder#rpc#handle_response_vim', [a:request_id]),
        \ 'err_cb': function('ricecoder#rpc#handle_error_vim', [a:request_id])
        \ })
endfunction

" Handle async response (neovim)
function! ricecoder#rpc#handle_response(request_id, job_id, data, event)
    if empty(a:data)
        return
    endif
    
    try
        let response = json_decode(join(a:data, ''))
        if has_key(s:pending_requests, a:request_id)
            let callback = s:pending_requests[a:request_id]
            call callback(response)
            unlet s:pending_requests[a:request_id]
        endif
    catch
        call ricecoder#error#report('Failed to parse RPC response: ' . v:exception)
    endtry
endfunction

" Handle async error (neovim)
function! ricecoder#rpc#handle_error(request_id, job_id, data, event)
    if !empty(a:data)
        call ricecoder#error#report('RPC Error: ' . join(a:data, ' '))
    endif
endfunction

" Handle async response (vim)
function! ricecoder#rpc#handle_response_vim(request_id, channel, msg)
    try
        let response = json_decode(a:msg)
        if has_key(s:pending_requests, a:request_id)
            let callback = s:pending_requests[a:request_id]
            call callback(response)
            unlet s:pending_requests[a:request_id]
        endif
    catch
        call ricecoder#error#report('Failed to parse RPC response: ' . v:exception)
    endtry
endfunction

" Handle async error (vim)
function! ricecoder#rpc#handle_error_vim(request_id, channel, msg)
    call ricecoder#error#report('RPC Error: ' . a:msg)
endfunction
