# Matriz inicial de riesgos

Registro preliminar; probabilidad y controles todavía no se estimaron con evidencia.

| ID | Riesgo | Impacto potencial | Acción antes de avanzar | Estado |
|---|---|---|---|---|
| R-01 | El juego o runtime no admite el flujo Wine/Proton propuesto | No se puede cumplir el objetivo central o se afecta una instalación/cuenta | Confirmar soporte upstream y políticas; usar laboratorio aislado y detenerse si no hay una ruta autorizada | Abierto |
| R-02 | La licencia de un runtime/dependencia no permite el uso o distribución planeados | Incumplimiento de obligaciones o necesidad de cambiar diseño | Identificar versiones y licencias; revisar dependencias y obligaciones antes de distribuir/forkear | Abierto |
| R-03 | Archivo de mod malicioso o con rutas diseñadas para escapar del directorio | Escritura/borrado fuera de la biblioteca o ejecución no deseada | Parser de archive seguro, límites, staging aislado, rechazo de traversal/enlaces y pruebas adversariales | Abierto |
| R-04 | Detección de rutas/versiones incorrecta por diferencias entre instalaciones | Cambio de configuración o lanzamiento en destino equivocado | Empezar con inspección de solo lectura, mostrar fuente/confianza y permitir confirmación manual | Abierto |
| R-05 | Configuración del runtime se corrompe durante instalación/actualización | Juego deja de iniciar o mods se pierden | Backups verificados, journal de operación y restauración ensayada antes de habilitar escritura | Abierto |
| R-06 | Symlinks u otro método de activación no es compatible con todas las rutas/runtime | Mod no aparece o operaciones afectan destino inesperado | Probar método con fixture y cada entorno soportado; no asumirlo como diseño cerrado | Abierto |
| R-07 | La función live requiere cambios upstream o no tiene un mecanismo soportado | Gran sobrecosto, mantenimiento de fork o imposibilidad técnica | Mantener live/IPC como investigación separada; no bloquear el primer MVP con ella | Abierto |
| R-08 | El alcance crece a muchos juegos, launchers y distribuciones | Retraso y matriz de soporte imposible de verificar | Limitar inicialmente a una combinación solo si fuentes y pruebas la justifican | Abierto |
| R-09 | Empaquetado incluye dependencias o recursos con obligaciones no identificadas | Release no se puede publicar como se planeó | Revisar SBOM/licencias y proceso de distribución antes de generar release | Abierto |

No es una evaluación de seguridad, legal ni de compatibilidad completada. Actualizar cada riesgo con evidencia, responsable, probabilidad y tratamiento al iniciar la prueba técnica.
