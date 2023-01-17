#!/usr/bin/env bash

__ghr_complete__static() {
  local options

  options=("${@:2}")

  compgen -W "${options[*]}" -- "$1"
}

__ghr_complete__repos() {
  local repositories suggestions

  repositories="$(ghr list)"
  suggestions="$(compgen -W "${repositories}" -- "$1")"

  if [[ $1 != -* && ${COMP_CWORD} -ne 2 ]]; then
    return
  fi

  echo "$suggestions"
}

__ghr_complete_cd() {
  __ghr_complete__repos "$1"
}

__ghr_complete() {
  local cword

  # Replaces ':' in $COMP_WORDBREAKS to prevent bash appends the suggestion after ':' repeatedly
  COMP_WORDBREAKS=${COMP_WORDBREAKS//:/}

  cword="${COMP_WORDS[COMP_CWORD]}"

  if [ "${COMP_CWORD}" = 1 ]; then
    COMPREPLY=($(__ghr_complete__static "${cword}" --help cd clone delete help init open path profile shell version))
    return 0
  fi

  case "${COMP_WORDS[1]}" in
  cd)
    COMPREPLY=($(__ghr_complete__repos "${cword}"))
    ;;
  clone)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  delete)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  init)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  open)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  path)
    COMPREPLY=($(__ghr_complete__repos "${cword}"))
    ;;
  profile)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  shell)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  version)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  help)
    COMPREPLY=()
    ;;
  *)
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
