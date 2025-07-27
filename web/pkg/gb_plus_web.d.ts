/* tslint:disable */
/* eslint-disable */
export class WebEmulator {
  free(): void;
  constructor();
  set_pause(value: boolean): void;
  change_palette(index: number): void;
  has_timer(): boolean;
  fetch_rtc(): string;
  load_rtc(json: string): void;
  load_rom(data: Uint8Array): void;
  step_frame(): void;
  load_save_state(data: Uint8Array): void;
  create_save_state(): number;
  save_state_length(): number;
  reload_rom(data: Uint8Array): void;
  get_screen(): number;
  get_screen_length(): number;
  read_ringbuffer(): number;
  pop_sample(): number | undefined;
  load_save(buf: Uint8Array): void;
  has_saved(): boolean;
  get_save_length(): number;
  save_game(): number;
  get_buffer_len(): number;
  update_input(button: number, pressed: boolean): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_webemulator_free: (a: number, b: number) => void;
  readonly webemulator_new: () => number;
  readonly webemulator_set_pause: (a: number, b: number) => void;
  readonly webemulator_change_palette: (a: number, b: number) => void;
  readonly webemulator_has_timer: (a: number) => number;
  readonly webemulator_fetch_rtc: (a: number) => [number, number];
  readonly webemulator_load_rtc: (a: number, b: number, c: number) => void;
  readonly webemulator_load_rom: (a: number, b: number, c: number) => void;
  readonly webemulator_step_frame: (a: number) => void;
  readonly webemulator_load_save_state: (a: number, b: number, c: number) => void;
  readonly webemulator_create_save_state: (a: number) => number;
  readonly webemulator_save_state_length: (a: number) => number;
  readonly webemulator_reload_rom: (a: number, b: number, c: number) => void;
  readonly webemulator_get_screen: (a: number) => number;
  readonly webemulator_get_screen_length: (a: number) => number;
  readonly webemulator_read_ringbuffer: (a: number) => number;
  readonly webemulator_pop_sample: (a: number) => number;
  readonly webemulator_load_save: (a: number, b: number, c: number) => void;
  readonly webemulator_has_saved: (a: number) => number;
  readonly webemulator_get_save_length: (a: number) => number;
  readonly webemulator_save_game: (a: number) => number;
  readonly webemulator_get_buffer_len: (a: number) => number;
  readonly webemulator_update_input: (a: number, b: number, c: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_3: WebAssembly.Table;
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
