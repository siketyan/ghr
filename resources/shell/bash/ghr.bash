#!/usr/bin/env bash

__GHR=$(which ghr | head -n 1)

__ghr_cd() {
    cd "$($__GHR path $@)"
}

__ghr_contains() {
    for VAR in ${@:2}; do
        if [ "$VAR" = "$1" ]; then
            return 0
        fi
    done

    return 1
}

__ghr_remove() {
    for VAR in ${@:2}; do
        if [ "$VAR" = "$1" ]; then
            continue
        fi

        echo "$VAR"
    done
}

ghr() {
    if [ "$#" -gt 1 ]; then
        if [ "$1" = "cd" ]; then
            __ghr_cd ${@:2}
            return
        fi

        if { [ "$1" = "clone" ] || [ "$1" = "init" ]; } && __ghr_contains "--cd" ${@:2}; then
            $__GHR "$1" ${@:2}
            __ghr_cd ${@:2}
            return
        fi
    fi

    $__GHR $@
}

source ./ghr-completion.bash
