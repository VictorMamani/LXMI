// Real webview and IPC; synthetic resources. No global input, mocks, or loader calls.
(async () => {
  if (window.__LXMI_NATIVE_SMOKE_RUNNING__) return;
  window.__LXMI_NATIVE_SMOKE_RUNNING__ = true;
  const { root, phase } = window.__LXMI_NATIVE_SMOKE__;
  const steps = [];
  const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const invoke = (command, args) =>
    window.__TAURI_INTERNALS__.invoke(command, args);
  const emit = (report) =>
    invoke("plugin:event|emit", {
      event: "lxmi-native-mod-smoke",
      payload: report,
    });
  const check = (condition, message) => {
    if (!condition) throw new Error(message);
  };
  const scope = () => document.querySelector(".mod-manager");
  const buttons = () => [...scope().querySelectorAll("button")];
  const button = (name) =>
    buttons().find((node) => node.textContent.trim() === name);
  const wait = async (predicate, label) => {
    const end = performance.now() + 60000;
    while (performance.now() < end) {
      const result = predicate();
      if (result) return result;
      await pause(100);
    }
    throw new Error(`Timeout: ${label}`);
  };
  const ready = () =>
    button("Actualizar biblioteca") &&
    !button("Actualizar biblioteca").disabled;
  const note = async (step) => {
    steps.push(step);
    await emit({ status: "progress", phase, step });
  };
  const set = async (id, value) => {
    const node = document.querySelector(`#${id}`);
    check(node && !node.disabled, `Input unavailable: ${id}`);
    const proto =
      node.tagName === "SELECT"
        ? HTMLSelectElement.prototype
        : HTMLInputElement.prototype;
    Object.getOwnPropertyDescriptor(proto, "value").set.call(node, value);
    node.dispatchEvent(
      new Event(node.tagName === "SELECT" ? "change" : "input", {
        bubbles: true,
      }),
    );
    await pause(100);
  };
  const click = async (name) => {
    const node = button(name);
    check(node && !node.disabled, `Button unavailable: ${name}`);
    node.click();
    await pause(100);
    await wait(ready, name);
    check(
      !scope().querySelector('[role="alert"]'),
      `UI error after ${name}: ${scope().querySelector('[role="alert"]')?.textContent}`,
    );
  };
  const mod = (name) =>
    [...scope().querySelectorAll(".mod-row")]
      .find((node) => node.querySelector("strong").textContent === name)
      ?.querySelector("input");
  const toggle = async (name) => {
    check(mod(name), `Missing mod: ${name}`);
    mod(name).click();
    await pause(100);
  };
  const importMod = async (name, source) => {
    await set("mod-name", name);
    await set("mod-source", source);
    await click("Importar mod");
  };
  try {
    await wait(
      () => document.querySelector(".primary-navigation"),
      "workspace navigation",
    );
    [...document.querySelectorAll(".navigation-tab")]
      .find(
        (node) =>
          node.textContent.trim() ===
          (phase === "activation" ? "Compatibilidad" : "Mods"),
      )
      .click();
    if (phase === "activation") {
      const runtimePanel = () => document.querySelector(".runtime-panel");
      const runtimeButton = (name) =>
        [...runtimePanel().querySelectorAll("button")].find(
          (n) => n.textContent.trim() === name,
        );
      const runtimeReady = () =>
        runtimePanel() &&
        runtimeButton("Actualizar estado") &&
        !runtimeButton("Actualizar estado").disabled &&
        document.querySelector("#runtime-importer");
      await wait(runtimeReady, "runtime panel ready");
      const status = await invoke("inspect_game_mod_runtime", {
        gameId: "wuthering-waves",
        importerId: null,
        librariesId: null,
      });
      const importer = status.importer_packages.find(
        (p) => p.authenticity === "official_release_verified",
      );
      const libraries = status.libraries_packages.find(
        (p) => p.authenticity === "official_release_verified",
      );
      check(importer && libraries, "Official WWMI and Libraries seed required");
      await set("runtime-importer", importer.id);
      await wait(runtimeReady, "importer chosen");
      await set("runtime-libraries", libraries.id);
      await wait(runtimeReady, "libraries chosen");
      const args = {
        gameId: "wuthering-waves",
        importerId: importer.id,
        librariesId: libraries.id,
      };
      const action = async (name) => {
        const node = runtimeButton(name);
        check(node && !node.disabled, `Runtime action unavailable: ${name}`);
        node.click();
        await pause(100);
        await wait(runtimeReady, name);
        check(
          !runtimePanel().querySelector('[role="alert"]'),
          `Runtime action failed: ${runtimePanel().querySelector('[role="alert"]')?.textContent}`,
        );
      };
      await action("Preparar runtime WWMI");
      const base = await invoke("inspect_game_mod_runtime", args);
      check(
        base.base_runtime?.target_process === "Client-Win64-Shipping.exe",
        "WWMI payload target missing",
      );
      await note("native WWMI assembly from authenticated upstream packages");
      await set("activation-profile", "Native daily");
      await action("Revisar aplicación del perfil");
      check(
        runtimePanel().querySelector(".activation-review"),
        "Review missing",
      );
      await action("Aplicar perfil al runtime");
      const applied = await invoke("inspect_game_mod_runtime", args);
      check(
        applied.selection.current?.profile_name === "Native daily",
        "Active profile not persisted",
      );
      check(
        applied.selected_runtime.runtime_id ===
          applied.selection.current.runtime_id,
        "Runtime resolver ignored the profile",
      );
      await note(
        "native reviewed activation with official WWMI packages and selected-runtime resolution",
      );
      await action("Revertir selección anterior");
      const undone = await invoke("inspect_game_mod_runtime", args);
      check(
        undone.selection.current === null &&
          undone.selected_runtime.runtime_id === undone.base_runtime.runtime_id,
        "Undo did not restore base selection",
      );
      await note(
        "native rollback restores base without deleting derived runtime",
      );
      await action("Rehacer selección");
      const redone = await invoke("inspect_game_mod_runtime", args);
      check(
        redone.selection.current.runtime_id ===
          applied.selection.current.runtime_id,
        "Redo did not reuse the verified derived runtime",
      );
      await note("native redo reuses the same verified selection");
      await emit({
        status: "passed",
        phase,
        transport: "real native Tauri IPC",
        interaction: "DOM events scoped to native GTK/WebKit webview",
        steps,
        game_executed: false,
      });
      return;
    }
    await wait(() => scope() && ready(), "library loaded");
    if (phase === "exercise") {
      await importMod("Native A", `${root}/source-a`);
      check(mod("Native A"), "Folder import did not appear");
      await note("folder import through native UI and IPC");
      await importMod("Duplicate A", `${root}/source-a`);
      check(
        scope().querySelectorAll(".mod-row").length === 1,
        "Duplicate was not deduplicated",
      );
      await note("content deduplication");
      await importMod("Native B", `${root}/source-b`);
      await toggle("Native A");
      await toggle("Native B");
      await set("mod-profile-name", "Native daily");
      await click("Guardar perfil");
      let library = await invoke("list_mod_library", {
        gameId: "wuthering-waves",
      });
      check(
        library.profiles[0].mod_ids.length === 2,
        "Profile did not persist two mods",
      );
      await note("save two-mod profile");
      await toggle("Native A");
      check(
        button("Preparar copia del perfil").disabled,
        "Dirty profile can be prepared",
      );
      await click("Recuperar selección guardada");
      check(mod("Native A").checked, "Direct restore failed");
      await note("dirty guard and saved-selection restore");
      await click("Preparar copia del perfil");
      await wait(
        () => scope().querySelector(".mod-prepared"),
        "prepared result",
      );
      await note("prepare actual private snapshot");
      await set("mod-name", "Invalid");
      await set("mod-source", `${root}/invalid-mod`);
      button("Importar mod").click();
      await pause(100);
      await wait(ready, "invalid import");
      check(
        scope().querySelector('[role="alert"]')?.textContent.includes("INI"),
        "Invalid mod error missing",
      );
      await note("invalid mod error");
      await set("mod-game", "zenless-zone-zero");
      await wait(ready, "ZZZ loaded");
      check(
        scope().querySelectorAll(".mod-row").length === 0,
        "Libraries mixed across games",
      );
      await importMod("Native ZZZ", `${root}/source-a`);
      await toggle("Native ZZZ");
      await set("mod-profile-name", "Native ZZZ daily");
      await click("Guardar perfil");
      await note("independent ZZZ import and profile");
      await set("mod-game", "wuthering-waves");
      await wait(ready, "WuWa restored");
      await set("saved-mod-profile", "Native daily");
      check(
        mod("Native A").checked && mod("Native B").checked,
        "Saved selection did not restore",
      );
      await note("game switch and restore persisted profile");
    } else {
      check(
        mod("Native A") && mod("Native B"),
        "Persisted mods missing after restart",
      );
      await set("saved-mod-profile", "Native daily");
      check(
        mod("Native A").checked && mod("Native B").checked,
        "Persisted selection missing after restart",
      );
      await click("Preparar copia del perfil");
      await note("restart persistence and snapshot reuse");
      await importMod("Native ZIP", `${root}/source-c.zip`);
      check(mod("Native ZIP"), "ZIP import missing from native UI");
      await note("ZIP import through native UI and real extraction");
      await set("mod-game", "zenless-zone-zero");
      await wait(ready, "ZZZ after restart");
      await set("saved-mod-profile", "Native ZZZ daily");
      check(mod("Native ZZZ").checked, "ZZZ profile missing after restart");
      await note("ZZZ restart persistence");
    }
    await emit({
      status: "passed",
      phase,
      transport: "real native Tauri IPC",
      interaction:
        "DOM events inside the native GTK/WebKit webview; no OS keyboard injection",
      steps,
    });
  } catch (error) {
    await emit({ status: "failed", phase, error: String(error), steps });
  }
})();
