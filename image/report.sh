#!/usr/bin/env bash
# Writes the job summary (GITHUB_STEP_SUMMARY) with the result of each test and its screenshots.
# - Converts PNG frames to JPG and builds one contact sheet per case (<= 12 frames + the last one)
# - Publishes sheets and last screenshots to the orphan branch "ci-screenshots" (only the last runs are kept)
#   so they can be embedded in the markdown (artifacts cannot be linked as images)
# Environment: REPO, GH_TOKEN, VARIANT, GITHUB_RUN_ID, GITHUB_RUN_ATTEMPT, GITHUB_STEP_SUMMARY
set -uo pipefail

ROOT=shots
VARIANT=${VARIANT:-vm}
RUN="${GITHUB_RUN_ID:-0}-${GITHUB_RUN_ATTEMPT:-1}-${VARIANT}"
OUT="report/$RUN"
FONT=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf
SUMMARY=${GITHUB_STEP_SUMMARY:-/dev/stdout}
KEEP_RUNS=12
mkdir -p "$OUT"

# --- frames -> JPG + contact sheet ---------------------------------------------------------------
FONTARGS=()
[ -f "$FONT" ] && FONTARGS=(-font "$FONT")
for dir in "$ROOT"/*/; do
  [ -d "$dir" ] || continue
  name=$(basename "$dir")
  ls "$dir"frame-*.png >/dev/null 2>&1 || continue
  mogrify -format jpg -quality 65 -resize '960x>' "$dir"frame-*.png && rm -f "$dir"frame-*.png
  mapfile -t F < <(ls "$dir"frame-*.jpg 2>/dev/null)
  n=${#F[@]}
  [ "$n" -gt 0 ] || continue
  step=$(( (n + 11) / 12 ))
  sel=()
  for ((i = 0; i < n; i += step)); do sel+=("${F[i]}"); done
  [ "${sel[-1]}" = "${F[n-1]}" ] || sel+=("${F[n-1]}")
  mkdir -p "$OUT/$name"
  montage "${FONTARGS[@]}" -pointsize 14 -label '%t' "${sel[@]}" -tile 4x -geometry '400x300+4+4' \
    -background '#202020' -fill white "$OUT/$name/sheet.jpg" 2>/dev/null
  cp "${F[n-1]}" "$OUT/$name/last.jpg"
done

# --- publish to the ci-screenshots branch ---------------------------------------------------------
HOSTED=0
if [ -n "$(ls -A "$OUT" 2>/dev/null)" ] && [ -n "${GH_TOKEN:-}" ] && [ -n "${REPO:-}" ]; then
  URL="https://x-access-token:${GH_TOKEN}@github.com/${REPO}.git"
  rm -rf evidence
  if ! git clone -q --depth 1 --branch ci-screenshots "$URL" evidence 2>/dev/null; then
    git init -q evidence
  fi
  if (
    cd evidence || exit 1
    git config user.name "github-actions"
    git config user.email "github-actions@users.noreply.github.com"
    git remote get-url origin >/dev/null 2>&1 || git remote add origin "$URL"
    mkdir -p runs
    cp -r "../$OUT" "runs/$RUN"
    ls -1 runs | sort -V | head -n "-$KEEP_RUNS" | xargs -r -I{} rm -rf "runs/{}"
    git checkout -q --orphan snapshot 2>/dev/null || git switch -q --orphan snapshot 2>/dev/null || true
    git add -A
    git commit -q -m "screenshots run $RUN"
    git push -q --force origin HEAD:refs/heads/ci-screenshots
  ); then
    HOSTED=1
  fi
fi
BASE="https://raw.githubusercontent.com/${REPO:-}/ci-screenshots/runs/${RUN}"

# Strip escape sequences (CSI like ESC[6n or ESC[=3h, and 2-byte ones like ESC7/ESC8) and keep plain text
clean() { sed 's/\x1b\[[0-9;?=<>]*[A-Za-z]//g; s/\x1b[()][A-Za-z0-9]//g; s/\x1b[78=>]//g' | tr -cd '\11\12\40-\176'; }

# --- markdown summary ---------------------------------------------------------------------------------
{
  echo "## Kyanite boot tests - ${VARIANT}"
  echo
  if [ ! -s "$ROOT/results.tsv" ]; then
    echo "No test was run (the build failed). Check the previous steps of the job."
  else
    echo "| Case | Result | Reason | Stages seen |"
    echo "|---|---|---|---|"
    while IFS=$'\t' read -r name status reason stages; do
      case "$status" in ok) icon="OK";; fail) icon="FAIL";; warn) icon="WARNING";; *) icon="SKIPPED";; esac
      echo "| $name | $icon | $reason | $stages |"
    done < "$ROOT/results.tsv"
    echo
    echo "_Stages: kernel = the kernel booted; root-ext4 = the root filesystem was mounted; live-init = the live initramfs reached switch_root;"
    echo "hw-setup = driver coldplug ran; smoke-script = the smoke test ran; installer = the installation finished."
    echo "\"Expected output\" is the regex of the \`expect\` input. A WARNING does not fail the workflow._"
    echo
    while IFS=$'\t' read -r name status reason stages; do
      case "$status" in ok) icon="OK";; fail) icon="FAIL";; warn) icon="WARNING";; *) icon="SKIPPED";; esac
      open=""; if [ "$status" = fail ] || [ "$status" = warn ]; then open=" open"; fi
      echo "<details$open><summary><b>$icon</b> $name - $reason ($stages)</summary>"
      echo
      if [ "$HOSTED" = 1 ] && [ -f "$OUT/$name/sheet.jpg" ]; then
        echo "Boot progress (one screenshot every few seconds; the label is the second):"
        echo
        echo "![$name]($BASE/$name/sheet.jpg)"
        echo
        echo "Last screen:"
        echo
        echo "![$name last]($BASE/$name/last.jpg)"
        echo
      fi
      if [ -f "$ROOT/$name/serial.log" ]; then
        echo "End of the serial console:"
        echo
        echo '```'
        tail -n 25 "$ROOT/$name/serial.log" | clean
        echo '```'
      fi
      echo "</details>"
      echo
    done < "$ROOT/results.tsv"
  fi
  if [ "$HOSTED" = 1 ]; then
    echo "Screenshots are published on the \`ci-screenshots\` branch (the last ${KEEP_RUNS} runs are kept)."
  else
    echo "Screenshots could not be embedded (publishing to the \`ci-screenshots\` branch failed or the repository is private)."
  fi
  echo "All screenshots and full serial logs are in the artifact **kyanite-ci-shots-${VARIANT}**; the images are in the image artifact of this variant."
} >> "$SUMMARY"
