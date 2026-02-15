/* tslint:disable */
/* eslint-disable */

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly main: (a: number, b: number) => number;
    readonly wasm_bindgen__closure__destroy__h49c680ffa1e275f6: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h1c6025adad97dcb2: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h80113798ecad8b8d: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h27a15a66e0e750cb: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h19eb54480858731b: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h32c9673f31401131: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h5b641ee896aff864: (a: number, b: number, c: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h9b704e0e055be319: (a: number, b: number) => number;
    readonly wasm_bindgen__convert__closures_____invoke__h9d7ca062d532fe7a: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h58a0fb7f8e3c29d8: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
