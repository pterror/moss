#!/usr/bin/env bash
set -euo pipefail

source ./utils.sh
. ./config.sh

classify() {
    local n="$1"
    if (( n < 0 )); then
        echo "negative"
    elif (( n == 0 )); then
        echo "zero"
    else
        echo "positive"
    fi
}

sum_array() {
    local total=0
    for num in "$@"; do
        (( total += num ))
    done
    echo "$total"
}

greet() {
    local name="${1:-World}"
    echo "Hello, ${name}!"
}

repeat() {
    local msg="$1"
    local count="$2"
    local i=0
    while (( i < count )); do
        echo "$msg"
        (( i++ ))
    done
}

setup_environment() {
    local dir="${1:-.}"
    if [[ ! -d "$dir" ]]; then
        mkdir -p "$dir"
    fi
    echo "Environment ready: $dir"
}

main() {
    greet "Bash"
    classify -3
    classify 0
    classify 5
    sum_array 1 2 3 4 5
    repeat "hello" 3
    setup_environment "/tmp/test_env"
}

scan() {
    local -a items=("$@")
    local i=0
    for item in "${items[@]}"; do
        if [[ "$item" == "skip" ]]; then
            continue
        fi
        if [[ "$item" == "stop" ]]; then
            break
        fi
        if [[ -z "$item" ]]; then
            return 1
        fi
        echo "$item"
    done
    return 0
}

# `function` keyword syntax (the other of Bash's two function-definition
# forms; `classify`/`sum_array`/etc. above use the bare `name() { }` form).
function cleanup {
    local dir="$1"
    rm -rf "$dir"
}

function build_report() {
    local -a lines=()
    lines+=("start")
    lines+=("end")
    printf '%s\n' "${lines[@]}"
}

# Pipelines and subshells.
count_lines() {
    grep -c "^" "$1" | tr -d ' '
}

with_subshell() {
    (
        cd /tmp || return 1
        pwd
    )
}

# Here-doc.
print_usage() {
    cat <<EOF
Usage: $0 [options]
  -h  show help
EOF
}

# case/esac with fallthrough and multi-pattern arms.
classify_ext() {
    local file="$1"
    case "$file" in
        *.sh|*.bash)
            echo "shell"
            ;;
        *.md)
            echo "markdown"
            ;&
        *.txt)
            echo "text"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

# C-style for loop, short-circuit &&/||, arithmetic ternary.
retry() {
    local attempts="$1"
    local -i i
    if [[ -n "$attempts" && "$attempts" -gt 0 ]]; then
        for (( i = 0; i < attempts; i++ )); do
            run_step "$i" && echo "ok" || echo "retrying"
        done
    fi
    local -i level=$(( attempts > 3 ? 2 : 1 ))
    echo "level=$level"
}

run_step() {
    return 0
}

# trap handler.
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

main "$@"
