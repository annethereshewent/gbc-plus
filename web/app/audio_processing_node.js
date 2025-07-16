export class AudioProcessingNode extends AudioWorkletProcessor {
  sampleBuffer = []
  constructor(context) {
    super()

    this.port.onmessage = (ev) => {
      if (ev.data.type == "samples") {
        this.sampleBuffer.push(...ev.data.samples)
      }
    }
  }

  process(inputs, outputs, parameters) {
    const left = outputs[0][0]
    const right = outputs[0][1]

    let leftIndex = 0
    let rightIndex = 0

    let isLeft = true

    while (leftIndex < left.length || rightIndex < right.length) {
      if (this.sampleBuffer.length == 0) {
        break
      }
      if (isLeft) {
        left[leftIndex] = this.sampleBuffer.shift()
        leftIndex++
      } else {
        right[rightIndex] = this.sampleBuffer.shift()
        rightIndex++
      }

      isLeft = !isLeft
    }

    return true
  }
}

registerProcessor('audio-processor', AudioProcessingNode)