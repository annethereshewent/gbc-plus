const CANVAS_WIDTH = 683
const CANVAS_HEIGHT = 256

const NUM_SNAPSHOTS = 10240

export class WaveformVisualizer {
  canvas: HTMLCanvasElement
  context: CanvasRenderingContext2D|null

  yCoords: number[] = new Array(NUM_SNAPSHOTS)

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas
    this.context = canvas.getContext("2d")
  }

  redrawBackground() {
    this.context!.fillStyle = "rgb(25, 25, 112)"
    this.context!.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT)
  }

  plot(yCoords: number[]) {
    this.redrawBackground()

    this.context!.lineWidth = 1  // 0x088f8f
    this.context!.strokeStyle = "rgb(8, 143, 143)"
    this.context!.beginPath()

    for (let x = 0; x < yCoords.length; x+= 2) {
      const y = yCoords[x]

      const realY = CANVAS_HEIGHT / 2 - Math.floor((y * CANVAS_HEIGHT) / 2)

      // x / 2 because of the left and right channels being interleaved together
      if (x == 0) {
        this.context!.moveTo(Math.floor(x / 2), realY)
      } else {
        this.context!.lineTo(Math.floor(x / 2), realY)
      }
    }

    this.context!.stroke()
  }
}