# ADR-023: Compatibilidad y autenticidad son dimensiones separadas

**Estado:** Aceptado para LXMI 0.5.1
**Fecha:** 2026-09-28

## Contexto

ZZMI publica metadata de versión y el launcher upstream mantiene metadata/verificación de firma. LXMI puede calcular SHA-256 del payload importado, pero ese hash solo permite comparar contenido con el inventario que LXMI creó. No demuestra quién publicó el contenido. De igual modo, que una carpeta cumpla el contrato estructural no demuestra que funcione con Steam, Linux o Proton.

## Decisión

LXMI representa de forma separada:

- integridad local por SHA-256;
- autenticidad/origen upstream;
- juego al que corresponde la integración;
- dependencia del paquete XXMI Libraries;
- soporte de Steam;
- compatibilidad Linux/Proton;
- posibilidad de generar un plan declarativo.

En 0.5.1, autenticidad de paquete y compatibilidad Steam/Linux/Proton quedan sin verificar. La configuración upstream de ZZMI indica que `XXMI Libraries` es una dependencia aparte. LXMI no implementa su verificación de firma ni convierte un plan generable en autorización para aplicarlo.

## Consecuencias

- No etiquetar un package como oficial/auténtico a partir de SHA-256.
- No mostrar `Compatible` por el mero hecho de detectar ZZZ, ZZMI o Proton.
- El plan puede ser técnicamente descriptivo y a la vez bloquearse como no verificado para la plataforma.
- La autenticación criptográfica y la comprobación de plataforma requieren decisiones y pruebas posteriores.

## Evidencia

La dependencia y metadata del paquete se revisaron en [la configuración ZZMI del launcher](https://github.com/SpectrumQT/XXMI-Launcher/blob/main/src/xxmi_launcher/core/packages/model_importers/zzmi_package.py), [el repositorio de ZZMI](https://github.com/leotorrez/ZZMI-Package) y [la solicitud upstream de soporte Steam/Linux](https://github.com/SpectrumQT/XXMI-Launcher/issues/318), consultados el 2026-09-28.
