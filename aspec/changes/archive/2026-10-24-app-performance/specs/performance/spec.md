## ADDED Requirements

### Requirement: Comandos IPC no bloqueantes

The system SHALL ejecutar fuera del hilo principal todo comando que inicie procesos,
use la red o realice IO de disco sobre un conjunto de archivos.

#### Scenario: Abrir la pestaña Uso no congela la ventana

- **WHEN** el usuario abre la pestaña Uso mientras el reporte de proveedores está en curso
- **THEN** la ventana sigue procesando eventos y repintando
- **AND** ningún frame supera 100 ms durante el refresco

#### Scenario: Cambiar de panel durante una operación de Git

- **WHEN** el usuario cambia de panel mientras se ejecuta `git_status` o `git_diff`
- **THEN** la interfaz responde a la navegación antes de que la operación termine

### Requirement: Reporte de uso con frescura explícita

The system SHALL servir el último reporte de uso disponible y SHALL refrescarlo en
segundo plano cuando esté vencido, sin bloquear la vista.

#### Scenario: Reporte en caché vigente

- **WHEN** el usuario abre la pestaña Uso y el último reporte tiene menos de 5 minutos
- **THEN** las tarjetas se muestran desde la caché en menos de 100 ms
- **AND** no se inicia ninguna consulta de red ni proceso

#### Scenario: Reporte en caché vencido

- **WHEN** el usuario abre la pestaña Uso y el último reporte tiene más de 5 minutos
- **THEN** se muestran los datos anteriores marcados como vencidos
- **AND** se inicia un único refresco en segundo plano

#### Scenario: Refresco forzado

- **WHEN** el usuario presiona "Actualizar"
- **THEN** se inicia un refresco aunque el reporte esté vigente
- **AND** el botón queda deshabilitado hasta que el refresco termina

#### Scenario: Reporte inexistente

- **WHEN** el usuario abre la pestaña Uso por primera vez
- **THEN** se muestran skeletons por proveedor
- **AND** cada tarjeta aparece en cuanto su proveedor responde

### Requirement: Progreso por proveedor

The system SHALL publicar el resultado de cada proveedor de uso de forma
independiente, sin esperar a los demás.

#### Scenario: Un proveedor lento no retrasa al resto

- **WHEN** un proveedor tarda más que el presupuesto configurado
- **THEN** los demás proveedores ya se muestran con sus datos
- **AND** el proveedor lento termina en estado `error` con un mensaje accionable

#### Scenario: Evento de progreso perdido

- **WHEN** la vista se desmonta y vuelve a montarse durante un refresco
- **THEN** el estado se reconstruye desde el snapshot devuelto por el comando
- **AND** no se duplican tarjetas ni se pierden proveedores

### Requirement: Presupuesto de latencia del reporte

The system SHALL acotar el tiempo de cada proveedor y del refresco completo.

#### Scenario: Timeout HTTP

- **WHEN** un endpoint de uso no responde dentro de 5 segundos
- **THEN** ese proveedor se marca como `error` o `unavailable`
- **AND** el refresco completo continúa con los demás proveedores

#### Scenario: Proceso colgado

- **WHEN** `opencode stats` o `agy -p /usage` excede 14 segundos
- **THEN** el proceso se termina
- **AND** el proveedor reporta estado `error` sin bloquear el refresco

#### Scenario: Refresco concurrente

- **WHEN** dos invocaciones de `usage_report` ocurren mientras hay un refresco activo
- **THEN** se ejecuta un solo refresco
- **AND** ambas invocaciones reciben el mismo snapshot

### Requirement: Caché de uso sin secretos

The system SHALL persistir únicamente datos derivados del reporte de uso.

#### Scenario: Inspección de la caché

- **WHEN** se inspecciona `usage-cache.json` después de un refresco exitoso
- **THEN** el archivo no contiene claves de API, tokens ni encabezados de autorización
- **AND** en Unix sus permisos son `0600`

#### Scenario: Caché corrupta

- **WHEN** el archivo de caché está truncado o tiene una versión desconocida
- **THEN** el sistema lo ignora y se comporta como si no hubiera caché
- **AND** no falla la apertura de la pestaña Uso

### Requirement: Presupuesto de arranque y bundle

The system SHALL cargar de forma diferida el código que no es necesario para el
primer render.

#### Scenario: Arranque sin matemática

- **WHEN** la app arranca y el usuario no abre una vista con fórmulas
- **THEN** no se descargan las fuentes ni el CSS de KaTeX
- **AND** el chunk inicial de JavaScript no supera 180 KB gzip

#### Scenario: Terminal diferido

- **WHEN** el usuario abre la app sin sesiones activas
- **THEN** el código de xterm no forma parte del chunk inicial

#### Scenario: Chequeo de actualizaciones diferido

- **WHEN** la app completa el primer render
- **THEN** el chequeo de actualizaciones se ejecuta en tiempo ocioso
- **AND** no retrasa la primera pintura
