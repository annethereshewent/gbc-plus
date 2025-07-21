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

        this.yCoords = y

        this.plot(realX)
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
        this.context!.fillStyle = "rgb(200 200 200)"
        this.context!.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT)
    }

    plot(x: number) {
        // this.context!.fillStyle = "rgba(200, 200, 200, 0.05)"
        // this.context!.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT)

        this.drawAxisLines()
        // Begin the path
        this.context!.lineWidth = 5  // 0x088f8f
        this.context!.strokeStyle = "rgba(8, 143, 143, 0.5)"
        this.context!.beginPath()

        const realX = Math.floor(x / 15)
        for (const y of this.yCoords) {
            const realY = CANVAS_HEIGHT / 2 - Math.floor((y * CANVAS_HEIGHT) / 2)
            if (realX == 0) {
                this.context!.moveTo(realX, realY)
            } else {
                this.context!.lineTo(realX, realY)
            }
        }

        // this.context!.lineTo(CANVAS_WIDTH, CANVAS_HEIGHT / 2)
        this.context!.stroke()

    }
}