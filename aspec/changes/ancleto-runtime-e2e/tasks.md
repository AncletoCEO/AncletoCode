# Tasks: E2E y rollback del runtime Ancleto embebido

## Fase 1 — Estado y resolución

- [ ] 1.1 Unificar la resolución de runtime (incluida/activa/Node) para app y CLI, con estado tipado y mensajes accionables.
- [ ] 1.2 Mostrar en Ajustes versión activa, incluida, disponibilidad de update y error.

## Fase 2 — Update y rollback

- [ ] 2.1 Implementar activación atómica del update (temp + verificación + rename) sin escribir sobre la versión activa.
- [ ] 2.2 Implementar rollback a la versión anterior ante fallo de activación o de primer uso.
- [ ] 2.3 Rechazar checksum inválido, tarball corrupto y versión incompatible conservando la versión activa.

## Fase 3 — E2E

- [ ] 3.1 Fixtures de registro npm local: paquete bueno, corrupto y con checksum inválido.
- [ ] 3.2 E2E de instalación, update exitoso, update fallido, red caída y rollback.
- [ ] 3.3 Verificar que un fallo del runtime no impide abrir proyectos ni ejecutar `spec_status`.
