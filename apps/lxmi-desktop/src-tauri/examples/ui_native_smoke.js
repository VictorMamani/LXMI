// Read-only inspection in the application's own WebKitGTK view. No key events
// reach the game, and no save, activation, helper, or launch command is invoked.
(async () => {
  if (window.__LXMI_UI_SMOKE__) return;
  window.__LXMI_UI_SMOKE__ = true;
  const invoke = (command, args) =>
    window.__TAURI_INTERNALS__.invoke(command, args);
  const emit = (payload) =>
    invoke("plugin:event|emit", { event: "lxmi-native-ui-smoke", payload });
  const check = (condition, detail) => {
    if (!condition) throw new Error(detail);
  };
  const wait = async (predicate) => {
    const deadline = performance.now() + 90000;
    while (performance.now() < deadline) {
      if (predicate()) return;
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    throw new Error("Native UI did not settle");
  };
  try {
    await wait(() =>
      [...document.querySelectorAll("button")].some(
        (b) => b.textContent.trim() === "MANAGE MODS",
      ),
    );
    check(
      document.documentElement.lang === "en",
      "Document language must be English",
    );
    for (const label of ["Minimize", "Maximize", "Close"]) {
      check(
        document.querySelector(`button[aria-label="${label}"]`),
        `Missing native control: ${label}`,
      );
    }
    const gameId = "zenless-zone-zero";
    const [library, runtime] = await Promise.all([
      invoke("list_mod_library", { gameId }),
      invoke("inspect_game_mod_runtime", {
        gameId,
        importerId: null,
        librariesId: null,
      }),
    ]);
    check(Array.isArray(library.profiles), "Invalid real library response");
    check(
      "applied_profile" in runtime,
      "Real IPC must expose the applied profile identity",
    );
    if (runtime.selection.current) {
      check(
        runtime.applied_profile?.name ===
          runtime.selection.current.profile_name,
        "Applied profile name mismatch",
      );
      check(
        runtime.applied_profile.mod_ids.length ===
          runtime.selection.current.mod_count,
        "Applied profile count mismatch",
      );
    }
    [...document.querySelectorAll("button")]
      .find((b) => b.textContent.trim() === "MANAGE MODS")
      .click();
    await wait(() =>
      document.querySelector(
        'button[aria-label="Switch profile"]:not(:disabled)',
      ),
    );
    const trigger = document.querySelector(
      'button[aria-label="Switch profile"]',
    );
    trigger.click();
    await wait(
      () => document.activeElement?.getAttribute("role") === "menuitem",
    );
    document.dispatchEvent(
      new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }),
    );
    check(
      document.activeElement?.getAttribute("role") === "menuitem",
      "Menu keyboard focus lost",
    );
    document.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Escape", bubbles: true }),
    );
    await wait(() => !document.querySelector('[role="menu"]'));
    check(
      document.activeElement === trigger,
      "Menu did not restore trigger focus",
    );
    await emit({
      status: "passed",
      transport: "real Tauri IPC",
      engine: "WebKitGTK",
      english_ui: true,
      applied_profile_identity: true,
      menu_keyboard_focus: true,
      library_mutated: false,
      game_executed: false,
      helper_executed: false,
    });
  } catch (error) {
    await emit({
      status: "failed",
      error: String(error),
      library_mutated: false,
      game_executed: false,
      helper_executed: false,
    });
  }
})();
