import { Plus } from "lucide-react";
import { useState, type FormEvent } from "react";
import { Button } from "../../../components/Button";
import { Dialog } from "../../../components/Dialog";
import { TextField } from "../../../components/TextField";
import styles from "./ModsPage.module.css";

/** Names a new profile; it starts from the current selection. */
export function NewProfileDialog({
  existing,
  onCreate,
  onClose,
}: {
  existing: string[];
  onCreate: (name: string) => void;
  onClose: () => void;
}) {
  const [name, setName] = useState("");
  const trimmed = name.trim();
  const duplicate = existing.some(
    (p) => p.toLowerCase() === trimmed.toLowerCase(),
  );

  function submit(event: FormEvent) {
    event.preventDefault();
    if (trimmed && !duplicate) onCreate(trimmed);
  }

  return (
    <Dialog
      eyebrow="Perfiles"
      title="Nuevo perfil"
      onClose={onClose}
      width={440}
    >
      <form className={styles.form} onSubmit={submit}>
        <TextField
          label="Nombre"
          placeholder="Por ejemplo, Capturas"
          value={name}
          maxLength={100}
          onChange={(event) => setName(event.target.value)}
          hint={
            duplicate
              ? "Ya existe un perfil con ese nombre."
              : "Parte de la selección actual. Se guarda al aplicar."
          }
          autoFocus
        />
        <div className={styles.formActions}>
          <Button onClick={onClose}>Cancelar</Button>
          <Button
            type="submit"
            variant="primary"
            icon={Plus}
            disabled={!trimmed || duplicate}
          >
            Crear
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
