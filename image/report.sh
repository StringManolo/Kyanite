#!/usr/bin/env bash
# Genera el resumen del job (GITHUB_STEP_SUMMARY) con el resultado de cada test y sus capturas.
# - Convierte los PNG a JPG, hace una hoja de contacto por caso (<=12 fotogramas + el ultimo)
# - Publica hojas y ultimas capturas en la rama huerfana "ci-screenshots" (solo las 6 ultimas ejecuciones)
#   para poder incrustarlas en el markdown (los artifacts no se pueden enlazar como imagen).
# Entorno: REPO, GH_TOKEN, GITHUB_RUN_ID, GITHUB_RUN_ATTEMPT, GITHUB_STEP_SUMMARY
set -uo pipefail

ROOT=shots
RUN="${GITHUB_RUN_ID:-0}-${GITHUB_RUN_ATTEMPT:-1}"
OUT="report/$RUN"
FONT=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf
SUMMARY=${GITHUB_STEP_SUMMARY:-/dev/stdout}
mkdir -p "$OUT"

# --- fotogramas -> JPG + hoja de contacto ------------------------------------------------
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

# --- publicar en la rama ci-screenshots -----------------------------------------------------
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
    ls -1 runs | sort -V | head -n -6 | xargs -r -I{} rm -rf "runs/{}"
    git checkout -q --orphan snapshot 2>/dev/null || git switch -q --orphan snapshot 2>/dev/null || true
    git add -A
    git commit -q -m "screenshots run $RUN"
    git push -q --force origin HEAD:refs/heads/ci-screenshots
  ); then
    HOSTED=1
  fi
fi
BASE="https://raw.githubusercontent.com/${REPO:-}/ci-screenshots/runs/${RUN}"

clean() { sed 's/\x1b\[[0-9;?]*[A-Za-z]//g' | tr -cd '\11\12\40-\176'; }

# --- resumen markdown -----------------------------------------------------------------------
{
  echo "## Tests de arranque de Kyanite"
  echo
  if [ ! -s "$ROOT/results.tsv" ]; then
    echo "No se llego a ejecutar ningun test (fallo la construccion). Mira los pasos anteriores del job."
  else
    echo "| Caso | Resultado | Motivo | Etapas vistas |"
    echo "|---|---|---|---|"
    while IFS=$'\t' read -r name status reason stages; do
      case "$status" in ok) icon="OK";; fail) icon="FALLO";; *) icon="OMITIDO";; esac
      echo "| $name | $icon | $reason | $stages |"
    done < "$ROOT/results.tsv"
    echo
    echo "_Etapas: kernel = el kernel arranco; root-ext4 = monto la raiz; live-init = el initramfs live llego a switch_root;"
    echo "instalador = termino la instalacion. \"Salida esperada\" es el regex del input \`expect\`._"
    echo
    while IFS=$'\t' read -r name status reason stages; do
      case "$status" in ok) icon="OK";; fail) icon="FALLO";; *) icon="OMITIDO";; esac
      open=""; [ "$status" = fail ] && open=" open"
      echo "<details$open><summary><b>$icon</b> $name - $reason ($stages)</summary>"
      echo
      if [ "$HOSTED" = 1 ] && [ -f "$OUT/$name/sheet.jpg" ]; then
        echo "Evolucion del arranque (una captura cada pocos segundos; el nombre es el segundo):"
        echo
        echo "![$name]($BASE/$name/sheet.jpg)"
        echo
        echo "Ultima pantalla:"
        echo
        echo "![$name ultima]($BASE/$name/last.jpg)"
        echo
      fi
      if [ -f "$ROOT/$name/serial.log" ]; then
        echo "Final de la consola serie:"
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
    echo "Las capturas se publican en la rama \`ci-screenshots\` (se conservan las ultimas 6 ejecuciones)."
  else
    echo "No se pudieron incrustar las capturas (no se pudo publicar la rama \`ci-screenshots\`)."
  fi
  echo "Todas las capturas y los logs completos estan en el artifact **kyanite-ci-shots**; las imagenes en **kyanite-x86_64-images**."
} >> "$SUMMARY"
