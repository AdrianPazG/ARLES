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

  /** Un identificador de mentira, con la misma pinta que los del núcleo.
   *
   * No es un UUIDv7 de verdad y no hace falta: lo único que la pantalla hace
   * con un id es volver a pasarlo por la frontera. Lo que sí importa es que sea
   * distinto cada vez, porque dos campañas con el mismo id en la demostración
   * hacen que el freno pause la equivocada. */
  function nuevoId() { return 'x' + Math.random().toString(36).slice(2, 10); }

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

        // ── EL FRENO (7.0) ────────────────────────────────────────────────
        //
        // Los seis comandos de control. Se responden aquí porque, si no, la
        // demostración enseñaría un error justo al pulsar el botón más
        // importante de la pantalla — que es el defecto que once comandos sin
        // respuesta ya causaron una vez en cinco pantallas.
        case 'pausar_campana':
        case 'reanudar_campana':
        case 'detener_campana': {
          var lista = (e.campanas || DEMO.campanas).slice();
          var i = lista.findIndex(function (c) { return c.id === (args.id || ''); });
          if (i < 0) {
            return Promise.reject({ clave: 'error.db.campana_no_existe', detalle: '' });
          }

          // El nombre se comprueba igual que en Rust: sin espacios de sobra y
          // **sin ignorar mayúsculas**. Si el simulador fuera más blando que
          // el núcleo, la demostración enseñaría un control que el producto
          // real rechaza.
          if (cmd === 'detener_campana') {
            var escrito = (args.nombreEscrito || '').trim();
            if (escrito !== lista[i].nombre.trim()) {
              return Promise.reject({ clave: 'error.app.nombre_no_coincide', detalle: '' });
            }
          }

          var destino = cmd === 'pausar_campana' ? 'campana.estado.enPausa'
            : cmd === 'reanudar_campana' ? 'campana.estado.enviando'
            : 'campana.estado.detenida';
          var cancelados = cmd === 'detener_campana'
            ? Math.max(0, lista[i].audiencia - lista[i].aceptados)
            : 0;

          lista[i] = Object.assign({}, lista[i], { estado: destino });
          e.campanas = lista;
          escribir(e);
          return Promise.resolve({ estado: destino, cancelados: cancelados, etapas: 1 });
        }

        case 'parar_todo': {
          e.parada = { desde: ahora(), por: args.quien || null };
          escribir(e);
          return Promise.resolve({
            parado: true, desde: e.parada.desde, por: e.parada.por,
            clave: 'campana.parada.puesta',
          });
        }

        case 'levantar_parada': {
          e.parada = null;
          escribir(e);
          return Promise.resolve({
            parado: false, desde: null, por: null, clave: 'campana.parada.levantada',
          });
        }

        case 'estado_del_envio':
          return Promise.resolve(e.parada
            ? {
              parado: true, desde: e.parada.desde, por: e.parada.por,
              clave: 'campana.parada.puesta',
            }
            : { parado: false, desde: null, por: null, clave: 'campana.parada.levantada' });

        // ── ACTIVIDAD (7.3) ───────────────────────────────────────────────
        //
        // **Se DERIVA de las campañas**, no es una segunda lista. Si fuera un
        // decorado suelto, pausar una campaña en la pantalla de Campañas la
        // dejaría «enviando» en Actividad, y la demostración enseñaría dos
        // pantallas contradiciéndose sobre el mismo envío — que es exactamente
        // lo que el comando grueso de la 7.2 existe para impedir.
        case 'actividad': {
          var cs = (e.campanas || DEMO.campanas).map(function (c) {
            var extra = DEMO.actividad[c.id] || {};
            var pendiente = Math.max(0, c.audiencia - c.aceptados - (extra.fallo || 0));
            return {
              id: c.id, nombre: c.nombre, estado: c.estado,
              audiencia: c.audiencia, salio: c.aceptados, pendiente: pendiente,
              fallo: extra.fallo || 0, dadoDeBaja: extra.dadoDeBaja || 0,
              cancelado: extra.cancelado || 0, respondieron: extra.respondieron || 0,
              // El ritmo sólo de las que envían, igual que hace el comando de
              // Rust: una consulta por campaña parada no dice nada que alguien
              // vaya a leer.
              ritmo: c.estado === 'campana.estado.enviando' ? (extra.ritmo || []) : [],
              razon: extra.razon || null,
            };
          });
          return Promise.resolve({
            campanas: cs,
            // La suma de las de arriba, **nunca otra cuenta**: dos números que
            // no cuadran hacen que alguien deje de creerse los dos.
            totalSalio: cs.reduce(function (s, c) { return s + c.salio; }, 0),
            totalPendiente: cs.reduce(function (s, c) { return s + c.pendiente; }, 0),
            totalFallo: cs.reduce(function (s, c) { return s + c.fallo; }, 0),
            remitentes: DEMO.cupos,
          });
        }

        // ── EL REGISTRO Y SU RECORTE (7.4, D-7C) ─────────────────────────
        //
        // El filtro se aplica **aquí de verdad** y no se ignora: un simulador
        // que devuelve siempre lo mismo enseña unos filtros que no hacen nada,
        // y en la demostración eso se lee como que están rotos.
        case 'registro': {
          var f = args.filtro || {};
          var todos = (e.registro || DEMO.registro).slice();
          var filtrados = todos.filter(function (s) {
            if (f.campana && s.campana !== f.campana) return false;
            if (f.clave && s.clave !== f.clave) return false;
            if (f.soloDesenlaces === true && !s.esDesenlace) return false;
            if (f.soloDesenlaces === false && s.esDesenlace) return false;
            if (f.desde && s.cuando < f.desde) return false;
            // Inclusive, igual que el núcleo: quien pide «hasta el martes»
            // quiere el martes entero.
            if (f.hasta && s.cuando.slice(0, 10) > f.hasta) return false;
            if (f.texto) {
              // Sin dirección no puede casar: un suceso del motor no es de
              // nadie. Es la misma regla que el `LEFT JOIN` de la consulta.
              if (!s.direccion) return false;
              if (s.direccion.indexOf(f.texto) < 0) return false;
            }
            return true;
          });

          // Las clases **sin filtrar**, como el comando real: si se calcularan
          // sobre el filtro, elegir una dejaría el desplegable con una sola
          // opción y no habría forma de volver.
          var cuenta = {};
          todos.forEach(function (s) { cuenta[s.clave] = (cuenta[s.clave] || 0) + 1; });
          var clases = Object.keys(cuenta)
            .map(function (k) { return { clave: k, cuantos: cuenta[k] }; })
            .sort(function (a, b) { return b.cuantos - a.cuantos; });

          // Ordenado por fecha descendente, igual que el `ORDER BY` del
          // núcleo. Sin esto la demostración enseña el registro desordenado y
          // parece un fallo del producto, cuando es del decorado.
          filtrados.sort(function (a, b) { return a.cuando < b.cuando ? 1 : -1; });

          return Promise.resolve({
            sucesos: filtrados.slice(0, 500),
            total: filtrados.length,
            porPagina: 500,
            clases: clases,
          });
        }

        // ── EL RESUMEN EXPORTABLE (7.6) ───────────────────────────────────
        //
        // En la demostración **no se guarda ningún archivo**: no hay diálogo
        // del sistema en un navegador, y escribir en disco desde una vista
        // previa sería peor que no hacer nada. Se devuelve el nombre que
        // tendría, que es lo único que la pantalla enseña.
        case 'exportar_resumen': {
          var laCampana = (e.campanas || DEMO.campanas)
            .filter(function (c) { return c.id === (args.campana || ''); })[0];
          if (!laCampana) {
            return Promise.reject({ clave: 'error.db.campana_no_existe', detalle: '' });
          }
          var limpio = laCampana.nombre
            .normalize('NFD').replace(/[\u0300-\u036f]/g, '')
            .replace(/[^a-zA-Z0-9.\-_]/g, '-');
          return Promise.resolve({ nombreDelArchivo: 'resumen-' + limpio + '.txt' });
        }

        case 'que_se_recortaria': {
          var lista = e.registro || DEMO.registro;
          var ruido = lista.filter(function (s) { return !s.esDesenlace && s.viejo; });
          return Promise.resolve({
            seIrian: ruido.length,
            antesDe: '2026-07-31T00:00:00Z',
            seQuedan: lista.length - ruido.length,
          });
        }

        case 'recortar_registro': {
          // Se quedan los envíos **y** el ruido reciente. Lo que se va es sólo
          // el ruido viejo, que es lo que D-7C define.
          e.registro = (e.registro || DEMO.registro).filter(function (s) {
            return s.esDesenlace || !s.viejo;
          });
          escribir(e);
          return Promise.resolve({
            seIrian: 0,
            antesDe: '2026-07-31T00:00:00Z',
            seQuedan: e.registro.length,
          });
        }

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

        // ── R-06 · corregir lo ya escrito ─────────────────────────────────
        //
        // La versión **sube**, y el decorado lo imita porque es lo que la
        // pantalla enseña: un número que no cambiara haría parecer que
        // corregir no hizo nada.
        case 'nueva_version_de_plantilla': {
          var pv = (e.plantillas || DEMO.plantillas).map(function (p) {
            if (p.id !== String(args.plantilla)) return p;
            var num = (p.version ? p.version.numero : 0) + 1;
            return Object.assign({}, p, {
              version: {
                id: 'v' + Math.random().toString(36).slice(2, 10),
                numero: num,
                asunto: String(args.asunto || ''),
                cuerpoHtml: String(args.cuerpoHtml || ''),
                variables: variablesDe(String(args.asunto || '') + ' ' + String(args.cuerpoHtml || '')),
              },
            });
          });
          e.plantillas = pv; escribir(e);
          return Promise.resolve(pv);
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

        // Una firma se reescribe **encima**: no nace otra, y la lista tiene
        // que seguir teniendo las mismas. Es lo que distingue este caso del
        // de la plantilla, y lo que la pantalla promete.
        case 'editar_firma': {
          var fe = (e.firmas || DEMO.firmas).map(function (f) {
            if (args.porDefecto && f.id !== String(args.firma)) {
              return Object.assign({}, f, { porDefecto: false });
            }
            if (f.id !== String(args.firma)) return f;
            return Object.assign({}, f, {
              nombre: String(args.nombre || f.nombre),
              cuerpoHtml: String(args.cuerpoHtml || ''),
              porDefecto: !!args.porDefecto,
            });
          });
          e.firmas = fe; escribir(e);
          return Promise.resolve(fe);
        }

        case 'ventanas':
          return Promise.resolve(e.ventanas || DEMO.ventanas);

        // ── SEIS COMANDOS QUE LA DEMOSTRACIÓN NO CONTESTABA ───────────────
        //
        // Los encontró `todo_comando_que_piden_las_pantallas_lo_contesta_el_simulador`
        // el 30/09/2026, escrita en la 7.3. Sin ellos, el `default` de este
        // `switch` rechaza y la pantalla enseña un error: en la demostración
        // **no se podía crear una campaña, ni activarla, ni revisarla**, ni
        // guardar un horario, ni guardar un número de WhatsApp, ni probar el
        // envío de una cuenta. Todo eso funciona en el producto.
        //
        // Es el mismo defecto que once comandos causaron una vez en cinco
        // pantallas, y volvió porque nada comparaba las dos listas. Ahora sí.
        case 'guardar_ventana': {
          var vs = (e.ventanas || DEMO.ventanas).slice();
          vs.push({
            id: nuevoId(), nombre: String(args.nombre || 'Horario'),
            dias: Number(args.dias || 31), desde: Number(args.desde || 9),
            hasta: Number(args.hasta || 18),
            zona: String(args.zona || 'America/Monterrey'),
          });
          e.ventanas = vs;
          escribir(e);
          return Promise.resolve(vs);
        }

        case 'crear_campana': {
          // El aviso del §48 lo **exige** `crear_campana` en Rust, así que el
          // simulador tiene que exigirlo igual: si aquí fuera más blando, la
          // demostración enseñaría una campaña de alto volumen creada sin que
          // conste el aviso, que es justo lo que el núcleo impide.
          var tope = Number(args.limiteDiario || 0);
          if (tope > 50 && !args.avisoAceptado) {
            return Promise.reject({ clave: 'error.app.falta_aceptar_el_aviso', detalle: '' });
          }
          var nueva = {
            id: nuevoId(),
            nombre: String(args.nombre || 'Campaña nueva'),
            estado: 'campana.estado.borrador',
            audiencia: 0, aceptados: 0,
          };
          e.campanas = (e.campanas || DEMO.campanas).concat([nueva]);
          escribir(e);
          // `etapa` viaja desde Rust porque el envío de prueba cuelga de la
          // etapa y no de la campaña: se devuelve un id propio, no el de la
          // campaña con un sufijo.
          return Promise.resolve({ id: nueva.id, etapa: nuevoId() });
        }

        case 'activar_campana': {
          var cl = (e.campanas || DEMO.campanas).slice();
          var ci = cl.findIndex(function (c) { return c.id === (args.id || ''); });
          if (ci < 0) {
            return Promise.reject({ clave: 'error.db.campana_no_existe', detalle: '' });
          }
          // La audiencia se congela al activar (§57): a partir de aquí, quien
          // entre en la lista de contactos ya no entra en esta campaña.
          var enLa = (e.contactos || DEMO.contactos).length;
          var fuera = 1;
          cl[ci] = Object.assign({}, cl[ci], {
            estado: 'campana.estado.enviando',
            audiencia: Math.max(0, enLa - fuera),
          });
          e.campanas = cl;
          escribir(e);
          return Promise.resolve({ enLaAudiencia: Math.max(0, enLa - fuera), fueraPorBaja: fuera });
        }

        case 'revisar_campana':
          // El preflight de ocho puntos, en verde menos los tres de red, que
          // desde un navegador **no se pueden contestar**: van como «sin
          // comprobar» y no como aprobados, que es lo que hace el producto.
          return Promise.resolve({
            preflight: {
              puntos: [
                { punto: 'hayRemitente', veredicto: 'pasa' },
                { punto: 'laCuentaAutentica', veredicto: 'sinComprobar' },
                { punto: 'hayConexion', veredicto: 'sinComprobar' },
                { punto: 'registrosDelDominio', veredicto: 'sinComprobar' },
                { punto: 'hayPlantilla', veredicto: 'pasa' },
                { punto: 'hayAudiencia', veredicto: 'pasa' },
                { punto: 'hayHorario', veredicto: 'pasa' },
                { punto: 'seEnvioUnaPrueba', veredicto: 'avisa' },
              ],
              resultado: { resultado: 'preflight.conAvisos', avisos: 1, sinComprobar: 3 },
            },
            audiencia: (e.contactos || DEMO.contactos).length,
            contactosConHuecos: 0,
            // ── ESTA FORMA NO EXISTÍA, Y EL PASO 8 ENSEÑABA HUECOS ──
            //
            // Hasta la 8.6 E esto devolvía `{ horas, dias, porDia }`, que no
            // coincide con un solo campo de `Estimacion`: lo que manda Rust es
            // `techoDiario`, `diasConEnvio`, `audiencia`, `quienManda` y el
            // `desenlace` etiquetado. Así que en la demostración el paso de la
            // simulación llevaba tiempo pintando valores vacíos.
            //
            // La comprobación de la fase 7 —«la vista previa contesta lo que
            // las pantallas piden»— no lo cazaba y no podía: compara **qué
            // comandos** se piden y se contestan, no la **forma** de lo que
            // devuelven. Es el mismo tipo de hueco que el del tipo de
            // TypeScript que mentía, y por la misma razón: nadie lo leía.
            //
            // Se simula el caso interesante —una audiencia que no termina— a
            // propósito: es el que la 8.6 E vino a enseñar, y en la
            // demostración hay que poder verlo.
            estimacion: {
              desenlace: { desenlace: 'noCabeEnElHorizonte', faltan: 386046 },
              audiencia: (e.contactos || DEMO.contactos).length,
              techoDiario: 50,
              quienManda: 'elTopeDelDia',
              diasConEnvio: 1303,
            },
            diagnostico: null,
          });

        case 'probar_envio_de_la_cuenta':
          // Aceptado, **que no es entregado** (§65). El simulador no manda nada.
          return Promise.resolve({
            aceptado: true, problema: null,
            aDonde: String(args.para || 'tu@ejemplo.mx'),
          });

        case 'numeros_de_whatsapp':
          return Promise.resolve(e.whatsapp || DEMO.whatsapp);

        case 'guardar_numero_de_whatsapp': {
          // **Sin el token**, igual que el comando real: la credencial entra y
          // no sale (§29 y ADR-0011). Lo que se guarda aquí es lo que la
          // pantalla puede volver a leer, y el token no está en esa lista.
          var ns = [{
            id: nuevoId(),
            nombreVisible: String(args.nombreVisible || ''),
            telefonoE164: String(args.telefono || ''),
            wabaId: String(args.wabaId || ''),
            modo: String(args.modo || 'prospeccion_directa'),
            estado: 'connected',
            coexistenciaConfirmada: null, riesgoAceptado: null,
            calidad: null, limiteDeMeta: 250,
            puedeEnviar: false, escribeEnFrio: args.modo !== 'coexistencia',
          }];
          e.whatsapp = ns;
          escribir(e);
          return Promise.resolve(ns);
        }

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

        // El envío de prueba de WhatsApp (el botón que la 7.7 añadió).
        //
        // En la demostración **no sale ningún mensaje**: se contesta un
        // desenlace, igual que hace el del correo. Quien revisa tiene que poder
        // ver las dos caras, así que un número que acaba en 0 se rechaza — es
        // la única forma de mirar a ojo el aviso de que no salió, que es el
        // texto que más importa de los dos.
        case 'enviar_whatsapp_de_prueba': {
          var para = String(args.para || '').replace(/\D/g, '');
          if (para.length < 10) {
            return Promise.resolve({
              aceptado: false,
              clave: 'whatsapp.prueba.numeroNoExiste',
            });
          }
          return Promise.resolve(
            para.slice(-1) === '0'
              ? { aceptado: false, clave: 'whatsapp.prueba.tokenNoVale' }
              : { aceptado: true, clave: 'whatsapp.prueba.aceptado' },
          );
        }

        // ── El respaldo (8.2c) ────────────────────────────────────────────
        //
        // El preflight sí se contesta con cifras: es lo que hay que poder mirar
        // a ojo, porque elegir entre «Lo esencial» y «Todo» sin ver la
        // diferencia es elegir a ciegas (§94).
        case 'que_llevaria_el_respaldo': {
          var todo = args.alcance === 'completo';
          var tablas = [
            { tabla: 'contact', filas: (e.contactos || []).length || 1342 },
            { tabla: 'contact_channel', filas: ((e.contactos || []).length || 1342) * 2 },
            { tabla: 'suppression_entry', filas: 37 },
            { tabla: 'campaign', filas: (e.campanas || []).length || 3 },
            { tabla: 'template', filas: 4 },
            { tabla: 'message_attempt', filas: todo ? 48207 : 12483 },
          ];
          if (todo) {
            tablas.push({ tabla: 'audit_log', filas: 915 });
            tablas.push({ tabla: 'activity_event', filas: 6104 });
          }
          var total = 0;
          for (var i = 0; i < tablas.length; i += 1) { total += tablas[i].filas; }
          return Promise.resolve({
            esquema: 'V10',
            tablas: tablas,
            total: total,
            mesesEnElEsencial: 12,
          });
        }

        // **Devuelve `null`, que significa «cerró el diálogo sin elegir
        // sitio».** Y es la única respuesta honesta: aquí no hay disco donde
        // escribir ni diálogo del sistema que abrir, y contestar «respaldo
        // creado» enseñaría un éxito de algo que no ocurrió — que es
        // exactamente lo que este archivo dice que no es.
        case 'exportar_respaldo':
          return Promise.resolve(null);

        // ── RESTAURAR (8.3c) ────────────────────────────────────────────
        //
        // `revisar_respaldo` SÍ contesta con cifras, y no es una excepción a
        // la regla de este archivo: revisar es una **lectura**, como la lista
        // de contactos o las cifras de una campaña, y fabricar lecturas es
        // justo para lo que existe el decorado. Lo que no se puede fabricar es
        // el éxito de una escritura.
        //
        // Los números están elegidos para que la pantalla se vea como se verá
        // de verdad en el caso que importa: un respaldo «esencial» que **se
        // lleva por delante el registro de actividad** y no lo devuelve. Si
        // aquí saliera un preflight sin pérdidas, Dirección revisaría una
        // pantalla que nunca enseña su aviso más importante.
        case 'revisar_respaldo':
          return Promise.resolve({
            nombreDelArchivo: 'arles-20260921-esencial.arles',
            creadoEn: '2026-09-21T09:30:00Z',
            alcance: 'esencial',
            esquemaDelRespaldo: 'V11',
            esquemaDeLaBase: 'V11',
            llevaContrasena: true,
            sePuedeRestaurar: true,
            tablas: [
              { tabla: 'company', ahora: 1, enElRespaldo: 1 },
              { tabla: 'contact', ahora: (e.contactos || []).length || 1342, enElRespaldo: 1289 },
              { tabla: 'suppression_entry', ahora: 37, enElRespaldo: 31 },
              { tabla: 'campaign', ahora: (e.campanas || []).length || 3, enElRespaldo: 3 },
              { tabla: 'template', ahora: 4, enElRespaldo: 4 },
              { tabla: 'message_attempt', ahora: 48207, enElRespaldo: 12483 },
            ],
            totalDeFilas: 16294,
            noVuelven: [
              { tabla: 'activity_event', filas: 6104, esDeEsteEquipo: false },
              { tabla: 'whatsapp_quality_event', filas: 42, esDeEsteEquipo: false },
              { tabla: 'rate_budget', filas: 2, esDeEsteEquipo: true },
              { tabla: 'engine_control', filas: 1, esDeEsteEquipo: true },
            ],
            filasQueNoVuelven: 6146,
            bajasAhora: 37,
            bajasEnElRespaldo: 31,
            bajasAlMenos: 37,
            cuentasAReconectar: 2,
          });

        // **No finge haber restaurado.** Y el error que devuelve es verdad,
        // no una excusa cómoda: en la vista previa no hay ningún archivo en
        // disco que abrir. Contestar «restaurado» enseñaría el éxito de la
        // acción más destructiva del producto sin que hubiera pasado nada —
        // que es exactamente lo que este archivo existe para no hacer.
        case 'restaurar_respaldo':
          return Promise.reject({ clave: 'error.app.archivo_no_se_pudo_leer', detalle: '' });

        // ── ARCHIVAR EL HISTORIAL (8.3 bis b, D-8D) ──────────────────────
        //
        // `revisar_archivado` es una lectura, así que se fabrica, con los
        // números elegidos para que la pantalla enseñe **su aviso más
        // importante**: que borrar envíos se lleva apuntes del registro de
        // actividad que el archivo no guarda. Un preflight sin arrastre aquí
        // dejaría ese aviso sin revisar, que es el mismo motivo por el que
        // `revisar_respaldo` trae pérdidas.
        case 'revisar_archivado':
          return Promise.resolve({
            corte: '2025-10-06T00:00:00Z',
            enviosQueSeArchivan: 35724,
            enviosQueSeQuedan: 12483,
            primero: '2024-02-14T08:12:00Z',
            ultimo: '2025-10-05T18:44:00Z',
            campanas: (e.campanas || []).length || 3,
            arrastradas: [{ tabla: 'activity_event', filas: 41088 }],
            totalArrastrado: 41088,
          });

        // **No finge haber archivado**, y aquí la razón es más fuerte que en
        // el respaldo: contestar «hecho» enseñaría el éxito de un borrado de
        // historial que no ha ocurrido. El error es verdad — en la vista
        // previa no hay disco donde escribir el archivo.
        case 'archivar_historial':
          return Promise.reject({ clave: 'error.app.archivo_no_se_pudo_leer', detalle: '' });

        // ── RECUPERAR, EL LLAVERO PERDIDO (8.4) ──────────────────────────
        //
        // La vista previa puede enseñar esta pantalla porque es la que menos
        // se puede revisar de otra forma: en la aplicación de verdad sólo
        // aparece el día que alguien pierde su llavero, y no es un día para
        // descubrir que el texto está mal.
        //
        // Las cifras describen el caso real: una base que SIGUE EN DISCO y no
        // se puede leer. Decir 0 bytes contaría otra historia —«no había
        // nada»— y es justo la que no hay que contar.
        case 'que_paso_con_la_clave':
          return Promise.resolve({
            bytesDeLaBase: 48 * 1024 * 1024,
            nombreDeLaBase: 'arles.db',
          });

        case 'revisar_respaldo_para_recuperar':
          return Promise.resolve({
            nombreDelArchivo: 'arles-20260921-completo.arles',
            creadoEn: '2026-09-21T09:30:00Z',
            alcance: 'completo',
            esquemaDelRespaldo: 'V11',
            esquemaDeLaBase: 'V11',
            llevaContrasena: true,
            sePuedeRestaurar: true,
            // Sin base legible no hay con qué comparar, y el núcleo tampoco lo
            // inventa: aquí van vacías por lo mismo.
            tablas: [],
            totalDeFilas: 64501,
            noVuelven: [],
            filasQueNoVuelven: 0,
            bajasAhora: 0,
            bajasEnElRespaldo: 37,
            bajasAlMenos: 37,
            cuentasAReconectar: 2,
          });

        // No finge haber recuperado nada, por lo mismo que `restaurar_respaldo`:
        // aquí no hay archivo en disco que abrir, ni llavero que escribir.
        case 'recuperar_desde_respaldo':
          return Promise.reject({ clave: 'error.app.archivo_no_se_pudo_leer', detalle: '' });

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
      // ── DOS EN MARCHA, Y NO ES DECORADO GRATUITO ──────────────────────
      //
      // Hasta el 30/09/2026 el decorado sólo tenía una campaña terminada y un
      // borrador, así que en la demostración **no se veía ni un botón de
      // pausar o detener**: ninguno de esos dos estados los enseña.
      //
      // Dirección trabaja con hasta tres campañas a la vez, que además es el
      // caso donde el freno importa —el riesgo no es detener sin querer, es
      // detener la equivocada—. Sin una campaña enviando, la demostración
      // enseñaba un producto sin freno, que es justo lo que la 7.0 arregla.
      {
        id: '01900000-0000-7000-8000-0000000000c3',
        nombre: 'Clientes de agosto', estado: 'campana.estado.enviando',
        audiencia: 1342, aceptados: 340,
      },
      {
        id: '01900000-0000-7000-8000-0000000000c4',
        nombre: 'Reactivación', estado: 'campana.estado.enPausa',
        audiencia: 420, aceptados: 89,
      },
      // Una tercera enviando, y **quieta a propósito**: es la que enseña la
      // razón. Dirección trabaja con hasta tres a la vez (D-7A), y el caso que
      // hay que poder ver en la demostración es el de dos avanzando y una
      // parada sin que nadie la haya parado.
      {
        id: '01900000-0000-7000-8000-0000000000c5',
        nombre: 'Promoción de octubre', estado: 'campana.estado.enviando',
        audiencia: 860, aceptados: 120,
      },
      {
        id: '01900000-0000-7000-8000-0000000000c2',
        nombre: 'Seguimiento a cotizaciones', estado: 'campana.estado.borrador',
        audiencia: 0, aceptados: 0,
      },
    ],

    // ── LO QUE ACTIVIDAD AÑADE A CADA CAMPAÑA ─────────────────────────────
    //
    // Aparte y no dentro de `campanas` porque `CampanaIpc` no lleva estos
    // campos: mezclarlos haría que el decorado de la lista de campañas
    // enseñara un objeto que el comando real no devuelve, y la próxima
    // divergencia de esquema no se vería.
    //
    // Las tres situaciones que hay que poder mirar están representadas:
    // avanzando con ritmo, enviando y quieta por falta de cupo, y en pausa.
    actividad: {
      // Terminada: nada pendiente, y contestaron 87.
      '01900000-0000-7000-8000-0000000000c1': {
        respondieron: 87, dadoDeBaja: 14, fallo: 0,
      },
      // Enviando y avanzando: seis horas de ritmo desigual, con las dos
      // últimas más bajas — así la estimación no sale de un número plano.
      '01900000-0000-7000-8000-0000000000c3': {
        respondieron: 23, dadoDeBaja: 3, fallo: 11,
        ritmo: [
          { hora: '2026-09-30T12', salieron: 58 },
          { hora: '2026-09-30T13', salieron: 60 },
          { hora: '2026-09-30T14', salieron: 61 },
          { hora: '2026-09-30T15', salieron: 59 },
          { hora: '2026-09-30T16', salieron: 42 },
          { hora: '2026-09-30T17', salieron: 39 },
        ],
      },
      // En pausa: sin ritmo, y con la razón de por qué se paró el bucle.
      '01900000-0000-7000-8000-0000000000c4': {
        respondieron: 6, fallo: 4,
        razon: { clave: 'motor.parado', numeroA: null, numeroB: null, cuando: '2026-09-30T16:40:00Z' },
      },
      // **Enviando y quieta**: el caso que esta pantalla existe para explicar.
      // Se comió el cupo de la hora, y sin decirlo el cero de arriba parece una
      // avería y lo primero que alguien hace es reiniciar ARLES.
      '01900000-0000-7000-8000-0000000000c5': {
        respondieron: 9, fallo: 2, ritmo: [],
        razon: {
          clave: 'ritmo.topeDeLaHora', numeroA: 100, numeroB: null,
          cuando: '2026-09-30T17:52:00Z',
        },
      },
    },

    // El cupo de cada remitente. El primero lo comparten dos campañas, que es
    // lo que explica por qué una de ellas está quieta: sin los nombres, «100 de
    // 100 esta hora» no dice de quién se lo comió.
    cupos: [
      {
        nombre: 'Ventas', direccion: 'ventas@telemetry.mx',
        gastadoHoy: 460, topeDiario: 500, quedanHoy: 40,
        gastadoEstaHora: 100, topePorHora: 100,
        campanas: ['Clientes de agosto', 'Promoción de octubre'], hayReparto: true,
      },
      {
        nombre: 'Contacto', direccion: 'contacto@telemetry.mx',
        gastadoHoy: 89, topeDiario: 300, quedanHoy: 211,
        gastadoEstaHora: 12, topePorHora: 60,
        campanas: ['Reactivación'], hayReparto: false,
      },
    ],

    // ── EL REGISTRO DE LA DEMOSTRACIÓN ────────────────────────────────────
    //
    // Mezcla las tres cosas que hay que poder ver: envíos con su dirección,
    // sucesos del motor que no son de nadie, y ruido **viejo** —marcado con
    // `viejo`— que es lo único que el recorte se lleva.
    registro: [
      {
        id: 'r-1', clave: 'motor.aceptadoPorElProveedor', esDesenlace: true,
        campana: '01900000-0000-7000-8000-0000000000c3', canal: 'email',
        direccion: 'ana.lopez@empresa.mx', numeroA: null, numeroB: null,
        cuando: '2026-09-30T17:48:00Z',
      },
      {
        id: 'r-2', clave: 'reintento.seVolveraAIntentar', esDesenlace: true,
        campana: '01900000-0000-7000-8000-0000000000c3', canal: 'email',
        direccion: 'beto@otraempresa.mx', numeroA: 2, numeroB: 3,
        cuando: '2026-09-30T17:41:00Z',
      },
      {
        id: 'r-3', clave: 'reintento.direccionInexistente', esDesenlace: true,
        campana: '01900000-0000-7000-8000-0000000000c3', canal: 'email',
        direccion: 'no.existe@empresa.mx', numeroA: null, numeroB: null,
        cuando: '2026-09-30T17:36:00Z',
      },
      {
        id: 'r-4', clave: 'ritmo.topeDeLaHora', esDesenlace: false,
        campana: '01900000-0000-7000-8000-0000000000c5', canal: 'email',
        direccion: null, numeroA: 100, numeroB: null,
        cuando: '2026-09-30T17:52:00Z',
      },
      {
        id: 'r-5', clave: 'reintento.pidioLaBaja', esDesenlace: true,
        campana: '01900000-0000-7000-8000-0000000000c1', canal: 'email',
        direccion: 'ana.martinez@tercera.mx', numeroA: null, numeroB: null,
        cuando: '2026-09-29T11:20:00Z',
      },
      {
        id: 'r-6', clave: 'motor.fueraDeVentana', esDesenlace: false,
        campana: '01900000-0000-7000-8000-0000000000c3', canal: 'email',
        direccion: null, numeroA: null, numeroB: null,
        cuando: '2026-09-29T18:00:00Z',
      },
      // Ruido de hace más de dos meses: lo único que el recorte se lleva.
      {
        id: 'r-7', clave: 'motor.fueraDeVentana', esDesenlace: false,
        campana: '01900000-0000-7000-8000-0000000000c1', canal: 'email',
        direccion: null, numeroA: null, numeroB: null,
        cuando: '2026-06-15T18:00:00Z', viejo: true,
      },
      {
        id: 'r-8', clave: 'motor.noHayTrabajo', esDesenlace: false,
        campana: '01900000-0000-7000-8000-0000000000c1', canal: 'email',
        direccion: null, numeroA: null, numeroB: null,
        cuando: '2026-06-14T09:00:00Z', viejo: true,
      },
      // Un envío viejo, que **no** se recorta: es la prueba de a quién se le
      // escribió, y eso se guarda siempre.
      {
        id: 'r-9', clave: 'motor.aceptadoPorElProveedor', esDesenlace: true,
        campana: '01900000-0000-7000-8000-0000000000c1', canal: 'email',
        direccion: 'cliente.antiguo@empresa.mx', numeroA: null, numeroB: null,
        cuando: '2026-06-13T10:00:00Z', viejo: true,
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
