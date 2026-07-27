" Sample Vim script with functions, commands, and autocommands
source ~/.vim/utils.vim
runtime plugin/defaults.vim

let g:my_plugin_enabled = 1
let g:my_plugin_width = 80
let s:cache = {}

" Toggle a boolean global option
function! ToggleOption(name)
    if get(g:, a:name, 0)
        let g:[a:name] = 0
        echo a:name . ' disabled'
    else
        let g:[a:name] = 1
        echo a:name . ' enabled'
    endif
endfunction

" Format the current buffer
function! FormatBuffer()
    let l:pos = getpos('.')
    silent %!prettier --stdin-filepath %
    call setpos('.', l:pos)
endfunction

" Open a floating terminal window
function! OpenTerminal(cmd)
    let l:buf = nvim_create_buf(v:false, v:true)
    let l:opts = {
        \ 'relative': 'editor',
        \ 'width': g:my_plugin_width,
        \ 'height': 20,
        \ 'col': 10,
        \ 'row': 5,
        \ 'style': 'minimal',
        \ }
    call nvim_open_win(l:buf, v:true, l:opts)
    call termopen(a:cmd)
endfunction

" Lookup a symbol in the cache
function! s:LookupCache(key)
    return get(s:cache, a:key, '')
endfunction

" Classify a count relative to a threshold, exercising elseif
function! s:ClassifyCount(n, threshold)
    if a:n < 0
        return 'negative'
    elseif a:n < a:threshold
        return 'below'
    else
        return 'at-or-above'
    endif
endfunction

" Walk a list, exercising for/while/try/throw/continue/break
function! s:ProcessList(items)
    let l:total = 0
    for l:item in a:items
        if l:item < 0
            continue
        endif
        if l:item > 1000
            break
        endif
        let l:total += l:item
    endfor

    let l:i = 0
    while l:i < len(a:items)
        let l:i += 1
    endwhile

    try
        if l:total < 0
            throw 'negative total'
        endif
        call s:LookupCache('total')
    catch /negative/
        echo 'caught negative total'
    finally
        echo 'processing complete'
    endtry

    return l:total
endfunction

augroup MyPlugin
    autocmd!
    autocmd BufWritePre *.rs call FormatBuffer()
    autocmd BufReadPost *.vim syntax match Comment /#.*/
augroup END

augroup FileTypeSettings
    autocmd!
    autocmd FileType python setlocal expandtab shiftwidth=4 tabstop=4
    autocmd FileType go setlocal noexpandtab tabstop=4
augroup END

command! -nargs=0 ToggleWrap call ToggleOption('wrap')
command! -nargs=? Term call OpenTerminal(<q-args>)

if g:my_plugin_enabled
    nnoremap <leader>f :call FormatBuffer()<CR>
    nnoremap <leader>t :Term<CR>
endif
