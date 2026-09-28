/**
 * Textos en español de México.
 *
 * Reglas de UX_WRITING.md que este archivo debe respetar:
 *
 * - Tono profesional y claro. Nada de «¡Ups!» ni celebraciones (§94).
 * - Todo error dice **qué pasó, cómo arreglarlo y qué está a salvo** (§95).
 * - «Aceptado» nunca es «Entregado» (§65).
 * - Sin concatenación: interpolación con parámetros nombrados.
 */
export const es = {
  producto: {
    irAlSitio: 'Ir a telemetrymx.com · se abre en tu navegador',
    // ADR-0010: «I» y el número de versión nunca en el mismo renglón.
    nombre: 'ARLES RELAY',
    nombreComercial: 'ARLES RELAY I',
    atribucion: 'Software desarrollado por TELEMETRY INSIGHT',
  },

  nav: {
    principal: 'Navegación principal',
    inicio: 'Inicio',
    campanas: 'Campañas',
    contactos: 'Contactos',
    remitentes: 'Remitentes',
    actividad: 'Actividad',
    ajustes: 'Ajustes',
    // P-11. El botón dice lo que va a hacer, no en qué estado está: «Plegar»
    // cuando está abierta. Al revés obliga a deducir la acción del estado.
    plegar: 'Plegar la navegación',
    expandir: 'Expandir la navegación',
    plegadaPorAncho:
      'La navegación se pliega sola porque la ventana es estrecha. ' +
      'Ensánchala para poder abrirla.',
    // El nombre accesible de las migas. Sin él, un lector de pantalla anuncia
    // «navegación» dos veces —la barra y las migas— sin distinguirlas.
    migas: 'Dónde estás',

    /**
     * Las subsecciones. Cada una es una pantalla que cuelga de una sección y
     * se enseña indentada bajo ella cuando estás dentro.
     *
     * El nombre es el CORTO, no el título de la pantalla.
     *
     * «Bajas» lo decidió Dirección el 22/09/2026, viéndolo: el nombre largo
     * —«Quién pidió no recibir»— se parte en dos líneas dentro de la barra y
     * engorda esa entrada frente a las otras ocho. Una palabra no se parte.
     *
     * El título largo NO se toca: sigue encabezando la pantalla. La barra es
     * una brújula y necesita nombres cortos; el encabezado es una explicación
     * y puede permitirse una frase. No es la misma necesidad ni el mismo sitio.
     */
    sub: {
      configuracion: 'Configuración',
      importar: 'Importar',
      supresion: 'Bajas',
      derechos: 'Derechos',
    },
  },

  inicio: {
    titulo: 'Bienvenido a {producto}',
    // WhatsApp entró en el alcance el 17/09/2026 (P-15). Esta línea decía
    // «campañas de correo» y se quedó corta el mismo día.
    entradilla:
      'ARLES ejecuta campañas de correo y de WhatsApp con tus propias ' +
      'cuentas, respetando los límites que definas.',
    alta: 'Para poder enviar tu primera campaña',
    avance: '{hechos} de {total}',
    verPasos: 'Ver los {n} pasos',

    // ── Esta ventana no envía (entrega 5.4c) ──
    //
    // Aparece cuando hay otro ARLES abierto sobre la misma base: sólo uno puede
    // enviar, y es el que abrió primero.
    //
    // Las tres partes del §95, y ninguna sobra. Sin el «qué está a salvo», la
    // pregunta que queda es «¿entonces mi campaña se perdió?» — y la respuesta
    // es que no, que la está mandando la otra ventana. Sin esa frase, alguien
    // cerraría las dos y volvería a empezar.
    //
    // No dice «instancia» ni «cerrojo». Dice «otra ventana», que es lo que la
    // persona tiene delante.
    noEnvia: {
      titulo: 'Esta ventana no está enviando',
      que: 'Hay otro ARLES abierto con los mismos datos, y sólo uno puede enviar a la vez.',
      como: 'Ciérralo y vuelve a abrir éste, o sigue trabajando desde aquélla.',
      salvo:
        'Tus campañas siguen saliendo por la otra ventana: no se ha perdido ' +
        'nada ni se ha duplicado ningún correo.',
    },

    // Primera vez. Una sola cosa que hacer, sin panel que la rodee: quien abre
    // ARLES por primera vez no tiene nada que mirar en un panel.
    primera: {
      titulo: 'Empieza por aquí',
      entradilla:
        'Antes de enviar nada, ARLES necesita saber de qué empresa salen ' +
        'los correos.',
      porQue:
        'El nombre que verán quienes los reciban, y la zona horaria, que es ' +
        'la que decide a qué hora sale cada uno. Son cinco campos.',
      accion: 'Configurar mi empresa',
      // La cifra evita la conclusión de que con esto ya se puede enviar.
      despues: 'Después vienen cinco pasos más.',
    },

    accesos: {
      titulo: 'Accesos rápidos',
      // Un acceso que aún no lleva a nada lo dice, en vez de desaparecer:
      // quien busca dónde se cargan los contactos necesita leer «aún no»,
      // no encontrarse con el vacío.
      aun: 'aún no',
    },

    enConstruccion: {
      titulo: 'Lo que falta por construir',
      nota: '{n} pasos',
      cuerpo:
        'Los módulos de campañas, canales y actividad aparecerán aquí cuando ' +
        'existan. No se dibujan vacíos a propósito: un panel de cajas en ' +
        'espera anuncia cosas que ARLES todavía no sabe hacer.',
    },
    hecho: 'Hecho',
    pendiente: 'Pendiente',
    llegaEn: 'Llega en la entrega {entrega}',
    sinPasosDisponibles:
      'Los siguientes pasos se construyen en las entregas indicadas. ' +
      'No hay nada más que puedas hacer aquí todavía.',
    paso: {
      empresa: {
        titulo: 'Configurar tu empresa',
        detalle:
          'El nombre, el país y la zona horaria. La zona decide a qué hora ' +
          'sale cada correo.',
      },
      remitente: {
        titulo: 'Conectar una cuenta remitente',
        detalle:
          'La cuenta desde la que se envía. ARLES no envía por su cuenta: ' +
          'usa la tuya, con sus límites.',
      },
      contactos: {
        titulo: 'Cargar tus contactos',
        detalle:
          'Importa tu lista desde un archivo. ARLES no incluye contactos ni ' +
          'los compra.',
      },
      plantilla: {
        titulo: 'Escribir una plantilla',
        detalle: 'El mensaje, con los datos de cada contacto sustituidos.',
      },
      ventana: {
        titulo: 'Definir una ventana de envío',
        detalle:
          'Los días y las horas en que se puede enviar, en la zona horaria ' +
          'de tu empresa.',
      },
      campana: {
        titulo: 'Crear tu primera campaña',
        detalle: 'Une lo anterior y queda lista para revisarse antes de activarse.',
      },
    },
  },

  // C-1 y C-2. Apariencia, no «tema»: quien busca cómo poner la pantalla
  // clara no piensa en la palabra «tema».
  apariencia: {
    titulo: 'Apariencia',
    campo: 'Tema de la interfaz',
    ayuda: 'Se aplica al momento y se recuerda al reabrir ARLES.',
    auto: 'Automático · el de tu sistema',
    oscuro: 'Oscuro',
    claro: 'Claro',
    // No es un detalle cosmético: la elección cambia qué contrastes se aplican,
    // y los dos temas están medidos. Decirlo evita la pregunta de si el claro
    // es «el bueno» o un añadido a medias.
    porQue: '¿Cambia algo más que el color?',
    porQueDetalle:
      'No. Los dos temas se miden con las mismas reglas de contraste, así ' +
      'que ninguno se lee peor que el otro. «Automático» sigue a tu sistema ' +
      'y cambia con él mientras ARLES está abierto.',
  },

  // ── Configurar ARLES ────────────────────────────────────────────────
  //
  // Vivía en Ajustes hasta el 21/09/2026. Dirección lo señaló: Ajustes es de
  // la aplicación, no del negocio. Ver PantallaConfiguracion.vue.
  configuracion: {
    titulo: 'Configurar ARLES',
    entradilla:
      'Los datos de tu empresa y lo que falta para poder enviar tu primera campaña.',
    // «Editar» y no «Modificar»: es la palabra que la gente busca, y el botón
    // tiene que encontrarse sin leerlo dos veces.
    editar: 'Editar',
    cancelar: 'Cancelar',
  },

  ajustes: {
    entradilla: 'Cómo se ve y cómo se comporta la aplicación.',
    // Aquí vivían `datosDelNegocio`, `datosDelNegocioDetalle` e
    // `irAConfiguracion`: el bloque puente hacia la configuración. Dirección
    // lo mandó quitar el 21/09/2026, y con razón — en cuanto la barra lateral
    // enseña «Inicio › Configuración», el puente dejó de ser un camino y pasó
    // a ser Ajustes volviendo a hablar del negocio.
  },

  empresa: {
    entradilla: 'Los datos de tu empresa y cómo afectan a los envíos.',
    titulo: 'Configuración de empresa',
    guardar: 'Guardar',
    guardada: 'Configuración guardada.',
    guardadaDetalle: 'Los envíos usarán estos datos a partir de ahora.',
    porQueLaZona: '¿Por qué importa la zona horaria?',
    porQueLaZonaDetalle:
      'Las ventanas de envío se calculan siempre en la zona de tu empresa, ' +
      'nunca en la del equipo donde esté ARLES. Así una campaña programada ' +
      'de 9 a 18 sale a esa hora aunque el equipo viaje a otro huso.',
    // Nombres legibles de lo que ofrecen los desplegables.
    //
    // El **valor** sigue siendo el que exige el núcleo —`MX`,
    // `America/Mexico_City`—; esto es sólo lo que se lee. Enseñar el
    // identificador crudo obliga al usuario a saber qué es un identificador
    // IANA para elegir su ciudad.
    //
    // El validador comprueba que ninguna opción del núcleo se queda sin
    // nombre: si falta, se enseña el valor crudo y nadie se entera.
    pais: {
      MX: 'México',
    },
    zona: {
      'America/Mexico_City': 'Ciudad de México',
      'America/Cancun': 'Cancún · Quintana Roo',
      'America/Merida': 'Mérida · Yucatán',
      'America/Monterrey': 'Monterrey · Nuevo León',
      'America/Matamoros': 'Matamoros · frontera',
      'America/Chihuahua': 'Chihuahua',
      'America/Ojinaga': 'Ojinaga · frontera',
      'America/Mazatlan': 'Mazatlán · Sinaloa',
      'America/Bahia_Banderas': 'Bahía de Banderas · Nayarit',
      'America/Hermosillo': 'Hermosillo · Sonora',
      'America/Tijuana': 'Tijuana · frontera',
      UTC: 'UTC · tiempo universal',
    },
    campo: {
      nombreComercial: 'Nombre comercial',
      pais: 'País',
      zonaHoraria: 'Zona horaria',
      correoCorporativo: 'Correo corporativo',
      sitioWeb: 'Sitio web',
    },
    ayuda: {
      nombreComercial: 'Como quieres que aparezca en tus correos.',
      pais: 'Determina el marco legal aplicable. Esta versión opera en México.',
      zonaHoraria: 'Decide a qué hora sale cada correo.',
      correoCorporativo: 'Para avisos internos de ARLES. No se usa para enviar campañas.',
      sitioWeb: 'Opcional. Con https:// delante.',
    },
    // Claves que devuelve `arles_core::empresa`. Una por motivo, porque «dato
    // inválido» no dice qué corregir.
    error: {
      generico: 'Este dato no es válido.',
      nombreVacio: 'Escribe el nombre comercial de tu empresa.',
      nombreLargo: 'El nombre es demasiado largo: usa 120 caracteres o menos.',
      nombreConSaltos: 'El nombre no puede contener saltos de línea.',
      paisNoSoportado: 'Esta versión de ARLES opera en México.',
      zonaNoSoportada: 'Elige una zona horaria de la lista.',
      // `{'@'}` y no una arroba suelta: en la gramática de vue-i18n `@` abre
      // un enlace a otra clave, y el texto no compila. Reventaba la pantalla
      // entera, en silencio. Lo vigila una prueba que compila todos los textos.
      correoInvalido: "Revisa que el correo tenga la forma nombre{'@'}dominio.com.",
      sitioLargo: 'La dirección es demasiado larga.',
      sitioSinEsquema: 'Escribe la dirección completa, empezando por https://.',
      sitioInvalido: 'Esta dirección no es válida. Debe empezar por https:// o http://.',
    },
  },

  /** Estados de un intento de envío. Ver arles-core::AttemptState. */
  intento: {
    queued: 'En cola',
    claimed: 'En cola',
    sending: 'Enviando',
    // NO «Entregado»: el proveedor solo confirma que se hizo cargo.
    sent: 'Aceptado',
    failed: 'Reintentando',
    permanently_failed: 'Falló',
    suppressed: 'Suprimido',
    cancelled: 'Cancelado',
    presumed_sent: 'Envío no confirmado',
  },

  /**
   * Cuánto va a tardar la campaña (§11).
   *
   * Es probablemente la información más valiosa del producto: es lo que hace
   * que alguien reconsidere su configuración ANTES de lanzar, no después.
   * Todas las cifras son cifras (§94), y la fecha se dice «alrededor de»
   * porque la precisión del cálculo es de horas, no de minutos.
   */
  duracion: {
    // {cuantos} mensajes, {cuando} la fecha estimada.
    termina:
      'Esta campaña enviará {cuantos} mensajes y terminará alrededor del {cuando}.',
    // El caso que hay que cazar antes de activar: con este tope no sale ni uno.
    noAvanza:
      'Con este tope no saldría ni un mensaje. Revisa los límites por hora y ' +
      'por día antes de activar.',
    // {faltan} es lo que se quedaría fuera. Decir «terminará en 2053» sería
    // correcto y no serviría de nada; decir cuántos se quedan fuera, sí.
    noCabeEnElHorizonte:
      'Con esta configuración la campaña no termina en cinco años: {faltan} ' +
      'mensajes se quedarían sin enviar. Sube los límites, recorta la ' +
      'audiencia o divide la campaña.',

    // Quién pone el freno. Sin esto, lo normal es ampliar el horario y no
    // notar ninguna diferencia porque el freno era el tope diario.
    mandaElTopeDelDia:
      'El freno es tu tope de {porDia} al día. Ampliar el horario no daría ni ' +
      'un envío más.',
    mandaElTopeDeLaHora:
      'El freno es el horario: {horas} horas a {porHora} por hora son ' +
      '{techo} al día. Subir el tope diario no daría ni un envío más.',
    mandanLosDos:
      'Los dos topes dan lo mismo: {techo} al día. Para ir más rápido hay que ' +
      'subir los dos.',

    // Cabecera del desglose. Se detallan catorce días; el total se dice entero.
    porDia: 'Envíos por día',
    diasConEnvio: '{dias} días con envío',
    techoDiario: '{techo} mensajes al día',
  },

  /**
   * Las ocho comprobaciones de antes de activar (§63).
   *
   * Cada punto tiene TRES respuestas, no dos. «Sin comprobar» no es un
   * aprobado — es la misma regla que el validador aplica a las sondas.
   */
  preflight: {
    titulo: 'Comprobaciones antes de activar',
    lista: 'Todo listo para activar',
    // §94: cifras. {fallan} lo que hay que arreglar, {sinComprobar} lo que hay
    // que comprobar. Se dicen aparte porque lo que hay que hacer es distinto.
    bloqueada:
      'No se puede activar todavía: {fallan} por arreglar y {sinComprobar} ' +
      'sin comprobar.',

    pasa: 'Bien',
    falla: 'Hay que arreglarlo',
    // Ni aprobado ni suspenso: no se ha podido mirar, y por eso no se activa.
    sinComprobar: 'Sin comprobar',
    sinComprobarDetalle:
      'ARLES no ha podido comprobar esto, así que no lo da por bueno. Un ' +
      'punto sin comprobar impide activar igual que uno que falla.',

    laCuentaAutentica: 'La cuenta remitente conecta con su proveedor',
    laCredencialSirve: 'La contraseña o el permiso siguen vigentes',
    laAudienciaNoEstaVacia: 'Queda alguien a quien escribir tras aplicar las bajas',
    lasVariablesSeResuelven: 'Todos los datos de la plantilla se rellenan',
    elHorarioAlcanza: 'El horario configurado alcanza para terminar',
    hayConexion: 'Hay conexión a internet',
    seHizoElEnvioDePrueba: 'Se hizo el envío de prueba',
    // El único que no bloquea: informar, no dictar (§154).
    losRegistrosDelDominio: 'Los registros SPF, DKIM y DMARC de tu dominio',
    losRegistrosDelDominioAviso:
      'Sin estos registros tus correos entregan peor, pero entregan. Esto no ' +
      'impide activar: lo arregla quien administre tu dominio.',
  },

  /**
   * El motor de ejecución (Fase 4).
   *
   * Aquí no hay errores de tres partes: son **rótulos de estado**, lo que la
   * pantalla de Actividad enseña mientras la campaña corre. Las reglas que sí
   * aplican son el §94 —cifras, no adjetivos: «2 de 6», nunca «casi listo»— y
   * el §65: aceptado no es entregado.
   */
  campana: {
    estado: {
      borrador: 'Borrador',
      lista: 'Lista para activar',
      programada: 'Programada',
      enviando: 'Enviando',
      enPausa: 'En pausa',
      detenida: 'Detenida',
      terminada: 'Terminada',
    },

    accion: {
      pausar: 'Pausar',
      reanudar: 'Reanudar',
      // El botón dice lo que hace. La confirmación explica lo que cuesta.
      detener: 'Detener',
      // {cuantos} lo calcula ARLES antes de preguntar: una confirmación sin
      // cifra no es una confirmación informada, y esto no se deshace.
      confirmarDetener:
        'Detener cancela {cuantos} mensajes que todavía no han salido. ' +
        'Los que ya salieron no se pueden recuperar, y esto no se puede ' +
        'deshacer: para volver a escribir a esa gente habría que crear una ' +
        'campaña nueva.',
      pausarExplicacion:
        'Pausar conserva la cola intacta. Puedes reanudar cuando quieras y ' +
        'seguirá donde se quedó.',
    },

    parada: {
      levantada: 'El envío está activo',
      // {desde} es la hora en que se pulsó. Un «detenido» a secas invita a
      // preguntarse si se quedó puesto de ayer (§94).
      puesta: 'Envío detenido desde {desde}',
      poner: 'Detener todos los envíos',
      levantar: 'Reanudar los envíos',
      levantarExplicacion:
        'Quitar la parada no reanuda ninguna campaña: las que estaban en ' +
        'pausa siguen en pausa. Cada una se vuelve a arrancar por separado.',
    },

    error: {
      detenerNoSeDeshace: {
        que: 'Esa campaña está detenida, y detener no se deshace.',
        como:
          'Los mensajes que no habían salido se cancelaron. Para volver a ' +
          'escribir a esas personas, crea una campaña nueva.',
        salvo:
          'Los mensajes que sí salieron siguen registrados, y nadie ha ' +
          'recibido nada dos veces.',
      },
      noSePuedePausar: {
        que: 'Esa campaña no está enviando, así que no hay nada que pausar.',
        como: 'Actualiza la lista de campañas para ver cómo está ahora.',
        salvo: 'No se modificó nada.',
      },
      noSePuedeReanudar: {
        que: 'Esa campaña no está en pausa.',
        como: 'Actualiza la lista de campañas para ver cómo está ahora.',
        salvo: 'No se modificó nada.',
      },
      noHayNadaQueDetener: {
        que: 'Esa campaña ya no tiene mensajes en cola.',
        como: 'Actualiza la lista de campañas para ver cómo está ahora.',
        salvo: 'No se modificó nada, y no se canceló ningún mensaje.',
      },
      cambioMientrasTanto: {
        que: 'Esa campaña cambió de estado mientras se preparaba la acción.',
        como: 'Actualiza la pantalla y vuelve a decidir sobre lo que hay ahora.',
        salvo: 'No se modificó nada: ARLES no aplica una decisión tomada sobre un estado viejo.',
      },
    },
  },

  /**
   * Lo que el trabajador hace en cada paso. Es lo que se ve en Actividad.
   */
  motor: {
    // UN SUCESO, UN TEXTO. Aquí sólo están los estados del bucle que no
    // describe nadie más. Los desenlaces de un intento —reintento, rendición,
    // baja, credencial— los nombra `reintento.*`, y los motivos por los que no
    // cabe un envío, `ritmo.*`. Tener una copia aquí hacía que la pantalla de
    // Actividad dijera «Se volverá a intentar» y el detalle del mismo envío,
    // a un clic, «Reintento 2 de 3»: el mismo hecho contado de dos maneras.
    parado: 'Detenido',
    fueraDeVentana: 'Fuera del horario de envío',
    noHayTrabajo: 'No queda nada por enviar ahora mismo',
    cancelado: 'Cancelado antes de salir',
    // §65: el proveedor se hizo cargo. No sabemos si llegó al buzón.
    aceptadoPorElProveedor: 'Aceptado por el proveedor',
    // {minutos} es una cifra: «vuelve a intentarlo en 23 minutos» es un hecho;
    // «la cuenta está parada» a secas se lee como una avería (§94).
    laCuentaEstaEnPausa:
      'Esta cuenta está en pausa por fallos seguidos. Vuelve a intentarlo en ' +
      '{minutos} minutos.',
  },

  /**
   * Cómo viaja la conexión con el servidor de correo.
   *
   * Las dos cifran. **No hay un tercer valor**, y esto no es un desplegable
   * con opciones: es lo que ARLES enseña de una cuenta ya configurada (§29).
   */
  correo: {
    // ── Los campos del formulario de la cuenta · entrega 5.3 ─────────────
    //
    // Una línea por campo, en imperativo y diciendo qué hacer: el formulario
    // los pinta debajo de la casilla que está mal. «Campo inválido» no dice
    // qué corregir.
    error: {
      // Para el caso que no debería ocurrir: mejor decir algo que callar.
      generico: 'Revisa los datos de la cuenta.',
      servidorVacio: 'Escribe el servidor de tu correo, como smtp.gmail.com.',
      servidorLargo: 'El nombre del servidor es demasiado largo.',
      servidorNoVale:
        'Escribe sólo el nombre del servidor, sin espacios y sin http://.',
      // La regla menos obvia, así que el texto explica el porqué: sin esto,
      // quien lo lea pensará que ARLES es quisquilloso y buscará cómo
      // saltárselo.
      servidorEsUnaIp:
        'Escribe el nombre del servidor, no su dirección IP. El certificado ' +
        'de seguridad se emite para el nombre, y con una IP no se puede ' +
        'comprobar que estás hablando con quien crees.',
      puertoNoVale: 'El puerto es un número entre 1 y 65535. Para Gmail, 587.',
      cifradoNoVale: 'Elige una de las dos formas de cifrado.',
      usuarioVacio: 'Escribe el usuario con el que entras a tu correo.',
      usuarioLargo: 'El usuario es demasiado largo.',
      remitenteNoVale:
        "Revisa que el remitente tenga la forma nombre{'@'}dominio.com.",
      nombreLargo: 'El nombre es demasiado largo.',
      // No es un campo del formulario en sentido estricto —la contraseña no
      // se guarda con los demás datos—, pero se marca igual que uno para que
      // la pantalla señale la casilla en vez de soltar un aviso general.
      contrasenaVacia:
        'Escribe la contraseña de la cuenta. Si es de Gmail, tiene que ser ' +
        'una contraseña de aplicación.',
    },

    cifrado: {
      starttls: 'Cifrado (STARTTLS, puerto 587)',
      directo: 'Cifrado desde el inicio (puerto 465)',
    },

    // ── El resultado de «Probar conexión» · entrega 5.3 ───────────────────
    //
    // Se leen mientras alguien configura una cuenta, con el formulario
    // delante. Cada uno lleva a UNA acción distinta; si dos dijeran lo mismo,
    // uno de los dos sobraría.
    //
    // Llevan las tres partes del §95 aunque no sean errores del núcleo: quien
    // los lee está atascado, y «no se pudo conectar» a secas deja a alguien
    // probando combinaciones de puerto durante media hora.
    prueba: {
      correcto: {
        que: 'La cuenta funciona.',
        como:
          'Se conectó al servidor, la conexión va cifrada y la contraseña es ' +
          'correcta. Ya puedes guardarla.',
        // §65: aceptado no es entregado, y conectarse no es haber enviado.
        salvo: 'No se envió ningún correo: sólo se comprobó la conexión.',
      },
      no_se_llego: {
        que: 'No se pudo llegar al servidor.',
        como:
          'Revisa el nombre del servidor y el puerto. Si están bien, puede ' +
          'que la red de tu oficina bloquee la salida de correo: muchas lo ' +
          'hacen. Pruébalo desde otra red para salir de dudas.',
        salvo: 'No se envió ningún correo y no se guardó nada.',
      },
      tardo_demasiado: {
        que: 'El servidor aceptó la conexión y luego no contestó.',
        // A propósito NO se manda a revisar el puerto: si hubo conexión, el
        // nombre y el puerto están bien, y mandar ahí es hacer perder el rato.
        como:
          'El nombre y el puerto están bien, porque sí se llegó. Vuelve a ' +
          'intentarlo en unos minutos; si sigue igual, el problema está en el ' +
          'servidor de correo.',
        salvo: 'No se envió ningún correo y no se guardó nada.',
      },
      sin_cifrado: {
        que: 'No se pudo cifrar la conexión con el servidor.',
        // Aquí es donde se pide la casilla. La respuesta va por delante, sin
        // ambigüedad, porque dejarla abierta es invitar a pedirla otra vez.
        como:
          'El servidor no ofrece cifrado, o su certificado no corresponde al ' +
          'nombre que escribiste. Revisa el nombre y el puerto. ARLES no ' +
          'envía por una conexión sin cifrar y no hay forma de desactivarlo: ' +
          'la contraseña viajaría legible por la red.',
        salvo:
          'La contraseña no llegó a enviarse: se cortó antes. No se envió ' +
          'ningún correo y no se guardó nada.',
      },
      contrasena_rechazada: {
        que: 'El servidor rechazó la contraseña.',
        // El punto donde más gente se atasca, así que va el camino exacto.
        como:
          'Si es una cuenta de Gmail, necesitas una «contraseña de ' +
          'aplicación», que no es la de tu cuenta de Google: se genera en la ' +
          'configuración de seguridad de Google y hace falta tener activada ' +
          'la verificación en dos pasos.',
        salvo:
          'No se envió ningún correo. Si ya tenías una cuenta guardada, sigue ' +
          'como estaba.',
      },
      no_disponible: {
        que: 'El servidor contestó que ahora no puede atender.',
        // Es temporal, y decirlo importa: sin esto, alguien genera una
        // contraseña nueva —y tira la que funcionaba— para arreglar algo que
        // se arregla solo.
        como:
          'Es algo temporal del servidor, no un problema de tu contraseña. ' +
          'Espera unos minutos y vuelve a probar.',
        salvo: 'No se envió ningún correo y no se guardó nada.',
      },
    },
  },

  /**
   * El disyuntor por cuenta (§7).
   *
   * Tras cinco fallos seguidos del canal, esa cuenta deja de enviar. Un fallo
   * silencioso que simplemente ralentiza la campaña es peor que una parada
   * visible.
   */
  disyuntor: {
    cerrado: 'La cuenta está enviando con normalidad',
    // {minutos} para no obligar a dividir segundos entre sesenta al leerlo.
    abierto:
      'ARLES paró esta cuenta tras {fallos} fallos seguidos. Volverá a ' +
      'intentarlo en {minutos} minutos.',
    porQue:
      'Insistir contra un servidor que no responde no consigue que responda, ' +
      'y el proveedor toma nota de que este remitente sigue llamando.',
    // La diferencia que decide qué tiene que hacer el usuario.
    porCredencial:
      'Esta cuenta no para por la red: para porque su contraseña o su permiso ' +
      'dejaron de servir. Esperar no lo arregla — hay que renovarlo.',
    cerrarAMano: 'Ya lo arreglé, reanudar esta cuenta',

    // Lo que decide el núcleo, para la bitácora y la pantalla de Remitentes.
    reiniciar: 'Un envío salió bien: la cuenta vuelve a contar desde cero',
    contar: 'Fallo {van} de {de} antes de parar la cuenta',
    abrir: 'La cuenta se detiene',
    // Que una dirección no exista no dice nada malo del remitente.
    noCuenta: 'Este fallo es del destinatario, no de la cuenta',
  },

  /**
   * Por qué un envío se reintenta, se rinde o para la cuenta (D-12).
   */
  reintento: {
    // §94: «2 de 3» es una cifra. «Pronto» no.
    seVolveraAIntentar: 'Reintento {cual} de {de}',
    seAgotaronLosIntentos: 'Se agotaron los {de} intentos',
    direccionInexistente: 'Esa dirección no existe: se añadió a la lista de bajas',
    laCuentaNoAutentica: 'La cuenta remitente no autentica: se detuvo el envío',
    pidioLaBaja: 'Esa persona pidió no recibir',
  },

  /**
   * Por qué no cabe un envío más (§48, §53).
   */
  ritmo: {
    // El equipo estuvo suspendido. ARLES lo detecta para poder explicar el
    // retraso — NO para tocar el presupuesto: ver la nota de arles-core/ritmo.
    sinNovedad: 'El reloj corrió con normalidad',
    huboSuspension:
      'Tu equipo estuvo suspendido {minutos} minutos, así que la campaña va ' +
      'con retraso. No se perdió ningún envío ni se acumularon permisos.',
    fueraDeVentana: 'Fuera del horario que configuraste para esta campaña',
    topeDeLaHora: 'Se alcanzó el tope de {cabian} envíos por hora',
    topeDelDia: 'Se alcanzó el tope de {cabian} envíos al día',
  },

  /**
   * Qué tiene de malo un horario de envío (D-10).
   */
  ventana: {
    sinDias: {
      que: 'No marcaste ningún día, así que la campaña no enviaría nunca.',
      como: 'Marca al menos un día de la semana.',
      salvo: 'No se guardó el horario. La campaña sigue como estaba.',
    },
    horasAlReves: {
      que: 'La hora de fin ({hasta}) no es posterior a la de inicio ({desde}).',
      como: 'Pon una hora de fin mayor que la de inicio.',
      salvo: 'No se guardó el horario. La campaña sigue como estaba.',
    },
    horaFueraDeRango: {
      que: 'La hora {hora} no existe: van de 0 a 24.',
      como: 'Elige una hora entre 0 y 24.',
      salvo: 'No se guardó el horario. La campaña sigue como estaba.',
    },
    zonaDesconocida: {
      que: 'Esa zona horaria no está en la lista que ARLES reconoce.',
      como: 'Elige una de las zonas del desplegable.',
      salvo: 'No se guardó el horario. La campaña sigue como estaba.',
    },
  },

  /**
   * El cerrojo de instancia única: un solo ARLES sobre una sola base.
   */
  instancia: {
    yaHayOtroAbierto: {
      // {latioHaceS} es una cifra: dice que el otro está vivo, no que quizá.
      que: 'Ya hay otro ARLES abierto sobre estos mismos datos (dio señal hace {latioHaceS} segundos).',
      como:
        'Usa la ventana que ya está abierta. Si crees que no hay ninguna, ' +
        'espera un minuto y medio y vuelve a intentarlo.',
      salvo:
        'Tus datos y tus campañas están intactos, y la que esté enviando ' +
        'sigue enviando con normalidad.',
    },
    loTomoOtroALaVez: {
      que: 'Otro ARLES abrió estos datos mientras éste arrancaba.',
      como: 'Usa esa otra ventana. Ésta no va a enviar nada.',
      salvo: 'No se envió ni se canceló nada desde esta ventana.',
    },
    noSePudoEscribir: {
      que: 'ARLES no puede escribir junto a tu base de datos.',
      como:
        'Comprueba que la carpeta de datos existe, que tienes permiso para ' +
        'escribir en ella y que queda espacio en el disco.',
      salvo: 'No se modificó nada.',
    },
  },

  honestidad: {
    porQueAceptado: '¿Por qué «aceptado» y no «entregado»?',
    porQueAceptadoDetalle:
      'Tu proveedor de correo confirma que recibió el mensaje y que intentará ' +
      'entregarlo, pero no informa si llegó al buzón del destinatario. ARLES ' +
      'solo muestra lo que puede verificar.',
    rebotesParciales:
      'La detección automática de rebotes es parcial en esta versión. ARLES ' +
      'detecta los rechazos que el servidor de destino comunica durante el ' +
      'envío, pero no los que llegan después como correo a tu bandeja.',
    rebotesParcialesQueHacer:
      'Revisa tu bandeja periódicamente y suprime a mano las direcciones que reboten.',
    respaldosSinCifrar:
      'Los respaldos no están cifrados en esta versión. El archivo .arles ' +
      'contiene los datos de tus contactos: guárdalo en un lugar seguro.',
  },

  error: {
    // Cada error lleva sus tres partes (§95).

    // ── La cuenta de correo · entrega 5.2 ─────────────────────────────────
    //
    // Son problemas de CONFIGURACIÓN de la cuenta, no de un envío concreto.
    // Se leen en la pantalla de Remitentes, mientras alguien da de alta un
    // remitente, y por eso el «cómo» habla de campos de ese formulario.
    correo: {
      no_se_pudo_cifrar: {
        que: 'No se pudo preparar la conexión cifrada con el servidor.',
        // No se ofrece ninguna forma de seguir sin cifrar, ni «avanzada»:
        // la contraseña viajaría legible por la red (§29).
        como:
          'Revisa el nombre del servidor y el puerto. ARLES no envía por una ' +
          'conexión sin cifrar, así que no hay una opción para saltarse esto.',
        salvo: 'No se envió ningún correo y la cuenta quedó como estaba.',
      },
      el_servidor_no_vale: {
        que: 'El servidor que escribiste no es un nombre válido.',
        // El nombre y no la IP: el certificado se emite para el nombre.
        como:
          'Escribe el nombre del servidor, como smtp.gmail.com, sin espacios ' +
          'y sin usar una dirección IP.',
        salvo: 'No se envió ningún correo y la cuenta quedó como estaba.',
      },
      remitente_no_vale: {
        que: 'La dirección desde la que se enviaría no es una dirección de correo.',
        como: "Revisa que tenga la forma nombre{'@'}dominio.com.",
        salvo: 'No se envió ningún correo y la cuenta quedó como estaba.',
      },
      no_se_pudo_armar: {
        que: 'No se pudo preparar el correo con estos datos.',
        // Esto es un fallo nuestro, y se dice: mandar a revisar la cuenta
        // cuando el problema es de ARLES hace perder media tarde.
        como:
          'Vuelve a intentarlo. Si se repite, es un fallo de ARLES: avísanos ' +
          'con lo que aparece en esta pantalla.',
        salvo: 'No se envió nada y no se marcó a nadie.',
      },
    },

    cifrado_desconocido: {
      que: 'Una cuenta de correo guardada tiene un cifrado que no reconocemos.',
      // No se repara solo a propósito: elegir «el más parecido» aquí sería
      // decidir por su cuenta cómo viaja la contraseña de la empresa.
      como:
        'Abre esa cuenta en Remitentes y vuelve a elegir cómo se cifra la ' +
        'conexión. ARLES no elige por ti: sería decidir cómo viaja tu ' +
        'contraseña.',
      salvo: 'No se envió nada con esa cuenta y no se perdió ningún dato.',
    },

    email_invalido: {
      que: 'La dirección de correo no es válida.',
      como: "Revisa que tenga la forma nombre{'@'}dominio.com.",
      salvo: 'No se guardó ningún cambio.',
    },
    // L-1 · las etapas de una campaña. No los puede provocar quien usa ARLES a
    // ciegas: el asistente ofrece lo que se puede elegir. Están porque el
    // núcleo valida igualmente, y un error sin texto se pinta como su clave.
    etapa: {
      sin_etapas: {
        que: 'La campaña no tiene ninguna etapa.',
        como: 'Añade al menos una: un canal, un remitente y un mensaje.',
        salvo: 'La campaña sigue en borrador. No se envió nada.',
      },
      demasiadas: {
        que: 'Una campaña admite como mucho dos etapas.',
        como: 'Quita una, o crea una segunda campaña para el resto.',
        salvo: 'La campaña sigue como estaba.',
      },
      posiciones: {
        que: 'Las etapas están mal numeradas.',
        como: 'Tienen que ser la 1 y la 2, sin huecos ni repetidas.',
        salvo: 'La campaña sigue como estaba.',
      },
      canal_repetido: {
        que: 'Las dos etapas usan el mismo canal.',
        como:
          'Una etapa por canal: si las dos son de correo, la segunda no ' +
          'enviaría nada — para ARLES sería el mismo envío repetido.',
        salvo: 'La campaña sigue en borrador. No se envió nada.',
      },
      primera_no_espera: {
        que: 'La primera etapa no puede esperar ni depender de otra.',
        como: 'Quita la espera y la condición de la etapa 1.',
        salvo:
          'La campaña sigue como estaba. Si se hubiera activado así, no ' +
          'habría enviado nada y no habría forma de saber por qué.',
      },
      espera_desmesurada: {
        que: 'La espera entre etapas pasa de 30 días.',
        como: 'Ponla en 30 días o menos.',
        salvo: 'La campaña sigue como estaba.',
      },
      condicion_desconocida: {
        que: 'La condición de la etapa no es una de las admitidas.',
        como: 'Vuelve a elegirla en el paso de ritmo de la campaña.',
        salvo: 'La campaña sigue como estaba.',
      },
    },
    telefono_invalido: {
      que: 'El número de teléfono no es válido.',
      como:
        'Escríbelo con lada, por ejemplo 81 1234 5678, o con el prefijo del ' +
        'país si es de fuera de México: +1 415 555 0132.',
      salvo: 'No se guardó ningún cambio.',
    },
    // No lo puede provocar el usuario: el canal lo elige ARLES, no se escribe.
    // Si aparece, es una fila de la base con un canal que no reconocemos.
    motivo_de_supresion_desconocido: {
      que: 'Una baja guardada tiene un motivo que ARLES no reconoce.',
      como: 'Avisa a soporte: la base pudo editarse desde fuera de ARLES.',
      salvo: 'Esa persona sigue sin recibir nada. No se modificó nada.',
    },
    origen_de_supresion_desconocido: {
      que: 'Una baja guardada no dice bien quién la registró.',
      como: 'Avisa a soporte: la base pudo editarse desde fuera de ARLES.',
      salvo: 'Esa persona sigue sin recibir nada. No se modificó nada.',
    },
    alcance_de_supresion_desconocido: {
      que: 'Una baja guardada no dice bien a qué canales afecta.',
      como: 'Avisa a soporte: la base pudo editarse desde fuera de ARLES.',
      salvo: 'Esa persona sigue sin recibir nada. No se modificó nada.',
    },
    estado_de_campana_desconocido: {
      que: 'Una campaña guardada no dice bien en qué punto está.',
      como:
        'Avisa a soporte: la base pudo editarse desde fuera de ARLES. ' +
        'Mientras tanto, esa campaña no envía nada.',
      // Ante la duda, el motor para. No se supone «borrador»: suponer el
      // estado más inofensivo dejaría pasar un «reanudar» sobre una campaña
      // que en realidad estaba detenida.
      salvo:
        'No ha salido ni un mensaje más de esa campaña, y no se ha ' +
        'cancelado ninguno. Las demás campañas siguen normal.',
    },
    supresion_sin_canales: {
      que: 'No se dijo qué correo o qué número hay que dar de baja.',
      como: 'Elige al menos una dirección antes de continuar.',
      salvo: 'No se dio de baja a nadie.',
    },
    alcance_no_coincide: {
      que: 'Se pidió dar de baja un solo canal, pero llegaron varios.',
      como: 'Vuelve a intentarlo eligiendo «todos los canales» o uno solo.',
      salvo: 'No se dio de baja a nadie. ARLES no amplía una baja por su cuenta.',
    },
    origen_desconocido: {
      que: 'El origen declarado de esa importación no es uno de los que ARLES reconoce.',
      como:
        'No se puede corregir desde aquí. Si necesitas ese registro para ' +
        'responder a una reclamación, restaura el respaldo anterior a la ' +
        'importación.',
      // No se repara cayendo en «Otro»: eso cambiaría lo que el usuario
      // declaró, que es justo el dato que se guarda para poder probarlo.
      salvo:
        'Los contactos de esa importación siguen ahí, y el texto que se ' +
        'aceptó también.',
    },
    canal_desconocido: {
      que: 'Ese contacto tiene un medio de contacto que ARLES no reconoce.',
      como: 'Vuelve a importarlo, o avísanos con el nombre del contacto.',
      salvo:
        'No se le escribió nada. ARLES se detiene antes de enviar cuando no ' +
        'sabe por qué medio hacerlo.',
    },
    id_invalido: {
      que: 'El identificador no es válido.',
      como: 'Vuelve a abrir la pantalla desde el menú.',
      salvo: 'Tus datos están intactos.',
    },
    transicion_invalida: {
      que: 'Esa acción no es posible en el estado actual del envío.',
      como: 'Actualiza la vista para ver el estado más reciente.',
      salvo: 'No se modificó ningún envío.',
    },
    import: {
      formato_desconocido: {
        que: 'Ese archivo no es una tabla que ARLES sepa leer.',
        como: 'Usa un archivo .csv o .xlsx. Desde Excel: Archivo → Guardar como.',
        salvo: 'No se importó nada.',
      },
      demasiado_grande: {
        que: 'El archivo pesa más de lo que ARLES admite.',
        como: 'Pártelo en varios archivos más pequeños e impórtalos por turnos.',
        salvo: 'No se importó nada.',
      },
      demasiados_datos: {
        que: 'El archivo trae más datos de los que ARLES puede leer de una vez.',
        como:
          'Pártelo en varios archivos. Si crees que es pequeño, ábrelo en ' +
          'Excel y borra las filas y columnas vacías del final: a veces ' +
          'ocupan miles de celdas sin que se vean.',
        salvo: 'No se importó nada.',
      },
      sin_encabezados: {
        que: 'El archivo no tiene una fila de encabezados.',
        como:
          'La primera fila con datos tiene que decir qué es cada columna: ' +
          '«Nombre», «Correo», «Teléfono».',
        salvo: 'No se importó nada.',
      },
      sin_hojas: {
        que: 'El archivo de Excel no tiene ninguna hoja con datos.',
        como: 'Ábrelo y comprueba que la primera hoja tiene la tabla.',
        salvo: 'No se importó nada.',
      },
      no_se_pudo_leer: {
        que: 'No se pudo leer el archivo.',
        como:
          'Comprueba que no lo tienes abierto en Excel y que no está dañado.',
        salvo: 'No se importó nada.',
      },
    },
    db: {
      clave_incorrecta: {
        que: 'No se pudo abrir la base de datos con las credenciales guardadas.',
        como: 'Verifica que el almacén de credenciales del sistema esté desbloqueado.',
        salvo: 'La base de datos no se ha modificado.',
      },
      clave_mal_formada: {
        que: 'La credencial guardada en el sistema no tiene el formato esperado.',
        como: 'Contacta con soporte antes de continuar.',
        salvo: 'La base de datos no se ha modificado.',
      },
      migracion: {
        que: 'No se pudo actualizar la estructura de la base de datos.',
        como: 'Cierra ARLES y vuelve a abrirlo. Si persiste, restaura tu último respaldo.',
        salvo: 'Se hizo una copia de tu base de datos antes de intentarlo.',
      },
      sqlite: {
        que: 'Ocurrió un error al acceder a los datos.',
        como: 'Cierra ARLES y vuelve a abrirlo.',
        salvo: 'Las operaciones incompletas se revirtieron por completo.',
      },
      direccion_en_uso: {
        // `{detalle}` es la dirección en conflicto, que llega desde el núcleo.
        // Sin ella, con hasta diez formas de contacto en el formulario, el
        // mensaje obliga a repasarlas todas para dar con la repetida.
        que: '{detalle} ya está registrada en otro contacto.',
        como:
          'Búscala en la lista de contactos para ver de quién es. Si de verdad ' +
          'son la misma persona, edita el contacto que ya existe en vez de ' +
          'crear uno nuevo.',
        salvo:
          'No se guardó nada: ni el contacto ni las direcciones que sí eran ' +
          'nuevas. El contacto que ya existía no se tocó.',
      },
      supresion_no_existe: {
        que: 'Esa baja ya no está en la lista.',
        como: 'Actualiza la lista para ver cómo está ahora.',
        salvo: 'No se modificó nada.',
      },
      supresion_no_se_levanta: {
        que: 'Esa persona pidió ella misma no recibir más, así que ARLES no la saca de la lista.',
        como: 'Si volvió a dar su permiso, dala de alta otra vez como contacto nuevo.',
        salvo: 'Sigue sin recibir nada. No se modificó nada.',
      },
      contacto_no_existe: {
        que: 'Ese contacto ya no está: alguien lo dio de baja.',
        como: 'Actualiza la lista de contactos para ver cómo está ahora.',
        salvo: 'No se modificó nada.',
      },
      cuenta_de_correo_no_existe: {
      que: 'Esa cuenta de correo ya no está.',
      como: 'Vuelve a la lista de remitentes y elige una de las que hay.',
      salvo: 'No se cambió nada y no se envió ningún correo.',
    },

    campana_no_existe: {
        que: 'Esa campaña ya no está.',
        como: 'Actualiza la lista de campañas para ver cómo está ahora.',
        salvo: 'No se modificó nada, y no salió ningún mensaje.',
      },
      dato_invalido: {
        que: 'Un dato guardado no tiene la forma que ARLES espera.',
        como:
          'Vuelve a guardar la configuración desde Ajustes. Si el problema ' +
          'persiste, restaura tu último respaldo.',
        salvo: 'No se modificó ni se borró nada.',
      },
    },
    app: {
      cuenta_de_correo_invalida: {
        que: 'Los datos de la cuenta de correo tienen algún problema.',
        como: 'Revisa los campos marcados en rojo y vuelve a intentarlo.',
        salvo: 'No se guardó nada y no se envió ningún correo.',
      },

      // ── Una cuenta guardada que ahora mismo no puede enviar ──
      //
      // Van separados de `cuenta_de_correo_invalida` a propósito: aquél es un
      // formulario mal rellenado, y éstos son cuentas ya guardadas. Y van
      // separados entre sí porque **cada uno pide una acción distinta**. Un
      // único «la cuenta no sirve» obligaría a probar los seis.
      //
      // Los seis dicen lo mismo en «qué está a salvo» (§95) y no es relleno: la
      // pregunta de quien lee esto en mitad de una campaña es siempre la misma
      // —¿salió algún correo?—, y la respuesta tiene que estar en pantalla y no
      // deducirse.
      cuenta: {
        no_existe: {
          que: 'La cuenta de correo que iba a enviar ya no está.',
          como: 'Ve a Remitentes y elige otra cuenta para esta campaña.',
          salvo: 'No se envió ningún correo.',
        },
        hay_que_configurarla: {
          que: 'Esa cuenta está pendiente de configurar.',
          como:
            'Ve a Remitentes, abre la cuenta y escribe su contraseña de ' +
            'aplicación. Al guardarla se prueba la conexión.',
          salvo: 'No se envió ningún correo.',
        },
        detenida_por_fallos: {
          que: 'ARLES detuvo esa cuenta tras varios fallos seguidos.',
          como:
            'Ve a Remitentes y pulsa «Probar» para ver qué está contestando el ' +
            'servidor. Si la prueba pasa, la cuenta vuelve a quedar lista.',
          salvo: 'No se envió ningún correo.',
        },
        desactivada: {
          que: 'Esa cuenta está desactivada.',
          como: 'Ve a Remitentes y actívala, o elige otra para esta campaña.',
          salvo: 'No se envió ningún correo.',
        },
        sin_contrasena: {
          que: 'No hay ninguna contraseña guardada para esa cuenta.',
          como:
            'Ve a Remitentes, abre la cuenta y vuelve a escribir su contraseña ' +
            'de aplicación. Se guarda en el llavero de tu sistema.',
          salvo: 'No se envió ningún correo, y no se intentó entrar a tu cuenta.',
        },
        // No es algo que el usuario pueda arreglar: es una base escrita por una
        // versión más nueva de ARLES, o una migración incompleta. El texto lo
        // dice así en vez de mandarle a tocar la cuenta, que no serviría.
        estado_desconocido: {
          que: 'Esa cuenta está en un estado que esta versión de ARLES no conoce.',
          como:
            'Suele significar que los datos vienen de una versión más nueva. ' +
            'Actualiza ARLES; si ya está actualizado, avisa a soporte.',
          salvo: 'No se envió ningún correo y no se modificó la cuenta.',
        },
      },

      llavero_no_disponible: {
        que: 'No se puede acceder al almacén de credenciales de tu sistema.',
        como:
          'Verifica que esté disponible y desbloqueado, y vuelve a abrir ARLES. ' +
          'Sin él, ARLES no puede descifrar tus datos.',
        salvo: 'Tus datos están intactos: la base de datos no se ha modificado.',
      },
      // Hallazgo F2: se distingue de «llavero no disponible» porque la acción
      // del usuario es completamente distinta. Allí hay que desbloquear el
      // llavero; aquí lo único que recupera los datos es un respaldo.
      clave_maestra_perdida: {
        que: 'La clave que cifraba tu base de datos ya no está en tu sistema.',
        como:
          'Restaura tu último respaldo .arles. Sin esa clave, los datos ' +
          'existentes no se pueden leer: es la propiedad que los protegía.',
        salvo: 'El archivo no se ha modificado ni borrado: sigue donde estaba.',
      },
      // No lo produce el núcleo: lo produce la interfaz cuando corre sin él
      // (`vite dev`, las sondas). Fingir un guardado ahí sería enseñar una
      // pantalla que miente sobre lo que acaba de pasar.
      sin_nucleo: {
        que: 'Esta ventana no está conectada al núcleo de ARLES.',
        como: 'Abre ARLES desde su acceso directo, no desde el navegador.',
        salvo: 'No se guardó ningún cambio.',
      },
      contacto_invalido: {
        que: 'Hay algo que revisar en los datos del contacto.',
        como:
          'Los campos con problema están marcados abajo, cada uno con lo que ' +
          'le falta. Corrígelos y vuelve a guardar.',
        salvo: 'No se guardó nada todavía. Lo que escribiste sigue en pantalla.',
      },
      sin_importacion_en_curso: {
        que: 'Se perdió el archivo que estabas importando.',
        como: 'Vuelve a elegirlo y a revisar el resumen antes de confirmar.',
        salvo:
          'No se importó nada. Tus contactos están exactamente como estaban.',
      },
      archivo_no_se_pudo_leer: {
        que: 'No se pudo abrir el archivo que elegiste.',
        como:
          'Comprueba que sigue donde estaba y que no lo tienes abierto en ' +
          'Excel. Si está en una carpeta compartida, cópialo a tu equipo ' +
          'primero.',
        salvo: 'No se importó nada.',
      },
      empresa_no_configurada: {
        que: 'Todavía no has configurado los datos de tu empresa.',
        como: 'Ve a Ajustes, completa los datos de la empresa y vuelve aquí.',
        salvo: 'No se modificó nada.',
      },
      empresa_invalida: {
        que: 'Algunos datos de la empresa no son válidos.',
        como: 'Revisa los campos marcados y vuelve a guardar.',
        salvo: 'No se guardó ningún cambio: la configuración anterior sigue activa.',
      },
      directorio_de_datos: {
        que: 'No se pudo acceder a la carpeta de datos de ARLES.',
        como:
          'Comprueba que tu usuario tiene permiso de escritura en la carpeta ' +
          'de datos de las aplicaciones.',
        salvo: 'No se ha creado ni modificado ningún archivo.',
      },
      sitio_no_abre: {
        que: 'No se pudo abrir tu navegador.',
        como: 'Escribe telemetrymx.com en la barra de direcciones.',
        salvo: 'No cambió nada en ARLES: sólo no se abrió la página.',
      },
      // No lo puede provocar el desplegable, que tiene tres opciones. Si
      // aparece, algo está hablando con ARLES por su cuenta.
      tema_desconocido: {
        que: 'No se pudo guardar el tema.',
        como: 'Vuelve a elegirlo en Ajustes › Apariencia.',
        salvo: 'La pantalla sigue con el tema que ves; sólo no se recordó.',
      },
    },
  },

  vacio: {
    campanas: {
      titulo: 'Aún no hay campañas.',
      cuerpo:
        'Una campaña envía un mensaje a una lista de contactos, respetando los ' +
        'límites que definas.',
      accion: 'Crear campaña',
    },
    supresiones: {
      titulo: 'No hay direcciones suprimidas.',
      cuerpo:
        'Las direcciones suprimidas quedan excluidas de todos los envíos futuros. ' +
        'Se añaden cuando alguien se da de baja, cuando un correo rebota de forma ' +
        'permanente, o cuando las excluyes a mano.',
      accion: 'Suprimir una dirección',
    },
  },

  contactos: {
    titulo: 'Contactos',
    // La cifra es una cifra, no un adjetivo (§94): dice cuántos se ven de
    // cuántos hay, en vez de «muchos» o «casi todos».
    recuento: 'Viendo {viendo} de {total}',
    agregar: 'Agregar contacto',
    // Aquí vivía `verSupresion`. Lo quitó Dirección el 22/09/2026: la barra
    // lateral ya lleva a las bajas y dos puertas al mismo cuarto acaban
    // divergiendo. Ver la nota de PantallaContactos.vue.
    importar: 'Cargar una tabla',
    editar: 'Editar',
    baja: 'Dar de baja',
    sinNombre: 'Sin nombre',
    tablaEtiqueta: 'Lista de contactos',

    columna: {
      nombre: 'Nombre',
      empresa: 'Empresa',
      email: 'Correo principal',
      whatsapp: 'WhatsApp principal',
    },

    canal: {
      email: 'Correo',
      whatsapp: 'WhatsApp',
    },

    campo: {
      nombre: 'Nombre',
      apellido: 'Apellido',
      empresa: 'Empresa',
      tipo: 'Tipo',
      valor: 'Dirección o número',
    },

    ayuda: {
      empresa: 'La empresa del contacto, no la tuya.',
      email: "Por ejemplo: ana{'@'}empresa.mx",
      // El «1» del móvil mexicano se explica AQUÍ y no en un error: es la
      // trampa que más veces se escribe mal, y decirlo después de rechazar el
      // dato llega tarde.
      whatsapp:
        'Con lada, por ejemplo 81 1234 5678. Si lo escribes con +52 1, ' +
        'ARLES guarda la forma internacional sin ese 1 — es la que reconoce ' +
        'WhatsApp.',
    },

    canales: {
      titulo: 'Formas de contacto',
      ayuda:
        'Un contacto puede tener varios correos y varios WhatsApp. El marcado ' +
        'como principal es al que se escribe por defecto.',
      anadir: 'Añadir otra',
      principal: 'Principal',
      tope: '{hay} de {max}',
      quitarDe: 'Quitar {valor}',
      // Sólo aparece cuando lo guardado no es lo escrito. Es el caso del móvil
      // mexicano con el «1»: se dice en vez de cambiarlo por detrás.
      seUsara: 'Se usará {valor}',
      sinValor: 'esta forma de contacto',
    },

    formulario: {
      introAlta:
        'Con una sola forma de contacto basta para empezar. Puedes añadir más ' +
        'ahora o más adelante.',
      introEdicion:
        'Los cambios sustituyen las formas de contacto anteriores. Las que ' +
        'quites dejan de usarse, pero se conserva a qué dirección se escribió ' +
        'en cada envío.',
      darDeAlta: 'Dar de alta',
    },

  supresion: {
      titulo: '{n} de estos contactos están en la lista de no escribir',
      // La frase «no impide guardarlo» es obligatoria: un aviso sin ella se
      // lee como «esto no se puede usar» (L-13).
      explicacion:
        'Estar en esa lista no impide guardarlos ni editarlos. Lo que hace es ' +
        'que las campañas los salten al enviar. Se entra en ella al darse de ' +
        'baja, al rebotar un correo de forma permanente, o a mano.',
      insignia: 'No escribir',
    },

    vacio: {
      titulo: 'Todavía no hay contactos.',
      cuerpo:
        'Un contacto es una persona con sus formas de contacto: uno o varios ' +
        'correos, uno o varios WhatsApp. Puedes cargar una tabla de Excel o ' +
        'CSV, o darlos de alta a mano de uno en uno.',
    },

    bajaDialogo: {
      titulo: 'Dar de baja el contacto',
      pregunta: '¿Dar de baja a {nombre}?',
      // La tercera parte del §95: qué está a salvo. Es lo que distingue esta
      // baja del borrado definitivo de la entrega 3.4.
      queSeConserva:
        'Deja de aparecer en la lista y las campañas dejan de escribirle. Se ' +
        'conserva el registro de a qué dirección se le escribió y cuándo, que ' +
        'es lo que permite responder a una reclamación. El borrado definitivo ' +
        'es otra cosa y llega en la entrega 3.4.',
      confirmar: 'Dar de baja',
    },

    error: {
      canalGenerico: 'Esta forma de contacto no es válida.',
      campoGenerico: 'Revisa este campo.',
    },
  },

  // ── Derechos de las personas · LFPDPPP ──────────────────────────────────
  //
  // Se escribe SIN la palabra «ARCO» en pantalla. Es la sigla de Acceso,
  // Rectificación, Cancelación y Oposición, y no se la sabe nadie que no
  // trabaje en protección de datos. Quien atiende la solicitud necesita
  // entender qué hace el botón, no aprobar un examen.
  derechos: {
    titulo: 'Derechos de las personas',
    explicacion:
      'Cuando alguien te escribe preguntando qué datos tienes de él, la ley ' +
      'te obliga a responder con todo lo que consta. Aquí se prepara esa ' +
      'respuesta en un archivo que puedes enviarle.',

    deQuienEsTitulo: '¿Quién responde, tú o ARLES?',
    // El responsable legal es el cliente (PRIVACIDAD_LFPDPPP.md §2). Se dice
    // sin rodeos: quien manda el archivo lo firma en nombre de su empresa.
    deQuienEs:
      'Respondes tú. ARLES no habla con esas personas ni guarda sus datos en ' +
      'ningún servidor: sólo reúne lo que hay en esta computadora para que ' +
      'puedas contestar sin pedirle ayuda a nadie.',

    canalEtiqueta: '¿Te escribió por correo o por WhatsApp?',
    canalAyuda: 'Se busca por la dirección exacta que te dio.',
    canal: {
      correo: 'Correo',
      whatsapp: 'WhatsApp',
    },
    direccionEtiqueta: 'Su dirección',
    // Se explica que mayúsculas y espacios dan igual, porque es la duda real
    // de quien copia y pega una dirección de un correo recibido.
    direccionAyuda:
      'Cópiala tal como te llegó. Las mayúsculas y los espacios de sobra no ' +
      'importan: ARLES la normaliza antes de buscar.',
    // `{'@'}` y no una arroba suelta: en la gramática de vue-i18n `@` abre un
    // enlace a otra clave, y el catálogo entero deja de compilar. Ya estaba
    // resuelto en `supresion.buscarMarcador`; aquí se repitió el error.
    marcadorCorreo: "ana{'@'}empresa.mx",
    marcadorWhatsApp: '+52 811 234 5678',
    entregar: 'Preparar el informe',

    // ── Los tres desenlaces buenos ──
    //
    // Los tres son un éxito. Lo que cambia es qué se encontró, y se dice:
    // entregar un informe vacío sin avisar dejaría a quien atiende creyendo
    // que la búsqueda falló.
    listo: 'Listo. Se guardó «{archivo}».',
    listoDetalle:
      'El archivo lleva todo lo que ARLES tiene de esa dirección: su ficha, ' +
      'de dónde salió, y los envíos que se le hicieron. Ya puedes enviárselo.',

    listoSoloRastro: 'Listo. Se guardó «{archivo}».',
    // §65 y honestidad: esa persona ya no es un contacto, pero queda rastro.
    // Decir «no hay nada» sería falso, y callarlo dejaría a quien atiende sin
    // entender por qué el informe no está vacío.
    listoSoloRastroDetalle:
      'Esa dirección ya no está dada de alta como contacto, pero sí queda ' +
      'rastro suyo: su baja, o los envíos que se le hicieron antes. El ' +
      'informe lo incluye, porque también es información sobre ella.',

    listoSinDatos: 'Listo. Se guardó «{archivo}», y va vacío.',
    // No es un error, y por eso NO se pinta en rojo. Puede ser exactamente la
    // respuesta que la persona quería.
    listoSinDatosDetalle:
      'No consta absolutamente nada de esa dirección, y el informe lo dice ' +
      'así. Envíaselo igual: «no tenemos nada suyo» es una respuesta válida, ' +
      'y muchas veces es la que quería oír.',

    cancelado: 'Cerraste la ventana sin guardar. No se preparó ningún archivo.',

    // ── Lo que puede salir mal (§95: qué pasó · cómo · qué está a salvo) ──
    sinNucleoQue: 'Esto es la vista previa, no ARLES.',
    sinNucleoComo:
      'Aquí no hay base de datos, así que no se puede preparar un informe ' +
      'de verdad. Ábrelo en la aplicación instalada.',
    sinNucleoSalvo: 'No se ha tocado ningún dato.',

    falloQue: 'No se pudo preparar el informe.',
    falloComo:
      'Vuelve a intentarlo. Si se repite, comprueba que tienes permiso para ' +
      'escribir en la carpeta que elegiste.',
    falloSalvo: 'No se cambió nada en tus contactos.',

    // ── Los otros tres derechos ──
    //
    // Se enseñan aunque dos ya estén hechos y uno no: quien atiende una
    // solicitud llega aquí sin saber cuál de los cuatro le pidieron, y
    // necesita que la pantalla le diga a dónde ir. Y el que falta se dice que
    // falta, en vez de callarlo.
    otrosTitulo: 'Si te pidieron otra cosa',
    rectificacion: 'Que corrijas un dato suyo',
    rectificacionDetalle:
      'Búscalo en Contactos y edítalo. No hace falta nada especial.',
    oposicion: 'Que dejes de escribirle',
    oposicionDetalle:
      'Dale de baja desde su ficha, o desde Bajas. Queda registrado y ARLES ' +
      'no vuelve a escribirle, ni aunque su dirección reaparezca en una ' +
      'importación futura.',
    // Aquí vivía `cancelacion`, que decía «todavía no está construido».
    // Ya está: es el bloque de abajo. Se quitó de esta lista porque enviar a
    // alguien a un sitio que está tres centímetros más abajo, en la misma
    // pantalla, es ruido.

    // ── Borrar de verdad · el derecho de Cancelación ────────────────────
    //
    // La única acción irreversible de toda la aplicación. Todo el texto de
    // este bloque está escrito para que nadie la pulse sin entenderla.
    borrarTitulo: 'Borrar los datos de alguien',
    borrarExplicacion:
      'Si alguien te pide que borres lo que tienes de él, aquí se hace. ' +
      'Primero te enseñamos qué va a desaparecer y qué se conserva; sólo ' +
      'después se puede borrar.',

    borrarAvisoTitulo: 'Esto no se puede deshacer',
    // §95: qué pasa, y qué está a salvo. Se dice exactamente qué sobrevive,
    // porque quien atiende la solicitud va a tener que explicárselo a alguien.
    borrarAviso:
      'La ficha de esa persona, sus correos, sus móviles y sus etiquetas ' +
      'desaparecen del disco. No hay papelera ni deshacer. Lo único que se ' +
      'conserva es el registro de los envíos que se le hicieron —sin su ' +
      'nombre, sin su dirección— y su baja, que es lo que impide volver a ' +
      'escribirle si su correo reaparece en otra lista.',

    borrarDireccionEtiqueta: 'La dirección que te pidió el borrado',
    borrarDireccionAyuda:
      'Cópiala tal como te llegó. Antes de borrar nada verás de quién es.',
    verQueSeBorraria: 'Ver qué se borraría',

    esteEs: 'Esto es lo que va a pasar',
    seBorra: 'Desaparece',
    seBorraDetalle: 'datos suyos, sin vuelta atrás',
    seConserva: 'Se conserva',
    seConservaDetalle: 'envíos, ya sin su nombre',
    bajaYaEstaba: 'Su baja ya está registrada y se queda.',
    bajaSeCrea:
      'No tenía baja: se le creará una al borrarlo, para que no vuelva a ' +
      'recibir nada si su dirección aparece en otra lista.',
    nadaQueBorrar:
      'No consta nada de esa dirección. Puedes borrar igual: quedará ' +
      'registrada su baja, para que siga sin constar nada.',

    // Escribir la dirección, no marcar una casilla. Una casilla se marca sin
    // leerla; escribir obliga a mirar a quién se está borrando.
    confirmarEtiqueta: 'Para confirmar, escribe {direccion}',
    confirmarAyuda:
      'No es un trámite: es para que vuelvas a leer a quién estás borrando.',
    borrarAhora: 'Borrar definitivamente',
    mejorNo: 'Mejor no',

    borradoTitulo: 'Borrado.',
    borradoDetalle:
      'Desaparecieron {borradas} datos suyos. Se conservan {conservados} ' +
      'envíos, ya sin su nombre.',
    borradoBajaYaEstaba: 'Su baja sigue registrada: no volverá a recibir nada.',
    borradoBajaCreada: 'Se le creó una baja: no volverá a recibir nada.',

    borrarFalloQue: 'No se pudo borrar.',
    borrarFalloComo:
      'Vuelve a intentarlo. Si se repite, cierra ARLES y ábrelo de nuevo.',
    // Lo más importante del error: que no quedó a medias.
    borrarFalloSalvo:
      'No se borró nada. O se borra todo o no se borra nada, así que sus ' +
      'datos siguen enteros.',
  },

  supresion: {
    titulo: 'Quién pidió no recibir',
    // Se explica de dónde sale la lista y para qué sirve. Una tabla de
    // direcciones sin contexto se lee como un error, no como un registro.
    explicacion:
      'Estas personas pidieron no recibir más, o su dirección dejó de funcionar. ARLES no les escribe, y se queda con el registro aunque borres su contacto: es lo que hace que la baja dure.',
    nota: '{cuantas} de {total}',
    cargando: 'Buscando las bajas…',
    vacioTitulo: 'Nadie ha pedido darse de baja',
    vacioCuerpo:
      'Aquí aparecerán quienes pidan no recibir más y las direcciones que dejen de funcionar. Por ahora no hay ninguna.',

    porCorreo: '{n} por correo',
    porWhatsApp: '{n} por WhatsApp',

    buscar: 'Buscar una dirección o un nombre',
    buscarMarcador: 'ana{\'@\'}empresa.mx',
    // §94: se dice sobre cuántas filas se está buscando de verdad, para que
    // nadie crea que ha buscado en la lista entera.
    filtroParcial: '{encontradas} de las {de} que se están viendo. La búsqueda mira esta página, no la lista completa.',

    canal: {
      correo: 'Correo',
      whatsapp: 'WhatsApp',
    },
    global: 'Todos sus canales',
    sinContacto: 'Sin contacto en ARLES',

    columna: {
      direccion: 'Dirección',
      deQuien: 'De quién',
      motivo: 'Por qué',
      fecha: 'Desde',
      acciones: 'Acciones',
    },

    motivo: {
      rebotePermanente: 'El correo rebotó de forma definitiva',
      baja: 'Pidió la baja',
      manual: 'Se añadió a mano',
      queja: 'Marcó el correo como no deseado',
      numeroInvalido: 'El número no existe o no tiene WhatsApp',
      falloDelCanal: 'El canal falló y no se va a arreglar reintentando',
      // Sólo aparece si una fila trae un motivo que ARLES no reconoce. Se dice
      // en vez de inventarse uno: esa persona sigue sin recibir igual.
      desconocido: 'Motivo no reconocido',
    },

    // Por qué esa fila no tiene botón. Un hueco sin explicación se lee como un
    // fallo de la aplicación, no como una decisión.
    //
    // Desde el 22/09/2026 este texto NO se pinta en cada fila: va en el
    // `title` del candado y en su texto para lectores de pantalla. Lo que se
    // lee a simple vista es `leyendaCandado`, una sola vez encima de la tabla.
    loPidioLaPersona: 'Lo pidió ella misma, así que ARLES no lo deshace',

    // La leyenda. Dice lo mismo en tercera persona y nombrando el símbolo,
    // porque se lee ANTES de encontrarse el primer candado y tiene que
    // enseñar a interpretarlo, no sólo repetir la regla.
    leyendaCandado:
      'El candado marca las bajas que pidió la persona. Ésas ARLES no las ' +
      'deshace: sólo se quitan si ella vuelve a dar su permiso.',
    volverAPermitir: 'Volver a permitir',
    confirmarLevantar:
      '¿Volver a permitir envíos a {direccion}?\n\nSi esa dirección vuelve a fallar, entrará otra vez en esta lista.',

    anteriores: 'Anteriores',
    siguientes: 'Siguientes',
  },

  importacion: {
    titulo: 'Cargar una tabla de contactos',
    // El número de fila es el de EXCEL: vas a abrir tu archivo para corregirlo,
    // y un número que no coincide con tu pantalla te hace contar a mano.
    fila: 'Fila {n}',

    paso: {
      elegir: 'Paso 1 de 4 · Elige el archivo',
      mapear: 'Paso 2 de 4 · Revisa qué es cada columna',
      revisar: 'Paso 3 de 4 · Mira qué va a pasar',
      hecho: 'Listo',
    },

    elegir: {
      titulo: 'Elige tu archivo de contactos.',
      cuerpo:
        'Un archivo de Excel (.xlsx) o un CSV. La primera fila tiene que ' +
        'decir qué es cada columna: «Nombre», «Correo», «Teléfono». ' +
        'Nada se guarda hasta que lo confirmes al final.',
      accion: 'Elegir archivo',
    },

    mapear: {
      recuento: '{filas} filas · {columnas} columnas',
      ayuda:
        'ARLES ya adivinó qué es cada columna. Revísalo y corrige lo que haga ' +
        'falta: debajo de cada una están sus primeros valores.',
      grupo: 'Qué es cada columna del archivo',
      sinEncabezado: 'Columna sin nombre',
      // Se dice ANTES de analizar. Dejarle ver un informe de cero filas sin
      // explicación es peor que avisarle aquí.
      sinCanales: 'Ninguna columna es un correo ni un WhatsApp.',
      sinCanalesDetalle:
        'Sin al menos una de las dos no hay a quién escribir, y no entraría ' +
        'ningún contacto. Marca la columna que tenga los correos o los móviles.',
      accion: 'Ver qué va a pasar',
    },

    revisar: {
      titulo: 'Esto es lo que va a pasar',
      // Cifras, no adjetivos (§94).
      cifras: 'Entran {entran} · se quedan fuera {fuera} · de {total} filas',
      choques: '{n} ya están en tus contactos',
      choquesDetalle:
        'No se van a duplicar. Si de verdad son personas distintas, corrige ' +
        'el archivo y vuelve a cargarlo.',
      yaEsDe: 'ya la tiene {quien}',
      repetidaEnElArchivo: 'repetida: ya estaba en la fila {fila}',
      fuera: '{n} no se pueden importar',
      volver: 'Volver a las columnas',
      accion: 'Importar {n} contactos',
    },

    hecho: {
      titulo: 'Importación terminada',
      importados: 'Se importaron {n} contactos.',
      cifras: 'Se importaron {importados} contactos de {total} filas.',
      choques: '{n} ya estaban en tus contactos y no se duplicaron.',
      fuera: '{n} no se pudieron importar.',
      accion: 'Ver mis contactos',
    },

    origen: {
      titulo: 'De dónde salió esta lista',
      campo: 'Origen de los contactos',
      porQue:
        'Queda registrado junto con la importación. Si algún día alguien ' +
        'reclama, poder decir de dónde salieron sus datos vale mucho más que ' +
        'una casilla marcada.',
      provisional: 'Este texto todavía no lo ha revisado un abogado.',
      provisionalDetalle:
        'Está pendiente la revisión jurídica (P-09). Se guarda íntegro y con ' +
        'su huella, así que cuando llegue el texto definitivo se puede ' +
        'sustituir sin perder nada de lo ya aceptado.',
      formularioPropio: 'Se dieron de alta en un formulario nuestro',
      clientesExistentes: 'Ya son clientes nuestros',
      eventoOFeria: 'Dejaron sus datos en un evento o feria',
      directorioPublico: 'Salieron de un directorio público',
      otro: 'Otro origen',
    },

    campo: {
      nombre: 'Nombre',
      apellido: 'Apellido',
      nombreCompleto: 'Nombre completo',
      empresa: 'Empresa',
      correo: 'Correo',
      whatsapp: 'WhatsApp',
      // «No importar» y no «ignorar»: describe lo que va a pasar con esa
      // columna, no un juicio sobre ella.
      ignorar: 'No importar',
    },

    rechazo: {
      sinCanales: 'No trae ni correo ni WhatsApp',
      demasiadosCanales: 'Trae más formas de contacto de las que caben',
      // Respaldo para un motivo que el núcleo emita y aquí no esté todavía.
      // Sin él, la pantalla pintaría la clave cruda en la lista de rechazos.
      otro: 'No se pudo leer esta fila.',
    },
  },

  comun: {
    cancelar: 'Cancelar',
    cargando: 'Cargando…',
    guardar: 'Guardar',
  },
  // ── Remitentes · entrega 5.3 ────────────────────────────────────────────
  //
  // «Remitente» y no «cuenta SMTP» en los títulos: quien usa ARLES sabe desde
  // qué dirección quiere escribir, y no tiene por qué saber qué es SMTP. Lo
  // técnico aparece donde hace falta —en el formulario, junto al campo— y no
  // en el nombre de la sección.
  remitentes: {
    titulo: 'Remitentes',
    explicacion:
      'Son las cuentas de correo desde las que ARLES envía. Necesitas al ' +
      'menos una para poder lanzar una campaña.',

    // §94: cifras, no adjetivos.
    nota: 'Tienes {cuantas} configuradas',
    notaConPendientes: '{cuantas} configuradas · {pendientes} por revisar',

    vacioTitulo: 'Todavía no hay ninguna cuenta',
    vacioCuerpo:
      'Añade la cuenta de correo desde la que quieres escribir. Si usas ' +
      'Gmail, hace falta una contraseña de aplicación: te explicamos cómo ' +
      'conseguirla en el formulario.',
    cargando: 'Buscando las cuentas configuradas…',

    anadir: 'Añadir remitente',
    editar: 'Editar',
    probarDeNuevo: 'Probar',
    cancelar: 'Cancelar',
    guardar: 'Guardar cuenta',
    guardando: 'Probando y guardando…',
    probar: 'Probar conexión',
    probando: 'Hablando con el servidor…',

    // ── El formulario ──
    //
    // El orden no es casual: primero de quién es el correo (lo que la persona
    // tiene claro), y después cómo se conecta (lo que hay que copiar de algún
    // sitio). Al revés, el primer campo del formulario sería el más técnico.
    formularioNuevo: 'Nueva cuenta de correo',
    formularioEditar: 'Editar la cuenta',

    remitente: 'Dirección desde la que se envía',
    remitenteAyuda: 'Es la que verá quien reciba el correo.',
    nombreVisible: 'Nombre que acompaña a la dirección',
    nombreVisibleAyuda: 'Opcional. Por ejemplo, el nombre de tu empresa.',
    usuario: 'Usuario',
    usuarioAyuda: 'Casi siempre es la misma dirección de correo.',
    contrasena: 'Contraseña',
    servidor: 'Servidor',
    servidorAyuda: 'Para Gmail, smtp.gmail.com.',
    puerto: 'Puerto',
    cifrado: 'Cifrado',

    // ── Lo que pasa con la contraseña ──
    //
    // Se dice en la pantalla, junto al campo, y no en un documento que nadie
    // abre: es la pregunta que se hace cualquiera al escribir la contraseña de
    // su correo en un programa que acaba de instalar.
    dondeVaLaContrasena:
      'La contraseña se guarda en el llavero de tu sistema, no en los ' +
      'archivos de ARLES. No se puede leer desde esta pantalla ni sale en ' +
      'ningún informe.',
    contrasenaGuardada:
      'La contraseña guardada se sigue usando. Escribe una nueva sólo si ' +
      'quieres cambiarla.',

    // ── La guía de Gmail ──
    //
    // El punto donde más gente se atasca (ABSTRACCION_EMAIL_PROVIDER.md §2).
    // Va plegada: quien ya la tiene no necesita leerla, y quien no, la abre.
    ayudaGmailTitulo: '¿Usas Gmail? Necesitas una contraseña de aplicación',
    ayudaGmailCuerpo:
      'Google no deja que otros programas entren con la contraseña normal de ' +
      'tu cuenta. Hay que generar una contraseña aparte, sólo para ARLES:',
    ayudaGmailPaso1:
      'Activa la verificación en dos pasos en tu cuenta de Google. Sin eso, ' +
      'la opción de contraseñas de aplicación no aparece.',
    ayudaGmailPaso2:
      'En la configuración de seguridad de Google, busca «Contraseñas de ' +
      'aplicaciones» y crea una nueva.',
    ayudaGmailPaso3:
      'Google te da dieciséis letras. Ésa es la que se escribe aquí, no la ' +
      'de tu cuenta.',
    ayudaGmailNota:
      'Si algún día quieres que ARLES deje de tener acceso, basta con borrar ' +
      'esa contraseña de aplicación desde tu cuenta de Google.',

    // ── El estado de cada cuenta ──
    estado: {
      connected: 'Lista para enviar',
      needs_reauth: 'Hay que configurarla',
      circuit_open: 'Detenida por fallos seguidos',
      disabled: 'Desactivada',
    },
    // §65 aplicado al alta: que la cuenta conecte no dice que un correo llegue.
    limites: 'Hasta {diario} al día · {porHora} por hora',

    seEstaProbando: 'Esto puede tardar unos segundos.',
  },

} as const
