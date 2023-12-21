import * as C from "./cardano_multiplatform_lib/cardano_multiplatform_lib.generated.js";

async function unsafeInstantiate(module: any, url: string) {
  try {
    await module.instantiate({
      // Exception for Deno fresh framework
      url: url
    });
  } catch (_e) {
    // This only ever happens during SSR rendering
  }
}

await Promise.all([
  unsafeInstantiate(
    C,
    `./cardano_multiplatform_lib/cardano_multiplatform_lib_bg.wasm`,
  ),
]);

export { C };
