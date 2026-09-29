/**
 * El núcleo simulado de la vista previa.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * QUÉ ES Y POR QUÉ EXISTE
 *
 * ARLES es una aplicación de escritorio: su núcleo en Rust vive detrás de la
 * frontera de Tauri. Abierta en un navegador no hay nadie al otro lado, así
 * que este archivo pone un `__TAURI_INTERNALS__` falso con un `invoke` que
 * contesta lo que contestaría el núcleo.
 *
 * Es lo que hace que la vista previa —y el enlace público de demostración—
 * enseñen una aplicación con datos en vez de una sucesión de pantallas vacías.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ ESTÁ EN SU PROPIO ARCHIVO DESDE EL 29/09/2026
 *
 * Estaba dentro de `generar.py`, como diecinueve mil caracteres de JavaScript
 * metidos en una cadena de Python. Funcionaba, y era casi imposible de tocar:
 * sin resaltado de sintaxis, sin lint, y con las barras invertidas escapadas
 * dos veces. Añadir un comando exigía contar comillas.
 *
 * Se sacó al preparar la demostración, porque faltaban **once comandos** de
 * las Fases 5 y 6 y del canal de WhatsApp, y eso hacía que cinco pantallas
 * recibieran a quien entraba con «Ocurrió un error, cierra ARLES».
 *
 * ─────────────────────────────────────────────────────────────────────────
 * CÓMO SE AÑADE UN COMANDO
 *
 * Un `case` más en el `switch` de `invoke`, devolviendo una promesa con la
 * misma forma que el tipo de Rust. Si el comando escribe algo, se guarda en
 * `localStorage` con `escribir()` para que el cambio sobreviva a navegar entre
 * pantallas — pero no a recargar con la tecla de recarga, y eso es deliberado:
 * una demostración empieza siempre igual.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE ESTE ARCHIVO NO ES
 *
 * No es el núcleo. No valida como Rust, no cifra nada, no manda ningún correo
 * y no habla con Meta. Un comportamiento que sólo se haya visto aquí **no está
 * verificado**: lo que vale es la prueba en Rust.
 */
