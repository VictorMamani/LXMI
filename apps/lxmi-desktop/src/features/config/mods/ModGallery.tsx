import {
  Box,
  FileQuestion,
  LayoutPanelTop,
  LoaderCircle,
  Paintbrush,
  Sparkles,
  type LucideIcon,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import type {
  ModDetails,
  ModKind,
  ModManifest,
  PreviewView,
} from "../../../api/types";
import { Switch } from "../../../components/Switch";
import { formatBytes, plural } from "../../../domain/format";
import {
  kindLabel,
  modCharacter,
  modSize,
  type ModChange,
} from "../../../domain/mods";
import styles from "./ModGallery.module.css";
import { usePreviewImage } from "./usePreviewImage";

// Details fade out after this long without pointer movement over a render.
const REVEAL_MS = 1500;

const kindIcon: Record<ModKind, LucideIcon> = {
  model: Box,
  texture: Paintbrush,
  ui: LayoutPanelTop,
  shader: Sparkles,
  other: FileQuestion,
};

const changeLabel: Record<Exclude<ModChange, null>, string> = {
  added: "+ NEW",
  removed: "− REMOVED",
};

const angles: { view: Exclude<PreviewView, "thumbnail">; label: string }[] = [
  { view: "front", label: "Front" },
  { view: "back", label: "Back" },
];

type Status = "rendering" | "queued" | null;

function placeholderText(details: ModDetails | undefined, status: Status) {
  if (!details) return "Reading mod…";
  if (status === "rendering") return "Rendering…";
  if (status === "queued") return "Waiting to render";
  if (!details.previewable)
    return `No render · ${kindLabel[details.kind].toLowerCase()} mod`;
  if (details.preview === "unavailable") return "Could not render this model";
  return "Turn on previews to render";
}

function Card({
  gameId,
  mod,
  details,
  status,
  view,
  checked,
  change,
  disabled,
  onToggle,
  onView,
}: {
  gameId: string;
  mod: ModManifest;
  details: ModDetails | undefined;
  status: Status;
  view: "front" | "back";
  checked: boolean;
  change: ModChange;
  disabled: boolean;
  onToggle: (checked: boolean) => void;
  onView: (view: "front" | "back") => void;
}) {
  const ready = details?.preview === "ready";
  const loaded = usePreviewImage(gameId, mod.id, view, ready);
  const [revealed, setRevealed] = useState(false);
  // Fetch the other angle while the details are shown, so switching is instant.
  usePreviewImage(
    gameId,
    mod.id,
    view === "front" ? "back" : "front",
    ready && revealed,
  );
  // Keep the previous angle on screen while the next one loads.
  const [shown, setShown] = useState<string | null>(null);
  if (loaded && loaded !== shown) setShown(loaded);
  const url = ready ? (loaded ?? shown) : null;
  const timer = useRef<number | undefined>(undefined);
  useEffect(() => () => window.clearTimeout(timer.current), []);

  function reveal() {
    setRevealed(true);
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setRevealed(false), REVEAL_MS);
  }
  function conceal() {
    window.clearTimeout(timer.current);
    setRevealed(false);
  }

  const Icon = kindIcon[details?.kind ?? "other"];
  const character = modCharacter(mod, details);
  return (
    <article
      className={styles.card}
      aria-label={mod.name}
      data-checked={checked}
      data-changed={change !== null}
      data-image={url !== null}
      data-revealed={revealed}
      data-disabled={disabled}
      onPointerMove={reveal}
      onPointerLeave={conceal}
      onClick={(event) => {
        // The switch and the angle buttons handle their own clicks.
        if (disabled || (event.target as Element).closest("button, input"))
          return;
        onToggle(!checked);
      }}
    >
      {url && <img className={styles.render} src={url} alt="" />}
      <div className={styles.veil} aria-hidden />

      <div className={styles.idle} aria-hidden>
        {checked && <span className={styles.dot} />}
        {change && <span className={styles.change}>{changeLabel[change]}</span>}
      </div>

      <div className={styles.details}>
        <div className={styles.top}>
          <label className={styles.active}>
            <Switch
              checked={checked}
              disabled={disabled}
              label={`Include ${mod.name}`}
              onChange={onToggle}
            />
            {checked ? "Active" : "Inactive"}
          </label>
          {change ? (
            <span className={styles.change}>{changeLabel[change]}</span>
          ) : (
            details && (
              <span className={styles.kind}>{kindLabel[details.kind]}</span>
            )
          )}
        </div>

        {!url && (
          <div className={styles.placeholder} data-status={status}>
            {status === "rendering" ? (
              <LoaderCircle
                className={styles.spinner}
                size={22}
                strokeWidth={1.5}
              />
            ) : (
              <Icon size={34} strokeWidth={1.25} />
            )}
            <span>{placeholderText(details, status)}</span>
          </div>
        )}

        <div className={styles.bottom}>
          <div className={styles.titles}>
            <p className={styles.character}>{character ?? mod.name}</p>
            {character && <p className={styles.name}>{mod.name}</p>}
          </div>
          <dl className={styles.meta}>
            <div>
              <dt>Size</dt>
              <dd>{formatBytes(modSize(mod))}</dd>
            </div>
            <div>
              <dt>Files</dt>
              <dd>{plural(mod.files.length, "file")}</dd>
            </div>
          </dl>
          {ready && (
            <div className={styles.angles} role="group" aria-label="Angle">
              {angles.map((angle) => (
                <button
                  key={angle.view}
                  type="button"
                  className={styles.angle}
                  aria-pressed={view === angle.view}
                  onClick={() => {
                    onView(angle.view);
                    // Step aside so the new angle is seen in full.
                    conceal();
                  }}
                >
                  {angle.label}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </article>
  );
}

/**
 * Mod library as cards with the render of each 3D model. Details fade out
 * over a render and come back while the pointer moves over the card.
 */
export function ModGallery({
  gameId,
  mods,
  details,
  rendering,
  queued,
  selected,
  changeOf,
  disabled,
  emptyText,
  onToggle,
}: {
  gameId: string;
  mods: ModManifest[];
  details: ReadonlyMap<string, ModDetails>;
  rendering: string | null;
  queued: boolean;
  selected: readonly string[];
  changeOf: (id: string) => ModChange;
  disabled: boolean;
  emptyText: string;
  onToggle: (id: string, checked: boolean) => void;
}) {
  // Angle chosen per mod for this session.
  const [views, setViews] = useState<ReadonlyMap<string, "front" | "back">>(
    new Map(),
  );

  if (mods.length === 0) return <p className={styles.empty}>{emptyText}</p>;
  return (
    <div className={styles.grid} role="list" aria-label="Mod library">
      {mods.map((mod) => {
        const info = details.get(mod.id);
        const pending = info?.previewable && info.preview === "missing";
        const status: Status =
          mod.id === rendering
            ? "rendering"
            : pending && queued
              ? "queued"
              : null;
        return (
          <div key={mod.id} role="listitem" className={styles.cell}>
            <Card
              gameId={gameId}
              mod={mod}
              details={info}
              status={status}
              view={views.get(mod.id) ?? "front"}
              checked={selected.includes(mod.id)}
              change={changeOf(mod.id)}
              disabled={disabled}
              onToggle={(checked) => onToggle(mod.id, checked)}
              onView={(view) =>
                setViews((current) => new Map(current).set(mod.id, view))
              }
            />
          </div>
        );
      })}
    </div>
  );
}
