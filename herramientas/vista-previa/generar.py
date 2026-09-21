#!/usr/bin/env python3
"""Empaqueta ARLES en **un solo archivo HTML** que se abre con doble clic.

    python3 herramientas/vista-previa/generar.py

Deja `vista-previa/ARLES-vista-previa.html`, sin nada al lado: la hoja de
estilos, el JavaScript, las cuatro tipografías y las imágenes viajan dentro.

─────────────────────────────────────────────────────────────────────────────
PARA QUÉ ES, Y QUÉ NO ES

Dirección pidió «solo preocuparme de descargar y probar». El instalador de
verdad no puede salir de aquí —los bundles son de Windows y de macOS, y esto es
Linux— y construirlo en GitHub exige etiquetas, permisos y veinte minutos.

Esto es otra cosa y llega antes: **la interfaz real**, con sus componentes, sus
tokens, sus dos temas y sus textos, corriendo en el navegador que ya está
instalado. Sirve para recorrer los bloques A a F de la checklist visual, que es
lo que había que revisar.

**Lo que NO es:**

- **No es la aplicación de escritorio.** No hay ventana nativa, ni base de datos
  cifrada, ni llavero del sistema. Lo que hay detrás es el **mismo núcleo
  simulado que usan las sondas**, guardando en el almacenamiento de la pestaña.
- Por tanto **nada de lo que escribas se guarda de verdad**. Sobrevive a
  recargar la página —que es lo que hace falta para probar «cerrar y volver a
  abrir»— y desaparece al cerrar la pestaña.
- **No sustituye a la revisión en la aplicación real**, y muy concretamente no
  sustituye a la de macOS (R-07): ahí el motor es WKWebView y aquí es el del
  navegador de quien lo abra.

── Sobre la tipografía ──

Esta vista previa **sí lleva Mont**, al contrario que el instalador. No es una
contradicción: el instalador se **distribuye**, y eso es lo que la licencia de
Fontfabric no cubre mientras P-01 siga sin confirmarse. Esto es una vista previa
interna para quien tiene esa licencia, equivalente a mirar el `vite dev` del
equipo. Si Dirección prefiere que tampoco la lleve, se genera con
`--sin-mont`.
─────────────────────────────────────────────────────────────────────────────
"""

from __future__ import annotations

import argparse
import base64
import mimetypes
import re
import shutil
import subprocess
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[2]
APP = RAIZ / "app"
DIST = APP / "dist-vista-previa"
SALIDA = RAIZ / "vista-previa" / "ARLES-vista-previa.html"