(function () {
  var K = 'arles-vista-previa';
  /**
   * El estado de la demostración.
   *
   * Si no hay nada guardado todavía, **se sirve el decorado del final del
   * archivo**, y no un objeto vacío. Es un solo sitio y por eso está aquí: los
   * veintitantos `case` de abajo leen `e.contactos`, `e.empresa`… y así todos
   * arrancan con datos sin que ninguno tenga que saberlo.
   *
   * Con el objeto vacío, la demostración se contaba dos historias distintas:
   * Inicio decía «0 de 6» y Contactos «0 de 0» mientras Campañas enseñaba
   * 1 342 destinatarios.
   */
  function leer() {
    try {
      var guardado = sessionStorage.getItem(K);
      if (guardado) { return JSON.parse(guardado); }
    } catch (e) { /* sessionStorage bloqueado: se sigue con el decorado */ }
    return {
      empresa: DEMO.empresa,
      contactos: DEMO.contactos,
    };
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
            // ── EL CAMPO QUE FALTABA, Y LO QUE COSTÓ ──────────────────────
            //
            // `EstadoDeArranque` de Rust lo tiene desde la 4.3b y aquí no
            // estaba, así que llegaba `undefined` y la pantalla de Inicio —la
            // PRIMERA que se ve— recibía a cualquiera con «Esta ventana no
            // está enviando: hay otro ARLES abierto». En la demostración no
            // hay ninguna otra ventana, ni motor, ni nada.
            //
            // El aviso está bien escrito y hace bien en salir cuando toca; lo
            // que estaba mal era el dato. Encontrado el 29/09/2026 mirando la
            // demostración pantalla por pantalla, no midiéndola: la sonda leía
            // el texto y lo daba por bueno porque es un aviso legítimo.
            elMotorEnvia: true,
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
          } else if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(correo)) {
            malos.push({ campo: 'correoCorporativo', clave: 'empresa.error.correoInvalido' });
          }
          var sitio = String(b.sitioWeb || '').trim();
          if (sitio && !/^https?:\/\//.test(sitio)) {
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

        // ── Supresión (entrega 3.4) ──────────────────────────────────
        //
        // La semilla trae las cuatro situaciones que la pantalla tiene que
        // saber pintar distinto, porque si todas las filas se parecen no se
        // puede juzgar si la pantalla las distingue:
        //
        //   - una baja que pidió la persona (NO se puede deshacer)
        //   - una queja               (tampoco)
        //   - un rebote               (sí se puede)
        //   - un número inválido de WhatsApp, y una baja global
        //
        // `sePuedeLevantar` viene RESUELTO, igual que en Rust: si el simulador
        // lo calculara a su manera, la sonda mediría una regla que no es la
        // del producto.
        case 'listar_supresiones': {
          if (!e.supresiones) {
            e.supresiones = [
              {
                id: 's1', canal: 'email', direccion: 'ana@empresa.mx',
                motivo: 'supresion.motivo.baja', sePuedeLevantar: false,
                deQuien: 'Ana Ruiz', notas: 'Lo pidió por correo',
                creada: '2026-08-14T10:00:00Z', global: false,
              },
              {
                id: 's2', canal: 'email', direccion: 'beto@otra.mx',
                motivo: 'supresion.motivo.queja', sePuedeLevantar: false,
                deQuien: '', notas: '',
                creada: '2026-08-02T09:30:00Z', global: false,
              },
              {
                id: 's3', canal: 'email', direccion: 'rebote@empresa.mx',
                motivo: 'supresion.motivo.rebotePermanente', sePuedeLevantar: true,
                deQuien: 'Carla Méndez', notas: '',
                creada: '2026-07-21T16:05:00Z', global: false,
              },
              {
                id: 's4', canal: 'whatsapp', direccion: '+528112345678',
                motivo: 'supresion.motivo.numeroInvalido', sePuedeLevantar: true,
                deQuien: '', notas: 'No tiene WhatsApp',
                creada: '2026-07-10T12:00:00Z', global: false,
              },
              {
                id: 's5', canal: 'whatsapp', direccion: '+528198765432',
                motivo: 'supresion.motivo.baja', sePuedeLevantar: false,
                deQuien: 'Diego Salas', notas: '',
                creada: '2026-06-30T08:15:00Z', global: true,
              },
            ];
            escribir(e);
          }
          return Promise.resolve({
            supresiones: e.supresiones,
            total: e.supresiones.length,
          });
        }

        case 'levantar_supresion': {
          var fila = (e.supresiones || []).filter(function (s) {
            return s.id === args.id;
          })[0];
          if (!fila) {
            return Promise.reject({ clave: 'error.db.supresion_no_existe', detalle: '' });
          }
          // La regla también aquí: pedirlo sobre una baja que pidió la persona
          // tiene que fallar, aunque la pantalla esconda el botón. Es lo que
          // permite a la sonda comprobar que la defensa no es sólo visual.
          if (!fila.sePuedeLevantar) {
            return Promise.reject({
              clave: 'error.db.supresion_no_se_levanta', detalle: '',
            });
          }
          e.supresiones = e.supresiones.filter(function (s) { return s.id !== args.id; });
          escribir(e);
          return Promise.resolve();
        }

        // El informe de Acceso. Simula los TRES desenlaces distintos, porque
        // la pantalla enseña un texto distinto en cada uno y la sonda tiene
        // que poder llegar a los tres:
        //
        //   - una dirección de la semilla de bajas → hay rastro, pero ya no
        //     es contacto;
        //   - `nadie@…`                            → no consta nada;
        //   - `cancela@…`                          → quien atiende cerró el
        //     diálogo sin guardar (el comando real devuelve `null`).
        //
        // NO se abre ningún diálogo aquí: en la aplicación real lo abre Rust,
        // y la webview nunca ve una ruta. Devolver sólo el NOMBRE del archivo
        // es parte de lo que se está simulando.
        case 'entregar_informe_de_acceso': {
          var d = String(args.direccion || '').trim().toLowerCase();
          if (d.indexOf('cancela') === 0) return Promise.resolve(null);
          var limpio = d.replace(/[^a-zA-Z0-9._-]/g, '-');
          // La lista va ESCRITA AQUÍ y no se lee de `e.supresiones`.
          //
          // La primera versión miraba las bajas guardadas, y eso hacía que el
          // caso «ya no es contacto pero queda rastro» dependiera de haber
          // visitado antes la pantalla de Bajas —que es quien las siembra—.
          // La sonda entra directa a Derechos, así que medía el caso
          // equivocado y fallaba por una razón que no era del producto.
          //
          // Un simulador cuyo resultado depende de por dónde hayas pasado no
          // simula nada: hace que las sondas se contagien entre sí.
          var soloRastro = ['beto@otra.mx', 'rebote@empresa.mx'];
          var enBajas = soloRastro.indexOf(d) >= 0;
          var vacio = d.indexOf('nadie') === 0;
          return Promise.resolve({
            nombreDelArchivo: 'datos-de-' + limpio + '.txt',
            habiaDatos: !vacio,
            sigueSiendoContacto: !vacio && !enBajas,
          });
        }

        // El borrado, en sus dos pasos. `que_se_borraria` NO borra: sólo
        // cuenta, y es lo que la pantalla enseña antes de dejar pulsar.
        //
        // Se simulan los dos desenlaces que cambian el texto:
        //   - una dirección conocida → hay datos, y baja según la semilla
        //   - `nadie@…`              → no consta nada, y se dice
        case 'que_se_borraria': {
          var dv = String(args.direccion || '').trim().toLowerCase();
          var vacia = dv.indexOf('nadie') === 0;
          var conBaja = ['ana@empresa.mx', 'beto@otra.mx'].indexOf(dv) >= 0;
          return Promise.resolve({
            direccion: dv,
            nombre: vacia ? '' : 'Ana Ruiz',
            sigueSiendoContacto: !vacia && !conBaja,
            direcciones: vacia ? 0 : 2,
            campos: vacia ? 0 : 3,
            pertenencias: vacia ? 0 : 1,
            envios: vacia ? 0 : 4,
            tieneBaja: conBaja,
          });
        }

        case 'cancelar_datos_de': {
          var dc = String(args.direccion || '').trim().toLowerCase();
          var vacio2 = dc.indexOf('nadie') === 0;
          var tenia = ['ana@empresa.mx', 'beto@otra.mx'].indexOf(dc) >= 0;
          return Promise.resolve({
            contactosBorrados: vacio2 ? 0 : 1,
            canalesBorrados: vacio2 ? 0 : 2,
            camposBorrados: vacio2 ? 0 : 3,
            pertenenciasBorradas: vacio2 ? 0 : 1,
            enviosAnonimizados: vacio2 ? 0 : 4,
            audienciasAnonimizadas: vacio2 ? 0 : 1,
            laBajaYaExistia: tenia,
          });
        }

        case 'suprimir_contacto':
          return Promise.resolve({ nuevas: 1, yaEstaban: 0 });

        case 'abrir_sitio_de_telemetry':
          window.open('https://telemetrymx.com', '_blank', 'noopener');
          return Promise.resolve();

        // ── Fases 5 y 6, y el canal de WhatsApp ──────────────────────
        //
        // Faltaban los once hasta el 29/09/2026, y sin ellos el `default` de
        // abajo devolvía un error de base de datos: cinco pantallas recibían a
        // quien entraba con «Ocurrió un error, cierra ARLES y vuelve a
        // abrirlo». Lo encontró `pruebas/sondas/recorrido.mjs`.
        //
        // Los datos son inventados y se nota: todo lleva `ejemplo.mx`.

        case 'remitentes':
          return Promise.resolve(e.remitentes || DEMO.remitentes);

        case 'probar_remitente':
          return Promise.resolve({
            diagnostico: 'correo.diagnostico.todoBien', sirve: true,
            remitentes: e.remitentes || DEMO.remitentes,
          });

        case 'guardar_remitente': {
          var rs = (e.remitentes || DEMO.remitentes).slice();
          rs.push(Object.assign({}, DEMO.remitentes[0], {
            id: 'r' + Math.random().toString(36).slice(2, 10),
            servidor: String((args.borrador || {}).servidor || 'smtp.ejemplo.mx'),
            usuario: String((args.borrador || {}).usuario || ''),
            remitente: String((args.borrador || {}).remitente || ''),
          }));
          e.remitentes = rs; escribir(e);
          return Promise.resolve({
            diagnostico: 'correo.diagnostico.todoBien', sirve: true, remitentes: rs,
          });
        }

        // §65: se dice «aceptado», nunca «entregado». ARLES no puede saber si
        // llegó a la bandeja, y la demostración no va a decir lo contrario.
        case 'enviar_correo_de_prueba':
          return Promise.resolve({
            aceptado: true, aDonde: String(args.para || 'tu@ejemplo.mx'),
          });

        case 'campanas':
          return Promise.resolve(e.campanas || DEMO.campanas);

        case 'plantillas':
          return Promise.resolve(e.plantillas || DEMO.plantillas);

        case 'guardar_plantilla': {
          var ps = (e.plantillas || DEMO.plantillas).slice();
          ps.push({
            id: 'p' + Math.random().toString(36).slice(2, 10),
            nombre: String(args.nombre || 'Sin nombre'),
            version: {
              id: 'v' + Math.random().toString(36).slice(2, 10),
              numero: 1,
              asunto: String(args.asunto || ''),
              cuerpoHtml: String(args.cuerpoHtml || ''),
              variables: variablesDe(String(args.asunto || '') + ' ' + String(args.cuerpoHtml || '')),
            },
          });
          e.plantillas = ps; escribir(e);
          return Promise.resolve(ps);
        }

        case 'firmas':
          return Promise.resolve(e.firmas || DEMO.firmas);

        case 'guardar_firma': {
          var fs = (e.firmas || DEMO.firmas).slice();
          if (args.porDefecto) {
            fs = fs.map(function (f) { return Object.assign({}, f, { porDefecto: false }); });
          }
          fs.push({
            id: 'f' + Math.random().toString(36).slice(2, 10),
            nombre: String(args.nombre || 'Sin nombre'),
            cuerpoHtml: String(args.cuerpoHtml || ''),
            porDefecto: !!args.porDefecto,
          });
          e.firmas = fs; escribir(e);
          return Promise.resolve(fs);
        }

        case 'ventanas':
          return Promise.resolve(e.ventanas || DEMO.ventanas);

        case 'numeros_de_whatsapp':
          return Promise.resolve(e.whatsapp || DEMO.whatsapp);

        // Las dos marcas se ponen de una en una, cada una con su pulsación.
        // Aquí se guardan con su fecha para que la pantalla enseñe el «desde
        // cuándo», que es la mitad de lo que hace útil una constancia.
        case 'confirmar_coexistencia':
        case 'aceptar_riesgo_del_canal': {
          var ns = (e.whatsapp || DEMO.whatsapp).map(function (n) {
            if (n.id !== args.id) return n;
            var m = Object.assign({}, n);
            if (cmd === 'confirmar_coexistencia') m.coexistenciaConfirmada = ahora();
            else m.riesgoAceptado = ahora();
            m.puedeEnviar = !!m.coexistenciaConfirmada &&
                            (!m.escribeEnFrio || !!m.riesgoAceptado);
            return m;
          });
          e.whatsapp = ns; escribir(e);
          return Promise.resolve(ns);
        }

        default:
          return Promise.reject({ clave: 'error.db.sqlite', detalle: 'desconocido: ' + cmd });
      }
    },
  };

  function configuracion(e) {
    var hay = !!e.empresa;
    var pasos = [
      // La ruta la manda el núcleo (`PasoDeOnboarding::ruta`). Aquí estuvo
      // escrita '/ajustes' hasta el 22/09/2026, cuatro días después de que el
      // núcleo la cambiara: la aplicación real llevaba bien, y la vista previa
      // —que es DONDE DIRECCIÓN REVISA— seguía llevando a Ajustes. Se reportó
      // dos veces el mismo defecto porque el simulador mentía.
      //
      // `comprobar_simulador.py` compara esta tabla con el núcleo y falla si
      // vuelven a separarse.
      // ── LOS SEIS PASOS SE CALCULAN, NO SE FIJAN ─────────────────────────
      //
      // Los cinco de abajo estaban escritos como `false` sin mirar nada, y era
      // correcto cuando el simulador sólo sabía de la empresa. Con remitentes,
      // contactos, plantillas y campañas en el decorado, Inicio decía
      // «Contactos · aún no» con tres contactos dados de alta y «1 / 6» con
      // cinco pasos hechos.
      //
      // Encontrado el 29/09/2026 mirando la demostración, no midiéndola: el
      // texto era legítimo y ninguna sonda podía saber que era falso.
      ['empresa', hay, true, '3.1', '/inicio/configuracion'],
      // Contactos y Remitentes ganaron ruta el 23/09/2026, al descubrirse que
      // Contactos llevaba desde la 3.2 anunciándose como «llega en la entrega
      // 3.2» con la pantalla ya construida. La comprobación de abajo volvió a
      // hacer su trabajo: avisó de que el simulador se había quedado atrás.
      ['remitente', (e.remitentes || DEMO.remitentes).length > 0, true, '5', '/remitentes'],
      ['contactos', (e.contactos || []).length > 0, true, '3.2', '/contactos'],
      ['plantilla', (e.plantillas || DEMO.plantillas).length > 0, true, '6', '/campanas/plantillas'],
      ['ventana', (e.ventanas || DEMO.ventanas).length > 0, false, '6', null],
      ['campana', (e.campanas || DEMO.campanas).length > 0, true, '6', '/campanas'],
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
        completados: pasos.filter(function (p) { return p.completado; }).length,
        total: 6,
        // El primero que falta, y no una suposición. Con todo hecho no hay
        // siguiente, y la pantalla sabe leer ese `null`.
        siguiente: (pasos.filter(function (p) { return !p.completado; })[0] || {}).paso || null,
      },
    };
  }
  /**
   * El decorado: lo que se enseña cuando todavía no se ha tocado nada.
   *
   * Está al final y en un solo objeto **para que se pueda cambiar entero sin
   * leer el resto del archivo**. Todo lleva `ejemplo.mx` a propósito: en una
   * demostración tiene que notarse que los datos son inventados.
   *
   * El número de WhatsApp nace **sin sus dos confirmaciones**, para que la
   * demostración enseñe los dos avisos — son una de las decisiones de producto
   * que más vale la pena ver, y un número ya confirmado las escondería.
   */
  var DEMO = {
    // La empresa ya configurada.
    //
    // El simulador arrancaba en blanco a propósito, «para que lo primero que
    // se vea sea la portada de primera vez». Tenía sentido cuando no había
    // más datos; con campañas de 1 342 destinatarios y un remitente dado de
    // alta, dejaba la demostración contándose a sí misma dos historias
    // distintas: Inicio decía «0 de 6» y Contactos «0 de 0» mientras Campañas
    // enseñaba mil trescientos.
    //
    // La portada de primera vez se sigue viendo: basta con el botón de
    // reiniciar de la demostración, o entrando en Configuración.
    empresa: {
      nombreComercial: 'Distribuidora del Norte',
      pais: 'MX',
      zonaHoraria: 'America/Monterrey',
      correoCorporativo: 'contacto@ejemplo.mx',
      sitioWeb: 'https://ejemplo.mx',
    },

    contactos: [
      {
        id: 'c-demo-1', nombre: 'Ana', apellido: 'Gutiérrez',
        empresa: 'Ferretería Elizondo', creadoEn: '2026-09-02T10:00:00Z',
        actualizadoEn: '2026-09-02T10:00:00Z',
        canales: [
          { canal: 'email', valorRaw: 'ana@ejemplo.mx', valorNormalizado: 'ana@ejemplo.mx', principal: true },
          { canal: 'whatsapp', valorRaw: '+52 81 1111 1111', valorNormalizado: '+528111111111', principal: true },
        ],
      },
      {
        id: 'c-demo-2', nombre: 'Luis', apellido: 'Mendoza',
        empresa: 'Aceros del Valle', creadoEn: '2026-09-03T10:00:00Z',
        actualizadoEn: '2026-09-03T10:00:00Z',
        canales: [
          { canal: 'email', valorRaw: 'luis@ejemplo.mx', valorNormalizado: 'luis@ejemplo.mx', principal: true },
        ],
      },
      {
        id: 'c-demo-3', nombre: 'Carmen', apellido: 'Villarreal',
        empresa: 'Plásticos del Golfo', creadoEn: '2026-09-04T10:00:00Z',
        actualizadoEn: '2026-09-04T10:00:00Z',
        canales: [
          { canal: 'email', valorRaw: 'carmen@ejemplo.mx', valorNormalizado: 'carmen@ejemplo.mx', principal: true },
          { canal: 'whatsapp', valorRaw: '+52 81 3333 3333', valorNormalizado: '+528133333333', principal: true },
        ],
      },
    ],

    remitentes: [{
      id: '01900000-0000-7000-8000-00000000000a',
      servidor: 'smtp.gmail.com', puerto: 587, cifrado: 'starttls',
      usuario: 'ventas@ejemplo.mx', remitente: 'ventas@ejemplo.mx',
      nombreVisible: 'Ventas · Distribuidora del Norte',
      estado: 'connected', limiteDiario: 50, limitePorHora: 10,
    }],

    campanas: [
      {
        id: '01900000-0000-7000-8000-0000000000c1',
        // `CampanaIpc.estado` **no lleva el estado crudo de la base, sino la
        // clave del rótulo** (§139): lo dice su comentario en `campanas.rs`, y
        // la pantalla la pasa tal cual a `t()`.
        //
        // El decorado puso primero `completada` y luego `completed`, y las dos
        // veces la pantalla enseñó «Estado no reconocido» — que es lo que hace
        // cuando la clave no existe, y está bien que lo haga. Encontrado el
        // 29/09/2026 recorriendo la demostración pantalla por pantalla.
        nombre: 'Bienvenida · septiembre', estado: 'campana.estado.terminada',
        audiencia: 1342, aceptados: 1342,
      },
      {
        id: '01900000-0000-7000-8000-0000000000c2',
        nombre: 'Seguimiento a cotizaciones', estado: 'campana.estado.borrador',
        audiencia: 0, aceptados: 0,
      },
    ],

    plantillas: [
      {
        id: '01900000-0000-7000-8000-00000000000b', nombre: 'Bienvenida',
        version: {
          id: '01900000-0000-7000-8000-00000000000c', numero: 2,
          asunto: 'Hola {{nombre}}, gracias por escribirnos',
          cuerpoHtml: '<p>Hola {{nombre}},</p><p>Gracias por tu interés en {{mi_empresa}}.</p>',
          variables: ['nombre', 'mi_empresa'],
        },
      },
      {
        id: '01900000-0000-7000-8000-00000000000d', nombre: 'Seguimiento a cotización',
        version: {
          id: '01900000-0000-7000-8000-00000000000e', numero: 1,
          asunto: 'Sobre tu cotización, {{nombre}}',
          cuerpoHtml: '<p>Hola {{nombre}}, ¿pudiste revisar la propuesta?</p>',
          variables: ['nombre'],
        },
      },
    ],

    firmas: [{
      id: '01900000-0000-7000-8000-00000000000f', nombre: 'Firma comercial',
      cuerpoHtml: '<p>Equipo de Ventas</p>', porDefecto: true,
    }],

    ventanas: [{
      id: '01900000-0000-7000-8000-0000000000d1', nombre: 'Días laborables',
      dias: 31, desde: 9, hasta: 18, zona: 'America/Monterrey',
    }],

    whatsapp: [{
      id: '01900000-0000-7000-8000-0000000000a1',
      nombreVisible: 'Atención a clientes', telefonoE164: '+528100000000',
      wabaId: '102938475610293', modo: 'prospeccion_directa', estado: 'connected',
      coexistenciaConfirmada: null, riesgoAceptado: null,
      calidad: null, limiteDeMeta: 250,
      puedeEnviar: false, escribeEnFrio: true,
    }],
  };

  /** Las variables que usa un texto, con el mismo patrón que el núcleo. */
  function variablesDe(texto) {
    var vistas = {};
    var re = /\{\{\s*(\w+)\s*\}\}/g;
    var m;
    while ((m = re.exec(texto)) !== null) { vistas[m[1]] = true; }
    return Object.keys(vistas);
  }
})();
