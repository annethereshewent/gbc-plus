import { InitOutput, WebEmulator } from "../../../pkg/gb_plus_web";

export const SCREEN_WIDTH = 160
export const SCREEN_HEIGHT = 144

export class VideoInterface {
  private wasm: InitOutput|null = null
  private canvas: HTMLCanvasElement
  private emulator: WebEmulator|null = null
  private context

  constructor(canvas: HTMLCanvasElement, context: CanvasRenderingContext2D) {
    this.canvas = canvas
    this.context = context
  }

  setMemory(wasm: InitOutput|null) {
    this.wasm = wasm
  }

  setEmulator(emulator: WebEmulator) {
    this.emulator = emulator
  }

  updateCanvas() {
    const emu = this.emulator!
    const memory = new Uint8Array(this.wasm!.memory.buffer, emu.get_screen(), emu.get_screen_length())

    const imageData = this.context.getImageData(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)

    for (let y = 0; y < SCREEN_HEIGHT; y++) {
      for (let x = 0; x < SCREEN_WIDTH; x++) {
        const index = x * 4 + y * SCREEN_WIDTH * 4

        imageData.data[index] = memory[index]
        imageData.data[index + 1] = memory[index + 1]
        imageData.data[index + 2] = memory[index + 2]
        imageData.data[index + 3] = memory[index + 3]
      }
    }

    this.context.putImageData(imageData, 0, 0)
  }
}