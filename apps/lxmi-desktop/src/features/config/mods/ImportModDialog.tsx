import { FolderPlus } from "lucide-react";
import { useState, type FormEvent } from "react";
import { call } from "../../../api/client";
import { errorMessage } from "../../../api/errors";
import type { ModManifest } from "../../../api/types";
import { Button } from "../../../components/Button";
import { Dialog } from "../../../components/Dialog";
import { InlineError } from "../../../components/InlineError";
import { TextField } from "../../../components/TextField";
import styles from "./ModsPage.module.css";

/** "/home/a/Downloads/Ellen — Summer.zip" → "Ellen — Summer" */
function nameFromPath(path: string): string {
  const leaf =
    path
      .trim()
      .replace(/[\\/]+$/, "")
      .split(/[\\/]/)
      .pop() ?? "";
  return leaf.replace(/\.zip$/i, "");
}

/** Copies a folder or ZIP into the private library of the game. */
export function ImportModDialog({
  gameId,
  gameName,
  onImported,
  onClose,
}: {
  gameId: string;
  gameName: string;
  onImported: (mod: ModManifest) => void;
  onClose: () => void;
}) {
  const [source, setSource] = useState("");
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const finalName = name.trim() || nameFromPath(source);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      onImported(
        await call("import_local_mod", {
          gameId,
          name: finalName,
          source: source.trim(),
        }),
      );
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog
      eyebrow={`Library · ${gameName}`}
      title="Import mod"
      onClose={onClose}
      width={560}
    >
      <form className={styles.form} onSubmit={(event) => void submit(event)}>
        <TextField
          label="Directory or ZIP"
          placeholder="/home/user/Downloads/my-mod.zip"
          value={source}
          onChange={(event) => setSource(event.target.value)}
          hint="Absolute path. The mod must contain an INI; extract RAR and 7z archives first."
          required
          autoFocus
        />
        <TextField
          label="Library name"
          placeholder={nameFromPath(source) || "Character — Variant"}
          value={name}
          maxLength={100}
          onChange={(event) => setName(event.target.value)}
        />
        <p>
          LXMI stores a private copy; the original and the game are not
          modified. Hashes detect local changes; they do not certify the mod
          origin.
        </p>
        {error && <InlineError>{error}</InlineError>}
        <div className={styles.formActions}>
          <Button onClick={onClose}>Cancel</Button>
          <Button
            type="submit"
            variant="primary"
            icon={FolderPlus}
            disabled={busy || !source.trim() || !finalName}
          >
            {busy ? "Importing…" : "Import"}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
