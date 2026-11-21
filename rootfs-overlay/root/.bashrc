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

# Keep $COLUMNS/$LINES correct when the window is resized.
shopt -s checkwinsize 2>/dev/null

# ------------------------------------------------------------
# History (unlimited)
# ------------------------------------------------------------
HISTFILE="$HOME/.bash_history"
HISTSIZE=1000000000
HISTFILESIZE=1000000000
shopt -s histappend 2>/dev/null

# ------------------------------------------------------------
# Colours (logo palette, see /etc/kyanite/colors.sh)
# ------------------------------------------------------------
[ -r /etc/kyanite/colors.sh ] && . /etc/kyanite/colors.sh

# ------------------------------------------------------------
# Prompt:  (blank line)
#          [rc] user@host:/path (branch) $
#   - [rc]     only shown, in red, when the last command failed
#   - (branch) git branch or short hash, read from .git/HEAD in pure shell
#              (no fork: forks are slow under proot)
#   - the $/# symbol is blue, or red after a failed command
# The prompt is rebuilt before every prompt by PROMPT_COMMAND. Sessions are
# often killed without a clean exit, so history is also written right away.
# ------------------------------------------------------------
_kx_host=
[ -r /etc/hostname ] && read -r _kx_host < /etc/hostname
: "${_kx_host:=kyanite}"

# Long paths wrap on phone screens: keep only the last 3 components.
PROMPT_DIRTRIM=3

# Sets _kx_br to the current git branch (or short hash); empty outside a repo.
_kx_branch() {
  local d=$PWD head
  _kx_br=
  while [ -n "$d" ]; do
    if [ -d "$d/.git" ]; then
      read -r head < "$d/.git/HEAD" 2>/dev/null || return 0
      case "$head" in
        'ref: refs/heads/'*) _kx_br=${head#ref: refs/heads/} ;;
        *)                   _kx_br=${head:0:7} ;;
      esac
      return 0
    fi
    d=${d%/*}
  done
}

_kx_prompt() {
  local rc=$? sym=$C_BLUE
  history -a 2>/dev/null
  PS1='\n'   # blank line before every prompt
  if [ "$rc" -ne 0 ]; then
    PS1="${PS1}\\[${C_RED}\\][${rc}]\\[${C_RESET}\\] "
    sym=$C_RED
  fi
  PS1="${PS1}\\[${C_BOLD}${C_LIGHT}\\]\\u@${_kx_host}\\[${C_RESET}\\]:\\[${C_BOLD}${C_BLUE}\\]\\w\\[${C_RESET}\\]"
  _kx_branch
  [ -n "$_kx_br" ] && PS1="${PS1} \\[${C_DIM}\\](${_kx_br})\\[${C_RESET}\\]"
  PS1="${PS1} \\[${sym}\\]\\\$\\[${C_RESET}\\] "
}
PROMPT_COMMAND=_kx_prompt

# ------------------------------------------------------------
# Colours for ls, less and man (directories in the light logo blue;
# the default dark blue is hard to read on dark backgrounds)
# ------------------------------------------------------------
if [ "$C_256" = 1 ]; then
  if [ "$C_TRUE" = 1 ]; then
    _b='38;2;83;185;251' _c='38;2;135;215;255' _g='38;2;143;207;154' _r='38;2;230;122;120'
  else
    _b='38;5;75' _c='38;5;117' _g='38;5;114' _r='38;5;210'
  fi
  export LS_COLORS="di=1;$_b:ln=$_c:ex=$_g:or=$_r:mi=$_r:ow=1;4;$_b:tw=1;4;$_b:st=1;4;$_b"
  export LESS_TERMCAP_md=$'\033[1;'"$_b"'m'   # bold  (man headings)
  export LESS_TERMCAP_us=$'\033[4;'"$_c"'m'   # underline (man arguments)
  export LESS_TERMCAP_me=$'\033[0m'
  export LESS_TERMCAP_ue=$'\033[0m'
  unset _b _c _g _r
fi
export LESS=-R

# ------------------------------------------------------------
# Aliases
# ------------------------------------------------------------
alias ls='ls --color=auto'
alias l='ls --color=auto'
alias la='ls --color=auto -a'
alias ll='ls --color=auto -lh'
alias grep='grep --color=auto'
alias v='vim'
clear() { printf '\033c'; }
c()     { printf '\033c'; }
cl()    { printf '\033c'; ls --color=auto; }
alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias .....='cd ../../../..'
alias pserv='python3 -m http.server'
alias gitc='git clone'
alias 775='chmod 775'
alias folder='du -h --max-depth=1'
alias h='history'
alias irc='weechat'
alias myip='curl -s https://ifconfig.me/ip'
alias quit='exit'
alias q='exit'
alias apk='kpkg'
alias kpk='kpkg'
alias pkg='kpkg'
alias kp='kpkg'
alias sb='. ~/.bashrc'
alias update='kpkg update && kpkg upgrade'

# ------------------------------------------------------------
# Functions
# ------------------------------------------------------------

# mkdir + cd:  mkcd path/to/dir
mkcd() {
  mkdir -p -- "$1" && cd -- "$1"
}

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
