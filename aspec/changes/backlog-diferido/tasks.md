# Tasks: Backlog diferido (CLI, AncletoCMD, npm y hosts Git alternativos)

> Nada de este change se ejecuta hasta que se apruebe su reapertura.

## CLI headless

- [ ] 1.1 (era 12.11) Windows: argv sin shell intermedia, VT/UTF-8 cuando esté disponible y pruebas en `cmd.exe` y PowerShell.

## AncletoCMD

- [ ] 2.1 (era 13.14) Tests de conformidad VT con fixtures y programas reales (vim, tmux, htop, git, less), y smoke tests en Windows (`cmd`, PowerShell, WSL), Linux y macOS.

## Publicación npm

- [ ] 3.1 (era 14.3) Publicar paquetes npm por plataforma con `checksums.json`: CLI `acode` y AncletoCMD `acmd`.
- [ ] 3.2 (era 14.4) Matriz npm en runners nativos y `npm publish --provenance`.
- [ ] 3.3 (era 14.8) Smoke test de `npm i -g @ancleto/code` y `npm i -g @ancleto/cmd`.

## Hosts Git alternativos

- [ ] 4.1 (era 16.14) GitLab con `glab`.
- [ ] 4.2 (era 16.15) Bitbucket/Azure/remoto genérico.
