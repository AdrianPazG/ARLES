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
      dato_invalido: {
        que: 'Un dato guardado no tiene la forma que ARLES espera.',
        como:
          'Vuelve a guardar la configuración desde Ajustes. Si el problema ' +
          'persiste, restaura tu último respaldo.',
        salvo: 'No se modificó ni se borró nada.',
      },
    },
    app: {
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
    cancelacion: 'Que borres sus datos',
    // §94 y honestidad: se dice qué falta, no «pronto».
    cancelacionDetalle:
      'Todavía no está construido. Llega en esta misma entrega, junto con el ' +
      'borrado de verdad. Su baja se conserva siempre, aunque se borre todo ' +
      'lo demás: es lo que impide volver a escribirle por error.',
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
} as const
