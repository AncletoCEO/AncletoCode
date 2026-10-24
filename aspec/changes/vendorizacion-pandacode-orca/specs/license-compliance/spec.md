## ADDED Requirements

### Requirement: Auditoría de referencia Orca

El sistema SHALL registrar si un módulo de Orca fue usado como referencia o
reutilizado, antes de incorporar código o activos.

#### Scenario: Referencia sin copia

- **WHEN** un módulo de Orca solo inspira el diseño
- **THEN** `docs/provenance.md` registra "referencia sin copia" con módulo y fecha
- **AND** no se agrega código ni activos de Orca al repositorio

#### Scenario: Reutilización de código MIT

- **WHEN** se copia o adapta código de Orca
- **THEN** el archivo lleva el aviso de Lovecast Inc. y su licencia MIT
- **AND** `THIRD_PARTY_NOTICES.md` y el ledger quedan actualizados
- **AND** se portan sus tests

### Requirement: Cobertura de tests de módulos vendorizados

El sistema SHALL ejecutar en CI los tests de cada módulo vendorizado o adaptado.

#### Scenario: Módulo adaptado

- **WHEN** un módulo vendorizado se modifica respecto del upstream
- **THEN** sus tests se adaptan y ejecutan en la suite del proyecto
- **AND** el resumen del diff queda registrado en `docs/provenance.md`

#### Scenario: Módulo sin tests upstream

- **WHEN** el módulo upstream no trae tests
- **THEN** se agregan tests propios que cubran el comportamiento adaptado
- **AND** la excepción queda documentada en el ledger
