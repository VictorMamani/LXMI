import assert from "node:assert/strict";
import { after, test } from "node:test";

// Load the production services with Node's TypeScript loader and an explicit
// native transport stand-in. No browser, native IPC, files, or game processes.
globalThis.isTauri = true;
const { mockTransport } = await import("../src/api/mock/mockTransport.ts");
const { resolveSources } = await import("../src/domain/runtime.ts");
const { sameModSelection } = await import("../src/domain/mods.ts");
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
