import JSZip from 'jszip'
import init, { WebEmulator, InitOutput } from "../../pkg/gb_plus_web"
import wasmData from '../../pkg/gb_plus_web_bg.wasm'
import { VideoInterface } from './output/video_interface'
import { AudioInterface } from './output/audio_interface'

const FPS_INTERVAL = 1000 / 60

export class GBC {
  private emulator: WebEmulator|null = null
  private wasm: InitOutput|null = null
  private canvas: HTMLCanvasElement = document.getElementById('game-canvas')! as HTMLCanvasElement
  private context = this.canvas.getContext("2d")
  private video: VideoInterface = new VideoInterface(this.canvas, this.context!)
  private audio: AudioInterface = new AudioInterface()
  private previousTime = 0

  async onGameChange(file: File) {
    const fileName = file.name

    const tokens = fileName.split('/')

    const extension = fileName.split('.').pop()!

    let data = null
    if (extension.toLowerCase() == 'zip') {
      const zipFile = await JSZip.loadAsync(file)

      data = await zipFile?.file(fileName)?.async('arraybuffer')
    } else if (['gb', 'gbc'].includes(extension.toLowerCase())) {
      data = await this.readFile(file) as ArrayBuffer
    }

    if (data != null) {
      this.startGame(data)
    }
  }

  async initWasm() {
    this.wasm = await init(wasmData)
    this.emulator = new WebEmulator()
  }

  startGame(data: ArrayBuffer) {
    if (this.emulator != null) {
      const byteArr = new Uint8Array(data)
      this.emulator.load_rom(byteArr)


      this.video.setEmulator(this.emulator)
      this.video.setMemory(this.wasm)

      this.audio.setEmulator(this.emulator)
      this.audio.setMemory(this.wasm)

      console.log('playing samples')
      this.audio.playSamples()

      requestAnimationFrame((time) => this.runFrame(time))
    }
  }

  runFrame(time: number) {
    const diff = time - this.previousTime

    if (diff >= FPS_INTERVAL || this.previousTime == 0) {
      this.emulator!.step_frame()
      this.video.updateCanvas()

      this.previousTime = time - (diff % FPS_INTERVAL)
    }

    requestAnimationFrame((time) => this.runFrame(time))
  }

  readFile(file: File): Promise<ArrayBuffer> {
    const fileReader = new FileReader()

    return new Promise((resolve, reject) => {
      fileReader.onload = () => resolve(fileReader.result as ArrayBuffer)

      fileReader.onerror = () => {
        fileReader.abort()
        reject(new Error("Error parsing file"))
      }

      fileReader.readAsArrayBuffer(file)
    })
  }

  addEventListeners() {
    const loadGame = document.getElementById('game-button')
    const gameInput = document.getElementById('game-input')

    if (loadGame != null && gameInput != null) {
      gameInput.onchange = (ev) => {
        const files = (ev.target as HTMLInputElement)?.files

        if (files != null) {
          const file = files[0]

          this.onGameChange(file)
        }
      }

      loadGame.onclick = (ev) => {
        if (gameInput != null) {
          gameInput.click()
        }
      }
    }

    document.onkeydown = (ev) => {
      switch (ev.key) {
        case 'Escape':
          const savesModal = document.getElementById('saves-modal')

          if (savesModal != null) {
            savesModal.className = 'modal hide'
            savesModal.style.display = 'none'
          }

          const helpModal = document.getElementById('help-modal')

          if (helpModal != null) {
            helpModal.className = 'modal hide'
            helpModal.style.display = 'none'
          }
          break
      }
    }
  }
}