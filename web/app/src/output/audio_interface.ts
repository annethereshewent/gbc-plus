import { InitOutput, WebEmulator } from "../../../pkg/gb_plus_web";

const SAMPLE_RATE = 44100
const BUFFER_SIZE = 8192

export class AudioInterface {
  private wasm: InitOutput|null = null
  private emulator: WebEmulator|null = null
  private audioContext = new AudioContext({ sampleRate: SAMPLE_RATE })
  private workletNode: AudioWorkletNode|null = null

  constructor() {
    this.initAudio()
  }

  async initAudio() {
    await this.audioContext.audioWorklet.addModule("audio_processing_node.js")

    this.workletNode = new AudioWorkletNode(this.audioContext, 'audio-processor', {
      numberOfOutputs: 1,
      outputChannelCount: [2]
    })
    this.workletNode.connect(this.audioContext.destination)

    await this.audioContext.resume()
  }

  setMemory(wasm: InitOutput|null) {
    this.wasm = wasm
  }

  setEmulator(emulator: WebEmulator) {
    this.emulator = emulator
  }

  pushSamples() {
    const ptr = this.emulator!.read_ringbuffer()
    const length = this.emulator!.get_buffer_len()
    const f32arr = new Float32Array(this.wasm!.memory.buffer, ptr, length)
    const samples = Array.from(f32arr)

    this.workletNode?.port.postMessage({ type: "samples", samples: samples })

    return samples
  }
}