#: El núcleo simulado. Es el mismo contrato que `arles-app`, con dos
#: diferencias deliberadas:
#:
#:   · Guarda en `sessionStorage`, no en una base cifrada. Así «cerrar y volver
#:     a abrir» —recargar la página— conserva el tema y la barra plegada, que es
#:     lo que piden los renglones B-5 y E-3 de la checklist.
#:   · Empieza **sin empresa configurada**, para que lo primero que se vea sea
#:     la portada de primera vez (bloque A).
NUCLEO_SIMULADO = """
<script>
(function () {
  var K = 'arles-vista-previa';
  function leer() {
    try { return JSON.parse(sessionStorage.getItem(K) || '{}'); } catch (e) { return {}; }
  }
  function escribir(e) {
    try { sessionStorage.setItem(K, JSON.stringify(e)); } catch (e2) {}
  }
  function ahora() { return new Date().toISOString(); }

  // Reproduce lo que hace `arles_core::analizar` EN LO QUE SE VE: reparte cada
  // fila en lista, choque, inválida o rechazada, y numera las filas como Excel
  // —la primera de datos es la 2—.
  function analizar(e, imp, mapeo) {
    var inf = { filasDeLasListas: [], choques: [], rechazadas: [], invalidas: [], _listas: [] };
    var vistas = {};
    var ya = {};
    (e.contactos || []).forEach(function (c) {
      var nom = [c.nombre, c.apellido].filter(Boolean).join(' ');
      c.canales.forEach(function (k) { ya[k.valorNormalizado] = nom; });
    });

    imp.filas.forEach(function (fila, i) {
      var n = i + 2;
      var ref = fila.find(function (c) { return String(c || '').trim(); }) || '';
      var nombre = '', apellido = '', empresa = '', canales = [], malo = null;

      mapeo.forEach(function (campo, j) {
        var v = String(fila[j] || '').trim();
        if (!v) return;
        if (campo === 'nombre') nombre = v;
        else if (campo === 'apellido') apellido = v;
        else if (campo === 'empresa') empresa = v;
        else if (campo === 'nombreCompleto') {
          var p = v.split(' ');
          nombre = p.shift(); apellido = p.join(' ');
        } else if (campo === 'correo' || campo === 'whatsapp') {
          var norm;
          if (campo === 'correo') {
            if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(v)) {
              malo = malo || 'contacto.error.correoInvalido';
              return;
            }
            norm = v.toLowerCase();
          } else {
            var d = v.replace(/[^0-9]/g, '');
            if (d.length === 13 && d.slice(0, 3) === '521') d = '52' + d.slice(3);
            if (d.length === 10) d = '52' + d;
            if (d.length !== 12) { malo = malo || 'contacto.error.telefonoInvalido'; return; }
            norm = '+' + d;
          }
          canales.push({
            canal: campo === 'correo' ? 'email' : 'whatsapp',
            valorRaw: v,
            valorNormalizado: norm,
            principal: !canales.some(function (k) {
              return k.canal === (campo === 'correo' ? 'email' : 'whatsapp');
            }),
          });
        }
      });

      if (malo) { inf.invalidas.push({ fila: n, motivo: malo, referencia: ref }); return; }
      if (!canales.length) {
        inf.rechazadas.push({ fila: n, motivo: 'importacion.rechazo.sinCanales', referencia: ref });
        return;
      }

      var choco = false;
      canales.forEach(function (k) {
        if (ya[k.valorNormalizado] !== undefined) {
          inf.choques.push({ fila: n, direccion: k.valorNormalizado, con: ya[k.valorNormalizado], dentroDelArchivo: false });
          choco = true;
        } else if (vistas[k.valorNormalizado] !== undefined) {
          inf.choques.push({ fila: n, direccion: k.valorNormalizado, con: String(vistas[k.valorNormalizado]), dentroDelArchivo: true });
          choco = true;
        }
      });
      if (choco) return;

      canales.forEach(function (k) { vistas[k.valorNormalizado] = n; });
      inf.filasDeLasListas.push(n);
      inf._listas.push({
        id: 'c' + Math.random().toString(36).slice(2, 10),
        nombre: nombre, apellido: apellido, empresa: empresa,
        canales: canales.sort(function (a, b) {
          if (a.canal !== b.canal) return a.canal === 'email' ? -1 : 1;
          return (b.principal ? 1 : 0) - (a.principal ? 1 : 0);
        }),
        creadoEn: ahora(), actualizadoEn: ahora(),
      });
    });
    return inf;
  }

  window.__TAURI_INTERNALS__ = {
    invoke: function (cmd, args) {
      var e = leer();
      switch (cmd) {
        case 'info_app':
          return Promise.resolve({
            nombre: 'ARLES RELAY',
            nombreComercial: 'ARLES RELAY I',
            version: '1.2.0',
            atribucion: 'Software desarrollado por TELEMETRY INSIGHT',
          });

        case 'estado_arranque':
          return Promise.resolve({
            baseDeDatosLista: true,
            versionEsquema: 4,
            empresaConfigurada: !!e.empresa,
          });

        case 'configuracion_de_empresa':
          return Promise.resolve(configuracion(e));

        case 'guardar_empresa': {
          var b = args.borrador || {};
          var malos = [];
          if (!String(b.nombreComercial || '').trim()) {
            malos.push({ campo: 'nombreComercial', clave: 'empresa.error.nombreVacio' });
          }
          var correo = String(b.correoCorporativo || '').trim();
          if (!correo) {
            malos.push({ campo: 'correoCorporativo', clave: 'empresa.error.correoVacio' });
          } else if (!/^[^@\\s]+@[^@\\s]+\\.[^@\\s]+$/.test(correo)) {
            malos.push({ campo: 'correoCorporativo', clave: 'empresa.error.correoInvalido' });
          }
          var sitio = String(b.sitioWeb || '').trim();
          if (sitio && !/^https?:\\/\\//.test(sitio)) {
            malos.push({ campo: 'sitioWeb', clave: 'empresa.error.sitioSinEsquema' });
          }
          if (malos.length) {
            return Promise.reject({
              clave: 'error.app.empresa_invalida', detalle: '', campos: malos,
            });
          }
          e.empresa = {
            nombreComercial: String(b.nombreComercial).trim(),
            pais: b.pais || 'MX',
            zonaHoraria: b.zonaHoraria || 'America/Mexico_City',
            correoCorporativo: correo,
            sitioWeb: sitio || null,
            creadaEn: e.empresa ? e.empresa.creadaEn : ahora(),
            actualizadaEn: ahora(),
          };
          escribir(e);
          return Promise.resolve(configuracion(e));
        }

        case 'preferencias_de_interfaz':
          return Promise.resolve({
            barraLateralPlegada: !!e.plegada,
            tema: e.tema || 'auto',
          });

        case 'guardar_barra_plegada':
          e.plegada = !!args.plegada; escribir(e); return Promise.resolve();

        case 'guardar_tema':
          e.tema = args.tema; escribir(e); return Promise.resolve();

        // ── Contactos (entrega 3.2) ──────────────────────────────────
        //
        // Reproduce lo que hace el núcleo de verdad EN LO QUE SE VE: normaliza
        // el correo a minúsculas y el móvil a E.164 quitando el «1» mexicano,
        // ordena los canales como L-14 y rechaza una dirección repetida con la
        // dirección dentro del error.
        //
        // No es el núcleo: aquí no hay base cifrada ni supresión de verdad. Es
        // lo justo para que la pantalla se pueda recorrer y juzgar.
        case 'listar_contactos': {
          var lista = (e.contactos || []).slice();
          return Promise.resolve({
            contactos: lista,
            total: lista.length,
            // Una dirección de muestra en la lista de no escribir, para que el
            // aviso de supresión se pueda ver sin montar nada.
            suprimidas: lista.length
              ? lista
                  .flatMap(function (c) { return c.canales; })
                  .filter(function (k) { return /(^|\.)ejemplo\./.test(k.valorNormalizado); })
                  .map(function (k) { return k.valorNormalizado; })
              : [],
          });
        }

        case 'crear_contacto':
        case 'editar_contacto': {
          var b2 = args.borrador || {};
          var otros = (e.contactos || []).filter(function (c) {
            return c.id !== args.id;
          });
          var malos2 = [];
          var canales = [];

          (b2.canales || []).forEach(function (k, i) {
            var v = String(k.valor || '').trim();
            if (!v) {
              malos2.push({ campo: 'canal', indice: i, clave: 'error.canal_vacio' });
              return;
            }
            var norm;
            if (k.canal === 'email') {
              if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(v)) {
                malos2.push({ campo: 'canal', indice: i, clave: 'error.email_invalido' });
                return;
              }
              norm = v.toLowerCase();
            } else {
              var d = v.replace(/[^0-9]/g, '');
              // El «1» del móvil mexicano: +52 1 55… y +52 55… son el mismo
              // número. Guardar los dos rompería la deduplicación.
              if (d.length === 13 && d.slice(0, 3) === '521') d = '52' + d.slice(3);
              if (d.length === 10) d = '52' + d;
              if (d.length !== 12 || d.slice(0, 2) !== '52') {
                malos2.push({ campo: 'canal', indice: i, clave: 'error.telefono_invalido' });
                return;
              }
              norm = '+' + d;
            }
            canales.push({
              canal: k.canal,
              valorRaw: v,
              valorNormalizado: norm,
              principal: !!k.principal,
            });
          });

          if (malos2.length) {
            return Promise.reject({
              clave: 'error.app.contacto_invalido', detalle: '', canales: malos2,
            });
          }
          if (!canales.length) {
            return Promise.reject({
              clave: 'error.app.contacto_invalido',
              detalle: '',
              canales: [{ campo: 'canales', indice: null, clave: 'error.sin_canales' }],
            });
          }

          // Una dirección sólo puede ser de un contacto.
          var choque = null;
          canales.forEach(function (k) {
            otros.forEach(function (c) {
              c.canales.forEach(function (o) {
                if (o.valorNormalizado === k.valorNormalizado) choque = k;
              });
            });
          });
          if (choque) {
            return Promise.reject({
              clave: 'error.db.direccion_en_uso',
              detalle: choque.valorNormalizado,
              canales: [],
            });
          }

          // L-14: correos primero, móviles después, el principal de cada tipo
          // arriba. El orden sale de aquí una sola vez, como en la base.
          ['email', 'whatsapp'].forEach(function (tipo) {
            var deEse = canales.filter(function (k) { return k.canal === tipo; });
            if (deEse.length && !deEse.some(function (k) { return k.principal; })) {
              deEse[0].principal = true;
            }
          });
          canales.sort(function (a, b3) {
            if (a.canal !== b3.canal) return a.canal === 'email' ? -1 : 1;
            return (b3.principal ? 1 : 0) - (a.principal ? 1 : 0);
          });

          var ficha = {
            id: args.id || 'c' + Math.random().toString(36).slice(2, 10),
            nombre: String(b2.nombre || '').trim(),
            apellido: String(b2.apellido || '').trim(),
            empresa: String(b2.empresa || '').trim(),
            canales: canales,
            creadoEn: ahora(),
            actualizadoEn: ahora(),
          };
          e.contactos = otros.concat([ficha]);
          escribir(e);
          return Promise.resolve(ficha.id);
        }

        case 'borrar_contacto': {
          e.contactos = (e.contactos || []).filter(function (c) {
            return c.id !== args.id;
          });
          escribir(e);
          return Promise.resolve();
        }

        // ── Importación (entrega 3.3) ────────────────────────────────
        //
        // Sin diálogo del sistema: un navegador no lo tiene. Se inventa un
        // archivo de muestra con las trampas que importan —encabezados en
        // español, el «1» del móvil mexicano, una fila sin contacto y una
        // repetida— para que la pantalla se pueda recorrer entera.
        case 'elegir_archivo_para_importar': {
          var enc = ['Nombre', 'Apellidos', 'Razón social', 'Correo electrónico', 'Teléfono móvil'];
          var filas = [
            ['Ana', 'Ruiz', 'Empresa SA', 'Ana@Empresa.MX', '+52 1 81 1234 5678'],
            ['Luis', 'Pérez', 'Otra SA', 'luis@empresa.mx', ''],
            ['Sin', 'Contacto', 'Tercera SA', '', ''],
            ['Mal', 'Escrito', 'Cuarta SA', 'esto-no-es-un-correo', ''],
            ['Repe', 'Tida', 'Quinta SA', 'ana@empresa.mx', ''],
          ];
          e.importacion = { enc: enc, filas: filas };
          escribir(e);
          return Promise.resolve({
            nombre: 'contactos-de-muestra.csv',
            encabezados: enc,
            muestra: filas,
            mapeo: ['nombre', 'apellido', 'empresa', 'correo', 'whatsapp'],
            totalDeFilas: filas.length,
          });
        }

        case 'analizar_importacion': {
          var imp = e.importacion;
          if (!imp) {
            return Promise.reject({ clave: 'error.app.sin_importacion_en_curso', detalle: '' });
          }
          return Promise.resolve(analizar(e, imp, args.mapeo));
        }

        case 'confirmar_importacion': {
          var imp2 = e.importacion;
          if (!imp2) {
            return Promise.reject({ clave: 'error.app.sin_importacion_en_curso', detalle: '' });
          }
          var inf = analizar(e, imp2, args.mapeo);
          e.contactos = (e.contactos || []).concat(inf._listas);
          delete e.importacion;
          escribir(e);
          return Promise.resolve({
            importados: inf._listas.length,
            choques: inf.choques.length,
            invalidas: inf.invalidas.length + inf.rechazadas.length,
            totalDeFilas: imp2.filas.length,
          });
        }

        case 'cancelar_importacion':
          delete e.importacion; escribir(e); return Promise.resolve();

        case 'abrir_sitio_de_telemetry':
          window.open('https://telemetrymx.com', '_blank', 'noopener');
          return Promise.resolve();

        default:
          return Promise.reject({ clave: 'error.db.sqlite', detalle: 'desconocido: ' + cmd });
      }
    },
  };

  function configuracion(e) {
    var hay = !!e.empresa;
    var pasos = [
      ['empresa', hay, true, '3.1', '/ajustes'],
      ['remitente', false, false, '5', null],
      ['contactos', false, false, '3.2', null],
      ['plantilla', false, false, '6', null],
      ['ventana', false, false, '6', null],
      ['campana', false, false, '6', null],
    ].map(function (p) {
      return {
        paso: p[0], clave: p[0], completado: p[1], disponible: p[2],
        entrega: p[3], ruta: p[4],
      };
    });
    return {
      empresa: e.empresa || null,
      zonas: ['America/Mexico_City', 'America/Cancun', 'America/Merida',
              'America/Monterrey', 'America/Matamoros', 'America/Chihuahua',
              'America/Ojinaga', 'America/Mazatlan', 'America/Bahia_Banderas',
              'America/Hermosillo', 'America/Tijuana', 'UTC'],
      paises: ['MX'],
      onboarding: {
        pasos: pasos,
        completados: hay ? 1 : 0,
        total: 6,
        siguiente: hay ? 'remitente' : 'empresa',
      },
    };
  }
})();
</script>
"""


