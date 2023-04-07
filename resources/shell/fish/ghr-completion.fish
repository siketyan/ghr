function __fish_is_arg_n --argument-names n
    test $n -eq (count (string match -v -- '-*' (commandline -poc)))
end

complete -c ghr -f

complete -c ghr -n "__fish_is_arg_n 1" -a cd -d "Change directory to one of the managed repositories (Shell extension required)"
complete -c ghr -n "__fish_is_arg_n 1" -a clone -d "Clones a Git repository to local"
complete -c ghr -n "__fish_is_arg_n 1" -a delete -d "Deletes a repository from local"
complete -c ghr -n "__fish_is_arg_n 1" -a init -d "Initialises a Git repository in local"
complete -c ghr -n "__fish_is_arg_n 1" -a list -d "Lists all managed repositories"
complete -c ghr -n "__fish_is_arg_n 1" -a open -d "Opens a repository in an application"
complete -c ghr -n "__fish_is_arg_n 1" -a path -d "Prints the path to root, owner, or a repository"
complete -c ghr -n "__fish_is_arg_n 1" -a profile -d "Manages profiles to use in repositories"
complete -c ghr -n "__fish_is_arg_n 1" -a shell -d "Writes a shell script to extend ghr features"
complete -c ghr -n "__fish_is_arg_n 1" -a version -d "Prints the version of this application"

complete -c ghr -n "__fish_is_arg_n 2; and __fish_seen_subcommand_from cd path open" -a "(ghr list)"

#profile subcommands
complete -c ghr -n "__fish_is_arg_n 2; and __fish_seen_subcommand_from profile" -a list -d "Lists all configured profiles"
complete -c ghr -n "__fish_is_arg_n 2; and __fish_seen_subcommand_from profile" -a show -d "Shows a profile in TOML format"
complete -c ghr -n "__fish_is_arg_n 2; and __fish_seen_subcommand_from profile" -a apply -d "Apply a profile"

complete -c ghr -n "__fish_is_arg_n 3; and __fish_seen_subcommand_from profile; and __fish_seen_subcommand_from show apply" -a "(ghr profile list --short)"
