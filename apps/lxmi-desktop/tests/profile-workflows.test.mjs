import assert from "node:assert/strict";
import { after, test } from "node:test";

// Load the production services with Node's TypeScript loader and an explicit
// native transport stand-in. No browser, native IPC, files, or game processes.
globalThis.isTauri = true;
const { mockTransport } = await import("../src/api/mock/mockTransport.ts");
const { resolveSources } = await import("../src/domain/runtime.ts");
const { sameModSelection } = await import("../src/domain/mods.ts");
const {
  launchConsentKey,
  launchLauncher,
  launchStepStates,
  launchStepIndex,
  launchStepsFor,
} = await import("../src/domain/launch.ts");
const { updateRuntime } = await import("../src/services/runtimeUpdate.ts");
const { saveAndApplyProfile } =
  await import("../src/services/profileActivation.ts");
globalThis.window = { __TAURI_INTERNALS__: { invoke: mockTransport } };
after(() => {
  delete globalThis.window;
  delete globalThis.isTauri;
});

const gameId = "zenless-zone-zero";
const inspect = (importerId = null, librariesId = null) =>
  mockTransport("inspect_game_mod_runtime", {
    gameId,
    importerId,
    librariesId,
  });

test("updating packages migrates an active profile without resolving its old selection against the new base", async () => {
  const initial = await inspect();
  const sources = resolveSources(initial);
  const status = await inspect(sources.importer, sources.libraries);
  await updateRuntime({
    gameId,
    status,
    sources,
    updates: [{ kind: "zzmi", tag: "v2.5.0", version: "2.5.0" }],
  });
  const updated = await inspect();
  assert.notEqual(
    updated.selection.current.integration_package_id,
    sources.importer,
  );
  assert.deepEqual(updated.applied_profile, status.applied_profile);
  await assert.rejects(inspect(sources.importer, sources.libraries), (error) =>
    error.detail.includes("different base runtime"),
  );
  const next = resolveSources(updated);
  assert.ok((await inspect(next.importer, next.libraries)).base_runtime);
});

test("a saved profile remains distinguishable from the active runtime after activation fails, and can be retried without another save", async () => {
  const initial = await inspect();
  const sources = resolveSources(initial);
  const status = await inspect(sources.importer, sources.libraries);
  const library = await mockTransport("list_mod_library", { gameId });
  const profileName = status.applied_profile.name;
  // Keep the count unchanged: comparing only names or counts would miss this.
  const original = status.applied_profile.mod_ids;
  const replacement = library.mods.find((mod) => !original.includes(mod.id)).id;
  const modIds = [...original.slice(1), replacement];
  let rejectActivation = true;
  globalThis.window.__TAURI_INTERNALS__.invoke = (command, args) => {
    if (command === "activate_mod_profile" && rejectActivation) {
      rejectActivation = false;
      return Promise.reject(new Error("Controlled activation failure"));
    }
    return mockTransport(command, args);
  };
  const params = {
    gameId,
    library,
    profileName,
    modIds,
    changed: true,
    status,
    sources,
  };
  await assert.rejects(
    saveAndApplyProfile(params),
    /Controlled activation failure/,
  );
  const saved = await mockTransport("list_mod_library", { gameId });
  const unchanged = await inspect();
  assert.equal(saved.revision, library.revision + 1);
  assert.deepEqual(
    saved.profiles.find((profile) => profile.name === profileName).mod_ids,
    modIds,
  );
  assert.deepEqual(unchanged.applied_profile.mod_ids, original);
  assert.equal(
    sameModSelection(modIds, unchanged.applied_profile.mod_ids),
    false,
  );
  const result = await saveAndApplyProfile({
    ...params,
    library: saved,
    changed: false,
  });
  assert.deepEqual(result, { applied: true });
  assert.deepEqual((await inspect()).applied_profile.mod_ids, modIds);
  assert.equal(
    (await mockTransport("list_mod_library", { gameId })).revision,
    saved.revision,
  );
});

test("mod identity comparison ignores ordering and detects equal-count replacements", () => {
  assert.equal(sameModSelection(["a", "b"], ["b", "a"]), true);
  assert.equal(sameModSelection(["a", "b"], ["a", "c"]), false);
  assert.equal(sameModSelection(["a"], []), false);
});

test("WWMI has experimental runtime metadata and a separate acknowledgement from the existing ZZZ hook", async () => {
  const wwmi = await mockTransport("inspect_game_mod_runtime", {
    gameId: "wuthering-waves",
    importerId: null,
    librariesId: null,
  });
  assert.equal(wwmi.launch_experimental, true);
  assert.equal((await inspect()).launch_experimental, false);
  // Keep existing ZZZ acknowledgement; it must never acknowledge Direct Inject.
  assert.equal(launchConsentKey(gameId), "lxmi.modLaunchAcknowledged.v1");
  assert.notEqual(
    launchConsentKey("wuthering-waves"),
    launchConsentKey(gameId),
  );
  assert.equal(launchLauncher("wuthering-waves"), "Wuthering Waves");
  assert.equal(launchLauncher(gameId), "HoYoPlay");
});

test("a failed Direct Inject launch keeps the reached step failed rather than reporting a loaded module", () => {
  assert.deepEqual(
    launchStepStates({ game_id: "wuthering-waves", phase: "failed" }, 3),
    ["done", "done", "done", "failed", "pending", "pending"],
  );
});

test("WWMI stages its loader before Steam starts the game while ZZZ keeps its launcher-first steps", () => {
  const wuwa = "wuthering-waves";
  assert.equal(launchStepIndex(wuwa, "arming"), 1);
  assert.equal(launchStepIndex(wuwa, "opening_launcher"), 2);
  assert.equal(launchStepIndex(gameId, "opening_launcher"), 1);
  assert.equal(launchStepIndex(gameId, "arming"), 2);
  assert.equal(launchStepsFor(wuwa)[3].label, "Injecting WWMI");
  assert.equal(launchStepsFor(gameId)[3].label, "Press 'Play' in the launcher");
  assert.deepEqual(
    launchStepStates({ game_id: wuwa, phase: "opening_launcher" }, 2),
    ["done", "done", "current", "pending", "pending", "pending"],
  );
});
