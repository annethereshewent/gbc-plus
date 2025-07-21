const CANVAS_WIDTH = 1024
const CANVAS_HEIGHT = 768

const NUM_SNAPSHOTS = 10000

export class WaveformAnalyzer {
    canvas: HTMLCanvasElement
    context: CanvasRenderingContext2D|null

    coordinates: number[][] = new Array(NUM_SNAPSHOTS)

    originSampleTime = 0

    constructor(canvas: HTMLCanvasElement) {
        this.canvas = canvas
        this.context = canvas.getContext("2d")
    }

    append(x: number, y: number[]) {
        const realX = Math.floor(x) % NUM_SNAPSHOTS

        if (x >= NUM_SNAPSHOTS) {
            this.originSampleTime = 0
            console.log("clearing coordinate buffer")
            this.coordinates.fill([], 0, NUM_SNAPSHOTS)
        }

        this.coordinates[realX] = y
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
        const originX = CANVAS_WIDTH / 2
        const originY = CANVAS_HEIGHT / 2

        this.context!.strokeStyle = '#ff00ff'
        this.context!.lineWidth = 1

        this.context!.beginPath()
        this.context!.moveTo(originX, 0)
        this.context!.lineTo(originX, CANVAS_HEIGHT)
        this.context!.stroke()

        this.context!.beginPath()
        this.context!.moveTo(0, originY)
        this.context!.lineTo(CANVAS_WIDTH, originY)
        this.context!.stroke()
        }

    plot() {
        this.context!.fillStyle = "rgb(200 200 200)"
        this.context!.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT)
        // Begin the path
        this.context!.lineWidth = 2
        this.context!.strokeStyle = "#088F8F"
        this.context!.beginPath()
        for (let x = 0; x < this.coordinates.length; x++) {
            const realX = Math.floor(x / 10)
            if (this.coordinates[x] == null) {
                continue
            }
            for (const y of this.coordinates[x]) {
                const realY = CANVAS_HEIGHT / 2 - Math.floor((y * CANVAS_HEIGHT) / 2)
                if (x == 0) {
                    this.context!.moveTo(realX, realY)
                } else {
                    this.context!.lineTo(realX, realY)
                }
            }
        }
        this.context!.lineTo(CANVAS_WIDTH, CANVAS_HEIGHT / 2)
        this.context!.stroke()
    }
}