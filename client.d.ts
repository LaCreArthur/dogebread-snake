/* tslint:disable */
/* eslint-disable */

export function wasm_main(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly wasm_main: () => void;
    readonly __wasm_bindgen_func_elem_5844: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_77599: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_172817: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_6090: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_6087: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_6088: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_172833: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_6086: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_77636: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
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
