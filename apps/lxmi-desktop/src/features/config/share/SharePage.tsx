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
      Coming soon
    </StatusDot>
  );
  return (
    <PageLayout eyebrow={`Space ${game.channel}`} title="Share">
      <div>
        <SettingRow
          label="Export profile"
          value="Package containing the mod list and hashes"
          status={soon}
          detail={`${profiles} ${profiles === 1 ? "available profile" : "available profiles"} in ${game.name}`}
          action={
            <Button icon={Upload} disabled>
              Export
            </Button>
          }
        />
        <SettingRow
          label="Import profile"
          value="Recreate a shared profile using mods from your library"
          status={soon}
          detail="Missing mods will be listed before anything is applied"
          action={
            <Button icon={Download} disabled>
              Import
            </Button>
          }
        />
      </div>
    </PageLayout>
  );
}
