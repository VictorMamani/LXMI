# ADR-024: Confianza de paquetes oficiales upstream

**Estado:** Aceptado para LXMI 0.5.2  
**Fecha:** 2026-09-28

## Contexto

LXMI 0.5.1 solo importaba carpetas locales, así que un SHA-256 recién calculado describía el contenido copiado, pero no demostraba su origen. ZZMI y XXMI Libraries publican releases con firmas verificadas por XXMI Launcher. GitHub API también publica un digest por asset. Son fuentes de evidencia distintas.

## Decisión

- Solo consultar los repositorios fijados en código: `leotorrez/ZZMI-Package` y `SpectrumQT/XXMI-Libs-Package`.
- Guardar en el manifiesto LXMI la release concreta: repositorio, release ID, tag, commit resuelto, asset, URL, fechas de publicación/consulta/descarga, firma declarada, SHA-256 de descarga y SHA-256 API cuando exista.
- Verificar con las claves públicas fijadas del commit inspeccionado de XXMI Launcher, reproduciendo ECDSA/SHA-256 sobre los bytes exactos del asset. Las claves DER observadas son secp384r1 (P-384).
- En Libraries, exigir `Manifest.json` y verificar con la clave upstream las firmas declaradas para cada DLL esperada. El manifest separado se coteja además con la metadata HTTPS/API; su documento no tiene firma separada.
- Mantener `SHA-256` como integridad de bytes y `Verified` de la firma como autenticidad respecto a la clave fijada. Ninguna implica compatibilidad del juego/plataforma.
- Rechazar descarga/importación oficial si falta o falla una firma obligatoria. La importación de carpeta local permanece disponible con origen explícitamente no autenticado.
- No copiar ni redistribuir los packages desde LXMI. El upstream recomienda instalar mediante XXMI Launcher; LXMI 0.5.2 termina en storage y dry-run, sin `apply`.

## Consecuencias

- Red solo bajo una acción iniciada por el usuario; pruebas unitarias offline usan `FixtureReleaseProvider`.
- Redirects manuales limitados a HTTPS y hosts GitHub permitidos; asset, tamaño, archivo, firma y rutas ZIP se validan antes de promoción.
- ZIP extraído primero a staging privado LXMI; solo después de validar tipo, estructura, inventario, checksums y firmas se promueve a managed storage.
- Los assets en el directorio de juego se inspeccionan read-only como candidato. El `importer_path` real de XXMI es configurable, así que LXMI lo deja sin resolver y bloquea apply.
- Licencias por componente y compatibilidad Steam/Linux/Proton quedan pendientes; procedencia criptográfica no resuelve esas preguntas.

## Fuentes upstream fijadas

- [ZZMI v1.4.5](https://github.com/leotorrez/ZZMI-Package/releases/tag/v1.4.5)
- [XXMI Libraries v1.1.7](https://github.com/SpectrumQT/XXMI-Libs-Package/releases/tag/v1.1.7)
- [Verificador en XXMI Launcher, commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a`](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/utils/security.py)
- Captura, IDs, digests, manifiestos, layout y límites: `SDLC/01_Descubrimiento/adquisicion_paquetes_xxmi_0_5_2.md`.
