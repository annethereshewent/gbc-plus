import { InitOutput, WebEmulator } from "../../../pkg/gb_plus_web";

const SAMPLE_RATE = 44100
const BUFFER_SIZE = 8192

export class AudioInterface {
  private wasm: InitOutput|null = null
  private emulator: WebEmulator|null = null
  private audioContext = new AudioContext({ sampleRate: SAMPLE_RATE })
  private scriptProcessor: ScriptProcessorNode = this.audioContext.createScriptProcessor(BUFFER_SIZE, 0, 2)

  setMemory(wasm: InitOutput|null) {
    this.wasm = wasm
  }

  setEmulator(emulator: WebEmulator) {
    this.emulator = emulator
  }

  async playSamples() {
    this.scriptProcessor.onaudioprocess = (e) => {
      const leftData = e.outputBuffer.getChannelData(0)
      const rightData = e.outputBuffer.getChannelData(1)

      this.emulator!.modify_samples(leftData, rightData)
    }

    this.scriptProcessor.connect(this.audioContext.destination)
    this.audioContext.resume()
  }

}