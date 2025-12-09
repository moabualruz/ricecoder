" RiceCoder Completion Module
" Handles code completion requests and responses

" Get completions from ricecoder
function! ricecoder#completion#get_completions(base)
    let params = {
        \ 'language': &filetype,
        \ 'file_path': expand('%:p'),
        \ 'position': {
            \ 'line': line('.') - 1,
            \ 'character': col('.') - 1
        \ },
        \ 'context': getline('.')
    \ }
    
    let response = ricecoder#rpc#call('completion/get_completions', params)
    
    if empty(response)
        return []
    endif
    
    " Convert ricecoder completion items to vim completion format
    let completions = []
    for item in response
        call add(completions, {
            \ 'word': item.insert_text,
            \ 'abbr': item.label,
            \ 'kind': ricecoder#completion#get_completion_kind(item.kind),
            \ 'menu': get(item, 'detail', ''),
            \ 'info': get(item, 'documentation', '')
        \ })
    endfor
    
    return completions
endfunction

" Map ricecoder completion kind to vim kind
function! ricecoder#completion#get_completion_kind(kind)
    let kind_map = {
        \ 'Text': 't',
        \ 'Method': 'm',
        \ 'Function': 'f',
        \ 'Constructor': 'f',
        \ 'Field': 'm',
        \ 'Variable': 'v',
        \ 'Class': 'c',
        \ 'Interface': 'i',
        \ 'Module': 'M',
        \ 'Property': 'm',
        \ 'Unit': 'u',
        \ 'Value': 'v',
        \ 'Enum': 'e',
        \ 'Keyword': 'k',
        \ 'Snippet': 's',
        \ 'Color': 'c',
        \ 'File': 'f',
        \ 'Reference': 'r',
        \ 'Folder': 'f',
        \ 'EnumMember': 'e',
        \ 'Constant': 'd',
        \ 'Struct': 's',
        \ 'Event': 'e',
        \ 'Operator': 'o',
        \ 'TypeParameter': 't'
    \ }
    
    return get(kind_map, a:kind, 't')
endfunction

" Handle completion done event
function! ricecoder#completion#on_complete_done()
    " Optional: perform any cleanup after completion
endfunction

" Enable omnifunc completion
function! ricecoder#completion#enable()
    setlocal omnifunc=ricecoder#complete
endfunction
