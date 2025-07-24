const CANVAS_WIDTH = 683
const CANVAS_HEIGHT = 256

const NUM_SNAPSHOTS = 10240

export class WaveformVisualizer {
  canvas: HTMLCanvasElement
  context: CanvasRenderingContext2D|null

  yCoords: number[] = new Array(NUM_SNAPSHOTS)

  originSampleTime = 0

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas
    this.context = canvas.getContext("2d")
  }

  append(x: number, y: number[]) {
    const realX = Math.floor(x) % NUM_SNAPSHOTS

    if (x >= NUM_SNAPSHOTS) {
      this.originSampleTime = 0
    }

    this.yCoords = this.yCoords.concat(y)

    while (this.yCoords.length >= CANVAS_WIDTH) {
      this.yCoords.shift()
    }

    this.plot()
  }

  drawOriginLines() {
    const originX = CANVAS_WIDTH / 2
    const originY = CANVAS_HEIGHT / 2

    this.context!.strokeStyle = 'red'
    this.context!.lineWidth = 1

    // Draw vertical line (Y-axis)
    this.context!.beginPath()
    this.context!.moveTo(originX, 0)
    this.context!.lineTo(originX, CANVAS_HEIGHT)
    this.context!.stroke()

    // Draw horizontal line (X-axis)
    this.context!.beginPath()
    this.context!.moveTo(0, originY)
    this.context!.lineTo(CANVAS_WIDTH, originY)
    this.context!.stroke()
  }

  drawAxisLines() {
    const originY = CANVAS_HEIGHT / 2

    this.context!.strokeStyle = '#000000'
    this.context!.lineWidth = 2

    this.context!.beginPath()
    this.context!.moveTo(0, originY)
    this.context!.lineTo(CANVAS_WIDTH, originY)
    this.context!.stroke()
  }

  redrawBackground() {
    // 191970
    this.context!.fillStyle = "rgb(25, 25, 112)"
    this.context!.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT)
  }

  plot() {
    this.redrawBackground()
    // this.context!.fillStyle = "rgba(200, 200, 200, 0.05)"
    // this.context!.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT)

    // this.drawAxisLines()
    // Begin the path
    this.context!.lineWidth = 5  // 0x088f8f
    this.context!.strokeStyle = "rgb(8, 143, 143)"
    this.context!.beginPath()

    for (let x = 0; x < this.yCoords.length; x++) {
      const y = this.yCoords[x]

      const realY = CANVAS_HEIGHT / 2 - Math.floor((y * CANVAS_HEIGHT) / 2)

      if (x == 0) {
        this.context!.moveTo(x, realY)
      } else {
        this.context!.lineTo(x, realY)
      }
    }

    // this.context!.lineTo(CANVAS_WIDTH, CANVAS_HEIGHT / 2)
    this.context!.stroke()
  }
}