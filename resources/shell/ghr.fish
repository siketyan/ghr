#!/usr/bin/env fish

function __ghr_remove
    for VAR in $argv[2..]
        if test "$VAR" = "$argv[1]"
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
        return 0
    end

    if test "$argv[1]" = "clone" || test "$argv[1]" = "init"
        if contains -- "--cd" $argv[2..]
            command ghr "$argv[1]" $argv[2..] && __ghr_cd (__ghr_remove "--cd" $argv[2..])
            return 0
        end
    end

    command ghr $argv[1..]
end

complete -c ghr -n "__fish_seen_subcommand_from cd path" -a "(ghr list)"
