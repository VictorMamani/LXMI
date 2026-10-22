import { LayoutGrid, List, MoreHorizontal } from "lucide-react";
import { MenuButton } from "../../../components/MenuButton";
import { Switch } from "../../../components/Switch";
import { plural } from "../../../domain/format";
import { kindFilters, type KindFilter } from "../../../domain/mods";
import styles from "./ModToolbar.module.css";
import type { PreviewProgress } from "./useModInsights";

export type ModLayout = "gallery" | "list";

const layouts = [
  { id: "gallery", label: "Gallery", icon: LayoutGrid },
  { id: "list", label: "List", icon: List },
] as const;

function previewStatus(
  enabled: boolean,
  progress: PreviewProgress | null,
  ready: number,
): string {
  if (!enabled) return "Previews off";
  if (progress && progress.done < progress.total)
    return `Rendering previews · ${progress.done + 1} of ${progress.total}`;
  const failed =
    progress && progress.failed > 0 ? ` · ${progress.failed} failed` : "";
  return `${plural(ready, "preview")}${failed}`;
}

/** Kind filter, gallery or list layout, and the optional preview generator. */
export function ModToolbar({
  filter,
  counts,
  onFilter,
  layout,
  onLayout,
  previewsEnabled,
  progress,
  readyPreviews,
  onPreviewsChange,
  onClearPreviews,
}: {
  filter: KindFilter;
  counts: Record<KindFilter, number>;
  onFilter: (filter: KindFilter) => void;
  layout: ModLayout;
  onLayout: (layout: ModLayout) => void;
  previewsEnabled: boolean;
  progress: PreviewProgress | null;
  readyPreviews: number;
  onPreviewsChange: (enabled: boolean) => void;
  onClearPreviews: () => void;
}) {
  return (
    <div className={styles.toolbar}>
      <div className={styles.filters} role="group" aria-label="Mod type">
        {kindFilters.map(({ id, label }) => (
          <button
            key={id}
            type="button"
            className={styles.filter}
            aria-pressed={filter === id}
            disabled={id !== "all" && counts[id] === 0 && filter !== id}
            onClick={() => onFilter(id)}
          >
            {label}
            <span className={styles.count}>{counts[id]}</span>
          </button>
        ))}
      </div>

      <div className={styles.previews}>
        <span
          className={styles.status}
          data-running={progress !== null && progress.done < progress.total}
          aria-live="polite"
        >
          {previewStatus(previewsEnabled, progress, readyPreviews)}
        </span>
        <div className={styles.layouts} role="group" aria-label="Layout">
          {layouts.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              type="button"
              className={styles.layout}
              aria-label={label}
              title={label}
              aria-pressed={layout === id}
              onClick={() => onLayout(id)}
            >
              <Icon size={14} strokeWidth={1.75} />
            </button>
          ))}
        </div>
        <label
          className={styles.toggle}
          title="Render a picture of every 3D model mod. Takes a few seconds per mod."
        >
          <Switch
            checked={previewsEnabled}
            label="Generate model previews"
            onChange={onPreviewsChange}
          />
          Previews
        </label>
        <MenuButton
          variant="panel"
          items={[
            {
              label: "Delete previews",
              hint: "Frees disk space; mods are kept",
              danger: true,
              disabled: readyPreviews === 0 && !previewsEnabled,
              onSelect: onClearPreviews,
            },
          ]}
          trigger={(props) => (
            <button
              type="button"
              className={styles.more}
              aria-label="Preview options"
              {...props}
            >
              <MoreHorizontal size={15} strokeWidth={1.75} />
            </button>
          )}
        />
      </div>
    </div>
  );
}
