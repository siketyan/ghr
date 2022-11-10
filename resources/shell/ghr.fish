#!/usr/bin/env fish

function __ghr_remove
    for VAR in $argv[2..]
        if test "$VAR" = "$1"
            continue
        end

        echo "$VAR"
    end
end

function __ghr_cd
    cd "$(ghr path $argv)"
end

function ghr
    if test "$argv[1]" = "cd"
        __ghr_cd $argv[2..]
    else if test "$argv[1]" = "clone" && contains -- "--cd" $argv[2..]
        command ghr clone $argv[2..] && __ghr_cd (__ghr_remove "--cd" $argv[2..])
    else
        command ghr $argv[1..]
    end
end