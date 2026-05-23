_ldc() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="ldc"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        ldc)
            opts="-o -f -h -V --output --format --font --font-mono --font-path --list-fonts --title --author --subject --margin --page-size --page-width --page-height --header-left --header-center --header-right --footer-left --footer-center --footer-right --no-header-rule --no-footer-rule --drop-caps --bibliography --lir --pdfa-level --color --help --version [INPUTS]..."
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --format)
                    COMPREPLY=($(compgen -W "pdf gir sir html epub txt docx sir2 ldir" -- "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -W "pdf gir sir html epub txt docx sir2 ldir" -- "${cur}"))
                    return 0
                    ;;
                --font)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --font-mono)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --font-path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --title)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --author)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --subject)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --margin)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page-size)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page-width)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page-height)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --header-left)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --header-center)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --header-right)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --footer-left)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --footer-center)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --footer-right)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --bibliography)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pdfa-level)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _ldc -o nosort -o bashdefault -o default ldc
else
    complete -F _ldc -o bashdefault -o default ldc
fi
