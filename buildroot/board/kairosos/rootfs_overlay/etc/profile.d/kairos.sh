#!/bin/sh
# KairosOS environment setup

# Set up Kairos paths
export PATH="$PATH:/opt/hermes/bin"

# Set default editor
EDITOR=nano

# Kairos prompt
if [ "$TERM" != "dumb" ]; then
    if command -v tput >/dev/null 2>&1; then
        GREEN=$(tput setaf 2)
        CYAN=$(tput setaf 6)
        RESET=$(put sgr0 2>/dev/null || tput sgr0)
        PS1="${GREEN}kairos${CYAN}@${RESET}\h ${CYAN}\w${RESET}\n${GREEN}›${RESET} "
    fi
fi

# Welcome message on first login
if [ -f /usr/share/kairos/welcome.txt ] && [ ! -f /home/kairos/.kairos-welcomed ]; then
    cat /usr/share/kairos/welcome.txt
    touch /home/kairos/.kairos-welcomed
fi
