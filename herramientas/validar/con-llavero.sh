#!/usr/bin/env bash
# Ejecuta un comando con un llavero funcional en Linux.
#
#   ./con-llavero.sh cargo test -p arles-app
#
# En Windows y macOS el llavero es nativo y esto no hace falta. En Linux —y en
# los contenedores de CI— hay que levantar una sesión de D-Bus y un
# gnome-keyring desbloqueado, o el arranque toma siempre la rama de rechazo
# (ADR-0011) y la ruta feliz nunca se prueba.
#
# Motivo por el que existe: un «todo verde» que solo ejercita el camino de
# error no es una validación, es una coincidencia.

set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
    exec "$@"
fi

for binario in dbus-run-session gnome-keyring-daemon; do
    if ! command -v "$binario" >/dev/null 2>&1; then
        echo "ERROR: falta $binario." >&2
        echo "Instala:  apt-get install -y gnome-keyring dbus-x11" >&2
        exit 127
    fi
done

# Un llavero previo queda BLOQUEADO y solo se desbloquea con un diálogo gráfico,
# que en un contenedor no existe. Se parte de cero para que el demonio cree uno
# nuevo ya desbloqueado.
rm -rf "${HOME}/.local/share/keyrings" "${HOME}"/.cache/keyring-* 2>/dev/null || true

exec dbus-run-session -- bash -c '
    export $(echo -n "arles-validacion" \
        | gnome-keyring-daemon --unlock --components=secrets 2>/dev/null)
    sleep 1
    exec "$@"
' _ "$@"
