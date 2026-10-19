import { version } from "../../../../package.json";
import { isDemoMode } from "../../../api/client";
import { StatusDot } from "../../../components/StatusDot";
import { PageLayout } from "../PageLayout";
import { SettingRow } from "../SettingRow";

export function AboutPage() {
  return (
    <PageLayout eyebrow="LXMI" title="About">
      <div>
        <SettingRow
          label="Version"
          value={`LXMI ${version}`}
          status={
            isDemoMode && (
              <StatusDot status="warn" size="sm">
                Demo mode
              </StatusDot>
            )
          }
          detail={
            isDemoMode
              ? "Browser interface with example data; actions do not touch your system"
              : "XXMI mod manager for Linux · Tauri + React"
          }
        />
        <SettingRow
          label="File management"
          value="Private copies with inventory and SHA-256"
          detail="Original mods and the game installation are not modified"
        />
        <SettingRow
          label="Mod loading"
          value="Official XXMI loader with a profile copy"
          detail="Steam and HoYoPlay are used as installed; each profile change is reversible"
        />
        <SettingRow
          label="Risk"
          value="Mods are third-party content in online games"
          detail="Their use may violate the game terms; the decision is yours"
        />
      </div>
    </PageLayout>
  );
}
