# ~/.bashrc - Kyanite interactive shell configuration.
# Loaded from /etc/profile for login shells (Brush does not read it by itself).

# Interactive shells only.
case "$-" in
  *i*) ;;
  *) return 0 ;;
esac

# ------------------------------------------------------------
# Environment
# ------------------------------------------------------------
export EDITOR=vi
export VISUAL=vi

# ------------------------------------------------------------
# History (unlimited)
# ------------------------------------------------------------
HISTFILE="$HOME/.bash_history"
HISTSIZE=1000000000
HISTFILESIZE=1000000000
shopt -s histappend 2>/dev/null
# Write each command right away: sessions are often killed without a clean exit.
PROMPT_COMMAND='history -a 2>/dev/null'

# ------------------------------------------------------------
# Prompt: user@host:/path#   (host comes from /etc/hostname)
# ------------------------------------------------------------
_host=
[ -r /etc/hostname ] && read -r _host < /etc/hostname
: "${_host:=kyanite}"

if [ -z "${NO_COLOR:-}" ] && [ -n "${TERM:-}" ] && [ "$TERM" != dumb ]; then
  case "$TERM" in
    *256color*|*truecolor*|*direct*) _c_user='38;5;75'; _c_path='38;5;33' ;;
    *)                               _c_user='36';      _c_path='34' ;;
  esac
  PS1='\[\033[1;'"$_c_user"'m\]\u@'"$_host"'\[\033[0m\]:\[\033[1;'"$_c_path"'m\]\w\[\033[0m\]\$ '
  unset _c_user _c_path
else
  PS1='\u@'"$_host"':\w\$ '
fi
unset _host

# ------------------------------------------------------------
# Aliases
# ------------------------------------------------------------
alias ls='ls --color=auto'
alias l='ls'
alias la='ls -a'
alias ll='ls -lh'
alias v='vim'
alias c='clear'
alias cl='clear && ls'
alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias .....='cd ../../../..'
alias pserv='python3 -m http.server'
alias gitc='git clone'
alias 775='chmod +775'
alias folder='du -h --max-depth=1'
alias h='history'
alias irc='weechat'
alias myip='curl -s https://ifconfig.me/ip'
alias quit='exit'
alias q='exit'
alias sb='source ~/.bashrc'
alias update='kpkg update && kpkg upgrade'

# ------------------------------------------------------------
# Functions
# ------------------------------------------------------------

# git add + commit + push:  gacp "commit message"
gacp() {
  git add --all && git commit -m "$*" && git push
}

# Prints a hint when a tool needed by extract() is missing.
_kx_need() {
  command -v "$1" >/dev/null 2>&1 && return 0
  echo "extract: '$1' not found. Install it with: kpkg add $2" >&2
  return 1
}

# Extracts any archive(s).
extract() {
  if [ $# -eq 0 ]; then
    echo "usage: extract <archive>..." >&2
    return 1
  fi
  for archive in "$@"; do
    if [ ! -f "$archive" ]; then
      echo "extract: '$archive' is not a valid file" >&2
      continue
    fi
    case "$archive" in
      *.tar.bz2|*.tbz2|*.tbz) _kx_need tar tar && _kx_need bzip2 bzip2 && tar xvjf "$archive" ;;
      *.tar.gz|*.tgz)         _kx_need tar tar && _kx_need gzip gzip && tar xvzf "$archive" ;;
      *.tar.xz|*.txz)         _kx_need tar tar && _kx_need xz xz-utils && tar xvJf "$archive" ;;
      *.tar.zst|*.tzst)       _kx_need tar tar && _kx_need zstd zstd && tar --zstd -xvf "$archive" ;;
      *.tar)                  _kx_need tar tar && tar xvf "$archive" ;;
      *.bz2)                  _kx_need bunzip2 bzip2 && bunzip2 "$archive" ;;
      *.gz)                   _kx_need gunzip gzip && gunzip "$archive" ;;
      *.xz)                   _kx_need unxz xz-utils && unxz "$archive" ;;
      *.zst)                  _kx_need unzstd zstd && unzstd "$archive" ;;
      *.Z)                    _kx_need uncompress ncompress && uncompress "$archive" ;;
      *.zip|*.jar|*.war)      _kx_need unzip unzip && unzip "$archive" ;;
      *.7z)                   _kx_need 7z 7zip && 7z x "$archive" ;;
      *.rar)
        if command -v unrar >/dev/null 2>&1; then
          unrar x "$archive"
        else
          _kx_need 7z 7zip && 7z x "$archive"
        fi
        ;;
      *) echo "extract: don't know how to extract '$archive'" >&2 ;;
    esac
  done
}
