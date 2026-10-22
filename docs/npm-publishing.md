# Publicación en npm

## Scope

Los paquetes usan el scope **`@ancleto`**, que ya existe en npm
(`@ancleto/spec`, `@ancleto/writer`). Para publicar hay que ser **miembro del
scope** `@ancleto` (organización o usuario) y tener permisos de publicación.

Comprobar después de autenticarse:

```bash
npm login
npm whoami
npm org ls ancleto
```

Si no tenés acceso al scope, pedí que te agreguen o cambiá el scope en
`packages/*/package.json`, `scripts/package-npm.mjs`, `scripts/package-aur.mjs`,
launchers y documentación.

## Paquetes

| Paquete | Contenido |
|---|---|
| `@ancleto/code` | meta paquete de la CLI + host (`acode`, `ancleto-code`) |
| `@ancleto/cmd` | meta paquete de AncletoCMD (`acmd`, `ancleto-cmd`) |
| `@ancleto/code-<plataforma>` | binarios por plataforma (`acode` + `ancleto-code-host`) |
| `@ancleto/cmd-<plataforma>` | binarios por plataforma (`acmd` + host) |

Plataformas: `linux-x64-gnu`, `linux-arm64-gnu`, `linux-x64-musl`,
`linux-arm64-musl` (sólo code), `win32-x64`, `win32-arm64`, `darwin-x64`,
`darwin-arm64`.

## Orden de publicación

1. Construir y publicar **primero los paquetes por plataforma**.
2. Publicar **después** los meta paquetes `@ancleto/code` y `@ancleto/cmd`.
   Sus `optionalDependencies` apuntan a los paquetes por plataforma de la misma
   versión.

En CI esto lo hace `.github/workflows/release.yml`: matriz nativa → platform
packages → meta paquetes.

## Publicación manual (Linux x64, ejemplo)

```bash
npm ci
npm run check:versions

# CLI + host
cargo build --release -p ancleto-code-cli -p ancleto-code-host
node scripts/package-npm.mjs --binary-dir target/release \
  --platform linux-x64-gnu --product code
npm publish packages/code-linux-x64-gnu --access public
npm publish packages/acode --access public

# AncletoCMD
cargo build --release -p ancleto-code-cmd
node scripts/package-npm.mjs --binary-dir target/release \
  --platform linux-x64-gnu --product cmd
npm publish packages/cmd-linux-x64-gnu --access public
npm publish packages/acmd --access public
```

Para una release completa, usar CI: cada plataforma se publica desde su runner
nativo y sólo después se publican los meta paquetes.

## Autenticación para CI

- Crear un token de npm con permiso de publicación y 2FA resuelto
  (Automation/Granular token) y guardarlo como secreto `NPM_TOKEN`.
- El workflow usa `npm publish --provenance --access public`.
- `--provenance` requiere `id-token: write` en GitHub Actions (ya configurado) y
  que el repositorio sea público.
- Alternativa: **Trusted Publishing** de npm con OIDC, configurando cada paquete
  para confiar en el workflow; en ese caso no hace falta `NPM_TOKEN`.

## Verificación

```bash
npm view @ancleto/code@0.0.1
npm view @ancleto/cmd@0.0.1
npm install -g @ancleto/code@0.0.1
acode --version
acode doctor
npm install -g @ancleto/cmd@0.0.1
acmd --version
```

## Reglas

- Una versión publicada es **inmutable**: si hay que corregir, subir a `0.0.2`.
- No publicar `0.0.1` dos veces.
- Verificar antes `npm pack --dry-run` en cada paquete y `npm run check:versions`.
- Los paquetes de plataforma no se commitean: se generan en CI con
  `scripts/package-npm.mjs`.