def corre(cmd: list[str], cwd: Path) -> None:
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)
    if r.returncode != 0:
        print(r.stdout[-3000:], file=sys.stderr)
        print(r.stderr[-3000:], file=sys.stderr)
        raise SystemExit(f"falló: {' '.join(cmd)}")


def incrustar_en_css(css: str, base: Path) -> str:
    """Mete fuentes e imágenes dentro del CSS como `data:`."""

    def uno(m: re.Match[str]) -> str:
        ref = m.group(1).strip("'\"")
        if ref.startswith(("data:", "http:", "https:")):
            return m.group(0)
        archivo = base / Path(ref).name
        if not archivo.exists():
            return m.group(0)
        tipo = mimetypes.guess_type(archivo.name)[0] or "application/octet-stream"
        datos = base64.b64encode(archivo.read_bytes()).decode("ascii")
        return f"url(data:{tipo};base64,{datos})"

    return re.sub(r"url\(([^)]+)\)", uno, css)


def main() -> int:
    p = argparse.ArgumentParser(description="ARLES en un solo archivo HTML")
    p.add_argument("--sin-mont", action="store_true", help="sin la tipografía de la marca")
    args = p.parse_args()

    if args.sin_mont:
        corre(["python3", "herramientas/marca/sin-mont.py", "--aplicar"], RAIZ)

    print("Compilando el frontend…")
    corre(
        [
            "npx", "vite", "build",
            "--config", "vite.config.vista-previa.ts",
            "--outDir", "dist-vista-previa", "--emptyOutDir",
        ],
        APP,
    )

    html = (DIST / "index.html").read_text(encoding="utf-8")
    activos = DIST / "assets"

    # El CSS, con las fuentes y las máscaras dentro.
    for m in re.finditer(r'<link[^>]+href="([^"]+\.css)"[^>]*>', html):
        archivo = activos / Path(m.group(1)).name
        css = incrustar_en_css(archivo.read_text(encoding="utf-8"), activos)
        html = html.replace(m.group(0), f"<style>{css}</style>")

    # El JavaScript.
    for m in re.finditer(r'<script[^>]+src="([^"]+\.js)"[^>]*></script>', html):
        archivo = activos / Path(m.group(1)).name
        js = archivo.read_text(encoding="utf-8")
        html = html.replace(m.group(0), f"<script type=\"module\">{js}</script>")

    # Cualquier imagen que haya quedado suelta en el HTML.
    def imagen(m: re.Match[str]) -> str:
        ref = m.group(2)
        archivo = activos / Path(ref).name
        if not archivo.exists():
            return m.group(0)
        tipo = mimetypes.guess_type(archivo.name)[0] or "application/octet-stream"
        datos = base64.b64encode(archivo.read_bytes()).decode("ascii")
        return f'{m.group(1)}="data:{tipo};base64,{datos}"'

    html = re.sub(r'(src|href)="(/assets/[^"]+)"', imagen, html)

    # ── La CSP ──
    #
    # El producto declara `default-src 'self'`, que **prohíbe el script en
    # línea**. Es un control real y hace exactamente su trabajo: la primera
    # versión de esta vista previa salía en blanco porque el navegador se negó a
    # ejecutar nada, y con razón.
    #
    # Aquí todo va en línea —es el punto de tener un solo archivo—, así que la
    # CSP se relaja **en esta copia y sólo aquí**. Se sustituye por una
    # explícita en vez de borrarla: sin ninguna, el archivo podría cargar
    # cualquier cosa de internet, y esto se abre desde el disco.
    #
    # Es, además, la razón más clara de por qué este archivo NO es el producto.
    html = re.sub(
        r'<meta\s+http-equiv="Content-Security-Policy"[^>]*>',
        '<meta http-equiv="Content-Security-Policy" content="'
        "default-src 'none'; "
        "script-src 'unsafe-inline'; "
        "style-src 'unsafe-inline'; "
        "img-src data:; "
        "font-src data:"
        '">',
        html,
        flags=re.S,
    )

    # El icono de la pestaña queda como una referencia a un archivo que no
    # viaja. No se incrusta: un favicon ausente es un detalle del navegador, y
    # dejar el enlace roto produce un error en la consola que confunde a quien
    # abra las herramientas de desarrollo buscando otra cosa.
    html = re.sub(r'<link[^>]+rel="icon"[^>]*>', "", html)

    # El núcleo simulado va **antes** del módulo de la aplicación: `hayNucleo()`
    # se consulta en el primer render, y si llegara después la pantalla ya
    # habría decidido que no hay núcleo.
    html = html.replace("</head>", NUCLEO_SIMULADO + "</head>", 1)

    # Un aviso dentro del propio archivo. Sin él, alguien que lo reciba
    # reenviado no tiene forma de saber que no es la aplicación.
    html = html.replace(
        "<title>",
        "<!-- VISTA PREVIA de ARLES RELAY I. No es la aplicación de escritorio:\n"
        "     no hay base de datos cifrada ni llavero del sistema, y nada de lo\n"
        "     que se escriba aquí se guarda de verdad. Generado con\n"
        "     herramientas/vista-previa/generar.py -->\n<title>",
        1,
    )

    SALIDA.parent.mkdir(parents=True, exist_ok=True)
    SALIDA.write_text(html, encoding="utf-8")
    shutil.rmtree(DIST, ignore_errors=True)

    sueltos = len(re.findall(r'(?:src|href)="(?:\./)?assets/', html))
    if sueltos:
        raise SystemExit(
            f"quedaron {sueltos} referencias a archivos de al lado: el HTML no "
            "es autocontenido y al abrirlo faltarían piezas."
        )

    kb = SALIDA.stat().st_size / 1024
    print(f"\n✓ {SALIDA.relative_to(RAIZ)}  ·  {kb:.0f} kB  ·  un solo archivo")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
