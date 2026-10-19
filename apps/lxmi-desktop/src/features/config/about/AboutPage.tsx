import { version } from "../../../../package.json";
import { isDemoMode } from "../../../api/client";
import { StatusDot } from "../../../components/StatusDot";
import { PageLayout } from "../PageLayout";
import { SettingRow } from "../SettingRow";

export function AboutPage() {
  return (
    <PageLayout eyebrow="LXMI" title="Acerca de">
      <div>
        <SettingRow
          label="Versión"
          value={`LXMI ${version}`}
          status={
            isDemoMode && (
              <StatusDot status="warn" size="sm">
                Modo demo
              </StatusDot>
            )
          }
          detail={
            isDemoMode
              ? "Interfaz en el navegador con datos de ejemplo; las acciones no tocan el sistema"
              : "Gestor de mods XXMI para Linux · Tauri + React"
          }
        />
        <SettingRow
          label="Gestión de archivos"
          value="Copias privadas con inventario y SHA-256"
          detail="Los mods originales y la instalación del juego no se modifican"
        />
        <SettingRow
          label="Carga de mods"
          value="Loader oficial de XXMI sobre una copia del perfil"
          detail="Steam y HoYoPlay se usan tal cual; cada cambio de perfil es reversible"
        />
        <SettingRow
          label="Riesgo"
          value="Los mods son contenido de terceros en juegos online"
          detail="Su uso puede infringir las condiciones del juego; la decisión es tuya"
        />
      </div>
    </PageLayout>
  );
}
