import { Download, Upload } from "lucide-react";
import { Button } from "../../../components/Button";
import { StatusDot } from "../../../components/StatusDot";
import { useGameData } from "../../../state/GameDataContext";
import { PageLayout } from "../PageLayout";
import { SettingRow } from "../SettingRow";

/** Placeholder until profile export/import exists in the backend. */
export function SharePage() {
  const { game, library } = useGameData();
  const profiles = library.data?.profiles.length ?? 0;
  const soon = (
    <StatusDot status="idle" size="sm">
      Próximamente
    </StatusDot>
  );
  return (
    <PageLayout eyebrow={`Espacio ${game.channel}`} title="Compartir">
      <div>
        <SettingRow
          label="Exportar perfil"
          value="Paquete con la lista de mods y sus hashes"
          status={soon}
          detail={`${profiles} ${profiles === 1 ? "perfil disponible" : "perfiles disponibles"} en ${game.name}`}
          action={
            <Button icon={Upload} disabled>
              Exportar
            </Button>
          }
        />
        <SettingRow
          label="Importar perfil"
          value="Recrea un perfil compartido con los mods de tu biblioteca"
          status={soon}
          detail="Los mods que falten se indicarán antes de aplicar nada"
          action={
            <Button icon={Download} disabled>
              Importar
            </Button>
          }
        />
      </div>
    </PageLayout>
  );
}
