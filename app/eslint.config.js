import js from '@eslint/js'
import vue from 'eslint-plugin-vue'
import globals from 'globals'
import ts from 'typescript-eslint'

export default ts.config(
  { ignores: ['dist/**', 'node_modules/**'] },

  js.configs.recommended,
  ...ts.configs.recommended,
  ...vue.configs['flat/recommended'],

  {
    files: ['**/*.vue'],
    languageOptions: {
      parserOptions: { parser: ts.parser },
    },
  },

  {
    // El código de la webview corre en un navegador. Sin declararlo, `no-undef`
    // marca como indefinidos `document`, `KeyboardEvent` o `setTimeout`, que es
    // lo que tienen que usar los componentes que gestionan foco y teclado.
    //
    // Lo que NO entra por aquí: `fetch` y `localStorage` siguen prohibidos por
    // las reglas de más abajo, que se comprueban ejecutando eslint contra un
    // archivo cebo en el validador de fase.
    languageOptions: {
      globals: globals.browser,
    },
  },

  {
    // Las sondas de `pruebas/` son scripts de Node que manejan un navegador,
    // no código de la webview: corren en la máquina de quien desarrolla y
    // nunca se empaquetan. Necesitan `process` y escriben su informe por
    // consola, que es su única salida.
    //
    // El bloque va ANTES de las reglas generales para que éstas sigan
    // aplicándose a `pruebas/`: aquí sólo se añaden globales y se permite la
    // consola. Las prohibiciones de `fetch` y `localStorage` siguen vigentes.
    files: ['pruebas/**/*.mjs'],
    languageOptions: {
      globals: globals.node,
      sourceType: 'module',
    },
  },

  {
    rules: {
      // La webview es la superficie con más exposición a contenido no
      // confiable: `any` desactiva ahí justo las comprobaciones que importan.
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_' },
      ],

      // Regla de frontera 3.1: la webview no decide nada de negocio, así que
      // tampoco debe hablar con el exterior por su cuenta. Todo I/O pasa por
      // comandos de Tauri, donde la autoridad vive en Rust.
      'no-restricted-globals': [
        'error',
        { name: 'fetch', message: 'Usa un comando de Tauri: el I/O vive en Rust.' },
      ],
      'no-restricted-properties': [
        'error',
        {
          object: 'window',
          property: 'fetch',
          message: 'Usa un comando de Tauri: el I/O vive en Rust.',
        },
      ],

      // §30: ninguna credencial toca el almacenamiento del navegador.
      'no-restricted-syntax': [
        'error',
        {
          selector:
            "MemberExpression[object.name=/^(localStorage|sessionStorage)$/]",
          message:
            'Prohibido el almacenamiento del navegador (§30). Los secretos van al llavero del SO.',
        },
      ],

      'no-console': ['warn', { allow: ['warn', 'error'] }],

      // Una prop opcional de TypeScript YA declara que puede faltar, y con
      // `exactOptionalPropertyTypes` un default `undefined` ni siquiera
      // compila. La regla está pensada para props declaradas en tiempo de
      // ejecución, donde el tipo no dice nada.
      'vue/require-default-prop': 'off',
    },
  },

  {
    // Va al final para que gane sobre el `no-console` general: el informe de
    // una sonda ES su salida. Sólo afecta a `pruebas/`.
    files: ['pruebas/**/*.mjs'],
    rules: { 'no-console': 'off' },
  },
)
