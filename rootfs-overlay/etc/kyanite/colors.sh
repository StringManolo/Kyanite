# /etc/kyanite/colors.sh - colour helpers shared by the MOTD scripts and ~/.bashrc (sourced).
#
# Sets C_RESET, C_BOLD, C_BLUE, C_LIGHT, C_DIM and C_RED (all empty when colour
# is disabled: NO_COLOR set, TERM unset or "dumb"), C_256=1 when the terminal
# supports 256 colours and C_TRUE=1 when it advertises 24-bit colour
# (COLORTERM=truecolor|24bit, or TERM containing "truecolor"/"direct").
#
# With C_TRUE=1 the blues are the exact logo colours: C_BLUE #1277f6 and
# C_LIGHT #53b9fb.
C_RESET= C_BOLD= C_BLUE= C_LIGHT= C_DIM= C_RED= C_256=0 C_TRUE=0

if [ -z "${NO_COLOR:-}" ] && [ -n "${TERM:-}" ] && [ "$TERM" != dumb ]; then
  _esc=$(printf '\033')
  C_RESET="${_esc}[0m"
  C_BOLD="${_esc}[1m"
  case "$TERM" in
    *256color*|*truecolor*|*direct*)
      C_256=1
      C_BLUE="${_esc}[38;5;33m"
      C_LIGHT="${_esc}[38;5;75m"
      C_DIM="${_esc}[38;5;244m"
      C_RED="${_esc}[38;5;210m"
      ;;
    *)
      C_BLUE="${_esc}[34m"
      C_LIGHT="${_esc}[36m"
      C_DIM="${_esc}[2m"
      C_RED="${_esc}[31m"
      ;;
  esac
  case "${COLORTERM:-}:$TERM" in
    truecolor:*|24bit:*|*:*truecolor*|*:*direct*)
      C_256=1 C_TRUE=1
      C_BLUE="${_esc}[38;2;18;119;246m"
      C_LIGHT="${_esc}[38;2;83;185;251m"
      C_DIM="${_esc}[38;5;244m"
      C_RED="${_esc}[38;2;230;122;120m"
      ;;
  esac
  unset _esc
fi
