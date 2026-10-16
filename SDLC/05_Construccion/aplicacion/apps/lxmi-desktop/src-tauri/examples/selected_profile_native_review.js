// Read-only production IPC, scoped DOM navigation, no keyboard or game launch.
(async () => {
  if (window.__LXMI_SELECTED_REVIEW_RUNNING__) return;
  window.__LXMI_SELECTED_REVIEW_RUNNING__ = true;
  const invoke = (command, args) => window.__TAURI_INTERNALS__.invoke(command, args);
  const emit = (payload) => invoke("plugin:event|emit", {
    event: "lxmi-selected-profile-native-review", payload,
  });
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const wait = async (step, predicate) => {
    const end = performance.now() + 180000;
    while (performance.now() < end) {
      if (predicate()) return;
      await new Promise((resolve) => setTimeout(resolve, 200));
    }
    throw new Error(`Native profile view did not settle: ${step}`);
  };
  try {
    await wait("game switch", () => document.querySelector('[aria-label="Seleccionar Zenless Zone Zero"]'));
    document.querySelector('[aria-label="Seleccionar Zenless Zone Zero"]').click();
    const playWithMods = () => [...document.querySelectorAll("button")].find((b) => b.textContent.trim() === "Jugar con mods");
    await wait("play with mods enabled", () => playWithMods() && !playWithMods().disabled);
    check(await invoke("game_with_mods_status") === null, "Unexpected launch session");
    await wait("compatibility tab", () => [...document.querySelectorAll("button")].some((b) => b.textContent.trim() === "Compatibilidad"));
    [...document.querySelectorAll("button")].find((b) => b.textContent.trim() === "Compatibilidad").click();
    const gameId = "zenless-zone-zero";
    const applied = (await invoke("inspect_game_mod_runtime", { gameId, importerId: null, librariesId: null })).selection.current;
    check(applied, "No applied ZZZ profile");
    await wait("applied profile shown", () => document.querySelector(".runtime-selection-line strong")?.textContent === applied.profile_name);
    const library = await invoke("list_mod_library", { gameId });
    const importerId = document.querySelector("#runtime-importer").value;
    const librariesId = document.querySelector("#runtime-libraries").value;
    check(importerId && librariesId, "Saved package selection not restored in native UI");
    const state = await invoke("inspect_game_mod_runtime", { gameId, importerId, librariesId });
    check(state.selection.current?.mod_count === applied.mod_count && applied.mod_count > 0, "Selected mod count differs from the applied profile");
    check(state.selected_runtime?.runtime_id === state.selection.current.runtime_id, "Resolver did not consume selected profile");
    check(state.launch_with_mods_available === true && state.launch_blockers.length === 0, "Play with mods should be available for the applied ZZZ profile");
    const readiness = await invoke("inspect_zzz_launch_readiness", { zzmiId: importerId, librariesId });
    await emit({
      status: "passed", transport: "real native Tauri IPC", interaction: "DOM navigation in own GTK/WebKit webview",
      profile_name: state.selection.current.profile_name, runtime_id: state.selected_runtime.runtime_id,
      selected_mod_count: state.selection.current.mod_count, library_mod_names: library.mods.map((m) => m.name),
      saved_package_selection_restored: true, resolved_selected_runtime: true,
      launch_with_mods_available: state.launch_with_mods_available, play_with_mods_button_enabled: true, readiness,
      library_mutated: false, game_executed: false, helper_executed: false, prefix_modified: false,
    });
  } catch (error) {
    await emit({ status: "failed", error: String(error), game_executed: false, helper_executed: false });
  }
})();
