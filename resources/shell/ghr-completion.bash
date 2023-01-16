__ghr_complete() {
  local i cur prev opts cmds
  COMPREPLY=()
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD - 1]}"
  cmd=""
  opts=""

  for i in ${COMP_WORDS[@]}; do
    case "${i}" in
    "$1")
      cmd="ghr"
      ;;
    cd)
      cmd+="__cd"
      ;;
    clone)
      cmd+="__clone"
      ;;
    delete)
      cmd+="__delete"
      ;;
    init)
      cmd+="__init"
      ;;
    open)
      cmd+="__open"
      ;;
    path)
      cmd+="__path"
      ;;
    profile)
      cmd+="__profile"
      ;;
    shell)
      cmd+="__shell"
      ;;
    version)
      cmd+="__version"
      ;;
    help)
      cmd+="__help"
      ;;
    *) ;;
    esac
  done

  case "${cmd}" in
  ghr)
    opts="--help cd clone delete help init open path profile shell version"
    if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]]; then
      COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
      return 0
    fi
    case "${prev}" in
    *)
      COMPREPLY=()
      ;;
    esac
    COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
    return 0
    ;;
  ghr__cd)
    COMP_WORDBREAKS=${COMP_WORDBREAKS//:/}
    opts="$(ghr list)"

    if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]]; then
      COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
      return 0
    fi
    case "${prev}" in
    *)
      COMPREPLY=()
      ;;
    esac
    COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
    return 0
    ;;
  ghr__clone)
    opts="--help"
    COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
    return 0
    ;;
  ghr__delete)
    opts="--help"
    COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
    return 0
    ;;
  ghr__init)
    opts="--help"
    COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
    return 0
    ;;
  ghr__open)
    opts="--help"
    COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
    return 0
    ;;
  ghr__path)
    COMP_WORDBREAKS=${COMP_WORDBREAKS//:/}
    opts="$(ghr list)"

    if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]]; then
      COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
      return 0
    fi
    case "${prev}" in
    *)
      COMPREPLY=()
      ;;
    esac
    COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
    return 0
    ;;
  ghr__profile)
    opts="--help"
    COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
    return 0
    ;;
  ghr__shell)
    opts="bash fish"
    COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
    return 0
    ;;
  ghr__version)
    opts=""
    COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
    return 0
    ;;
  ghr__help)
    opts=""
    COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
    return 0
    ;;
  esac
}

# complete is a bash builtin, but recent versions of ZSH come with a function
# called bashcompinit that will create a complete in ZSH. If the user is in
# ZSH, load and run bashcompinit before calling the complete function.
if [[ -n ${ZSH_VERSION-} ]]; then
  # First calling compinit (only if not called yet!)
  # and then bashcompinit as mentioned by zsh man page.
  if ! command -v compinit >/dev/null; then
    autoload -U +X compinit && if [[ ${ZSH_DISABLE_COMPFIX-} = true ]]; then
      compinit -u
    else
      compinit
    fi
  fi
  autoload -U +X bashcompinit && bashcompinit
fi

complete -F __ghr_complete -o bashdefault -o default ghr

# END ghr completion
