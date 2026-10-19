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

/** "/home/a/Descargas/Ellen — Verano.zip" → "Ellen — Verano" */
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
      eyebrow={`Biblioteca · ${gameName}`}
      title="Importar mod"
      onClose={onClose}
      width={560}
    >
      <form className={styles.form} onSubmit={(event) => void submit(event)}>
        <TextField
          label="Carpeta o ZIP"
          placeholder="/home/usuario/Descargas/mi-mod.zip"
          value={source}
          onChange={(event) => setSource(event.target.value)}
          hint="Ruta absoluta. El mod debe contener un INI; RAR y 7z deben extraerse antes."
          required
          autoFocus
        />
        <TextField
          label="Nombre en la biblioteca"
          placeholder={nameFromPath(source) || "Personaje — Variante"}
          value={name}
          maxLength={100}
          onChange={(event) => setName(event.target.value)}
        />
        <p>
          LXMI guarda una copia privada; el original y el juego no se modifican.
          Los hashes detectan cambios locales, no certifican el origen del mod.
        </p>
        {error && <InlineError>{error}</InlineError>}
        <div className={styles.formActions}>
          <Button onClick={onClose}>Cancelar</Button>
          <Button
            type="submit"
            variant="primary"
            icon={FolderPlus}
            disabled={busy || !source.trim() || !finalName}
          >
            {busy ? "Importando…" : "Importar"}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
