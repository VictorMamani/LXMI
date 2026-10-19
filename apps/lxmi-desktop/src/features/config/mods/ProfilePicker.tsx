import { ChevronsUpDown } from "lucide-react";
import type { ModProfile } from "../../../api/types";
import { MenuButton } from "../../../components/MenuButton";
import { PageTitle } from "../PageLayout";
import styles from "./ProfilePicker.module.css";

/** Page title 'Profile · Name' with the switcher next to it. */
export function ProfilePicker({
  name,
  profiles,
  applied,
  disabled,
  onOpen,
  onCreate,
}: {
  name: string | null;
  profiles: ModProfile[];
  applied: string | null;
  disabled?: boolean;
  onOpen: (name: string) => void;
  onCreate: () => void;
}) {
  return (
    <div className={styles.picker}>
      <PageTitle>{name ? `Profile · ${name}` : "No profiles"}</PageTitle>
      <MenuButton
        variant="panel"
        placement="bottom-start"
        items={[
          ...profiles.map((profile) => ({
            label:
              profile.name === applied
                ? `${profile.name}  · applied`
                : profile.name,
            selected: profile.name === name,
            onSelect: () => onOpen(profile.name),
          })),
          { label: "+ New profile…", onSelect: onCreate },
        ]}
        trigger={(props) => (
          <button
            type="button"
            className={styles.switch}
            aria-label="Switch profile"
            disabled={disabled}
            {...props}
          >
            <ChevronsUpDown size={14} strokeWidth={1.75} aria-hidden />
          </button>
        )}
      />
    </div>
  );
}
