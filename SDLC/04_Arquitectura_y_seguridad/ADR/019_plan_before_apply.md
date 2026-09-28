# ADR-019 — Plan before apply

Estado: adoptado en LXMI 0.5.

## Decisión

Separar discovery, assessment, plan e importación a staging. El dominio no ofrece executor, apply ni instalación al juego. `InstallationPlan` enumera fuentes, targets administrados, hashes, Create/Unchanged/ReplaceWithBackup, hashes previos y requisitos pendientes. `executable` siempre es false en 0.5.

Un paquete validado no demuestra compatibilidad. El planner mantiene la selección de Proton del plan de plataforma, incluida Unknown. WriteAccess y LaunchCompatibility permanecen desconocidos. La presencia de un directorio del juego no valida su completitud.

## Motivo y alternativas

Copiar directamente al juego antes de revisar rutas y requisitos haría difícil auditar o revertir cambios. Generar scripts instalables ahora también permitiría saltarse la validación futura. Se eligieron datos revisables y sin capacidad de ejecución.

Upstream administra WWMI fuera del juego. Por ello el destino propuesto es `LXMI/runtimes/wwmi`; no se inventa una copia de DLL al game folder como requisito confirmado de Linux. La estrategia de activación seguirá abierta hasta investigarse en 0.6.

## Consecuencias

0.6 debe volver a verificar fuentes y destinos, comprobar compatibilidad, construir backups, requerir aprobación del plan concreto y probar apply transaccional/rollback. El plan 0.5 es efímero: no es una autorización persistente ni una garantía de que nada cambió desde su generación.
