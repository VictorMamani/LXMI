# ADR-021 — Origen de paquetes y redistribución

Estado: adoptado para 0.5.

Se elige **instalación externa + importación local** de carpetas de paquetes. El usuario proporciona WWMI y, por separado, XXMI Libraries con el Manifest oficial. LXMI comprueba su estructura y guarda hashes; las firmas declaradas no se verifican en 0.5.

El modelo `PackageSource` conserva espacio para OfficialRelease, LocalArchive y ManagedCache. No implica que existan descargas o extracción implementadas. Solo LocalDirectory se crea en el flujo actual. Oficialidad no se infiere del nombre de una carpeta ni de campos autoafirmados.

No redistribuir DLLs/recursos como parte de LXMI. Las licencias consultadas y los componentes aún no resueltos constan en [la investigación](../../01_Descubrimiento/ecosistema_xxmi_wwmi.md). Una futura descarga upstream requiere una decisión separada sobre firmas, origen, licencias y actualizaciones; no se implementa ahora.
