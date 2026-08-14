#!/usr/bin/env fish

source ~/.config/fish/functions/utils.fish
. ~/.config/fish/functions/legacy.fish
source ~/.config/fish/conf.d/local.fish first_arg second_arg

function open_in_editor --description "open a file in the user's editor" --argument-names path
    set -q EDITOR
    or set EDITOR vim
    $EDITOR $path
end

function parse_args
    argparse 'h/help' 'v/verbose' -- $argv
    or return 1
    if set -q _flag_help
        echo "usage: parse_args [-h] [-v]"
        return 0
    end
    echo "verbose: "(set -q _flag_verbose; and echo yes; or echo no)
end

function classify
    set n $argv[1]
    if test $n -lt 0
        echo "negative"
    else if test $n -eq 0
        echo "zero"
    else
        echo "positive"
    end
end

function greet
    set name (test (count $argv) -gt 0; and echo $argv[1]; or echo "World")
    echo "Hello, $name!"
end

function sum_list
    set total 0
    for num in $argv
        set total (math $total + $num)
    end
    echo $total
end

function repeat_msg
    set msg $argv[1]
    set count $argv[2]
    for i in (seq 1 $count)
        echo $msg
    end
end

function setup_dir
    set dir $argv[1]
    if not test -d $dir
        mkdir -p $dir
        echo "Created: $dir"
    else
        echo "Exists: $dir"
    end
end

function count_down
    set n $argv[1]
    while test $n -gt 0
        echo $n
        set n (math $n - 1)
    end
end

function describe_day
    set day $argv[1]
    switch $day
        case Mon Tue Wed Thu Fri
            echo "Weekday"
        case Sat Sun
            echo "Weekend"
        case '*'
            echo "Unknown"
    end
end

function early_exit
    for i in (seq 1 10)
        if test $i -eq 5
            break
        end
        if test $i -eq 3
            continue
        end
        echo $i
    end
    return 0
end

greet "Fish"
open_in_editor /tmp/notes.txt
parse_args -v
classify -3
classify 0
classify 5
sum_list 1 2 3 4 5
repeat_msg "hello" 3
setup_dir /tmp/fish_test
count_down 3
describe_day Mon
early_exit
