# ADR-029 — Windows Runtime Context Join Probe

Fecha: 2026-09-30 local / 2026-10-01 UTC. Estado: aceptada para laboratorio; join de juego sin decisión.

## Decisión

Usar un evento Windows Local con nonce para medir visibilidad de objetos entre dos ejecutables LXMI en contextos Proton privados. Reutilizar el executor de lxmi-bridge, mapping de lxmi-xxmi y discovery existente. No hay nuevo injector/framework de ejecución ni IPC persistente.

El evento exige menos superficie que enumerar/abrir procesos o un pipe/server permanente. A mantiene el handle y B lo abre/señala; ambos verifican un marker independiente mediante mapping observado. El control cambia solo el prefix, conservando la tool. Un filesystem compartido por Z no acredita namespace de eventos común.

## Evidencia y consecuencia

HOST TEST: mismo prefix permitió el evento; prefixes distintos no lo hicieron. Se distinguen mappings completos de mapping del marker. Prefix reportado, tool explícita, candidato wineserver y namespace/session identity no se colapsan. El candidato wineserver no se convierte en endpoint probado.

`ContextJoinRequirements` mantiene NotEstablished para un loader de juego. `GameContextJoinPlan` es datos y no tiene executor. NoDecision de ADR-027, Hook guard y gate se conservan. La decisión no autoriza No Reload/IPC, shared game prefix, loaders o ZZZ.

## Seguridad y lifecycle

Solo dos ejecutables y paths construidos por backend, bajo LXMI; hashes y symlinks validados. La UI solo puede escoger un Proton descubierto y un modo enumerado. Peers no enumeran procesos ni abren process handles. Native executor usa argv, límites, timeout y stdout/stderr no bloqueantes. Context Lab no envía señales a grupos ni wineserver; termina solamente su launcher hijo en timeout, y los peers expiran por sí mismos. Se permite salida natural de servidores privados; sus PIDs son efímeros.

## Referencias

- [Investigación](../../01_Descubrimiento/contexto_wine_proton_0_8_4.md)
- [Verificación](../../06_Verificacion/verificacion_0_8_4.md)
- [ADR-028](028_steam_proton_context_evidence.md)
