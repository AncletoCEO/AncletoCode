# Paquetes AUR (generados)

Los `PKGBUILD` de Arch se generan en cada release con `scripts/package-aur.mjs` a
partir de los tarballs de Linux publicados en GitHub Releases. El release adjunta
`ancleto-code-bin`, `ancleto-cmd-bin` y `ancleto-code-cli-bin` con checksums
reales.

- `ancleto-code-bin`: app de escritorio; depende de `webkit2gtk-4.1` y del host.
- `ancleto-cmd-bin`: AncletoCMD; depende del host y de las librerías gráficas.
- `ancleto-code-cli-bin`: CLI y host estáticos (musl), sin dependencias de runtime.

Para publicar en AUR, copiar el `PKGBUILD` generado a un clon AUR y ejecutar
`makepkg --printsrcinfo > .SRCINFO` antes de hacer push. Los paquetes `-bin`
apuntan al tag `v${pkgver}` de GitHub Releases.
