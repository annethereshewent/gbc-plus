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

  playSamples() {
    this.scriptProcessor.onaudioprocess = (e) => {
      const leftData = e.outputBuffer.getChannelData(0)
      const rightData = e.outputBuffer.getChannelData(1)

      let isLeft = false

      let left = 0
      let right = 0

      let sample = this.emulator!.pop_sample()
      while (sample != null) {
        if (isLeft) {
          if (left < leftData.length) {
            leftData[left] = sample
            left++
          }
        } else if (right < rightData.length) {
          rightData[right] = sample
          right++
        } else {
          break
        }

        sample = this.emulator!.pop_sample()
        isLeft = !isLeft
      }
    }

    this.scriptProcessor.connect(this.audioContext.destination)
    this.audioContext.resume()
  }

  // pushSamples() {
  //   const length = this.emulator!.get_buffer_len()
  //   console.log(`length = ${length}`)
  //   const float32Samples = new Float32Array(this.wasm!.memory.buffer, this.emulator!.read_ringbuffer(), length)

  //   console.log(`float32Samples.length = ${float32Samples.length}`)

  //   const arr = Array.from(float32Samples)

  //   console.log(`arr.length = ${arr.length}`)

  //   this.samples = this.samples.concat(arr)

  //   console.log(`this.samples.length = ${this.samples.length}`)
  // }
}