# Design: E2E y rollback del runtime Ancleto embebido

## Approach

El runtime embebido se trata como un artefacto versionado con activación atómica: se
descarga/verifica en un directorio temporal y se activa solo si pasa integridad y
compatibilidad. El E2E simula el registro npm con fixtures locales para no depender de
la red.

```
versión incluida ──► resolución ──► [update disponible]
                                        │
                              descarga + checksum
                                        │
                        válido ──► activar (rename atómico)
                        inválido ──► conservar activa + error
```

## Architecture

- **Resolución**: el módulo `ancleto_runtime` devuelve versión activa, incluida, path de
  Node y error; app y CLI consumen el mismo resultado.
- **Update**: descarga a temp, verifica SHA-512 e integridad del tarball, extrae y
  activa con rename atómico. Nunca se escribe sobre la versión activa.
- **Rollback**: si la activación o el primer uso falla, se restaura el puntero a la
  versión anterior; el estado queda visible en Ajustes.
- **E2E con fixtures**: un registro npm falso local sirve paquetes buenos, corruptos y
  con checksum inválido. Los tests cubren instalación, update, fallo y rollback.
- **Degradación**: sin red, el update se reporta como no disponible y la versión
  incluida sigue operativa.

## Validation

```bash
cargo test --manifest-path src-tauri/Cargo.toml ancleto_runtime
npm run test
npm run check:commands
```

- E2E: versión incluida, update exitoso, checksum inválido, tarball corrupto, red
  caída y rollback.
- Verificar que un fallo no impide abrir proyectos ni ejecutar `spec_status`.
- Verificar que Ajustes muestra versión activa/incluida y el error.
