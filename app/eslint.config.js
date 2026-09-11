import js from '@eslint/js'
import vue from 'eslint-plugin-vue'
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
    },
  },
)
