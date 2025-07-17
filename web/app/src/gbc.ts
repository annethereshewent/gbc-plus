import JSZip from 'jszip'
import init, { WebEmulator, InitOutput } from "../../pkg/gb_plus_web"
import wasmData from '../../pkg/gb_plus_web_bg.wasm'
import { VideoInterface } from './output/video_interface'
import { AudioInterface } from './output/audio_interface'
import { Joypad } from './input/joypad'

const FPS_INTERVAL = 1000 / 60

export class GBC {
  private emulator: WebEmulator|null = null
  private wasm: InitOutput|null = null
  private canvas: HTMLCanvasElement = document.getElementById('game-canvas')! as HTMLCanvasElement
  private context = this.canvas.getContext("2d")
  private video: VideoInterface = new VideoInterface(this.canvas, this.context!)
  private audio: AudioInterface|null = null
  private previousTime = 0
  private joypad: Joypad = new Joypad()

  private saveName = ""
  private rtcName = ""

  private timeoutIndex: any|null = null

  async onGameChange(file: File) {
    const fileName = file.name

    const tokens = fileName.split('/')

    let gameName = tokens.pop()!

    const gameNameTokens = gameName.split('.')

    const extension = gameNameTokens.pop()!

    gameName = gameNameTokens.join('.')

    let saveName = gameName + ".sav"

    let data = null
    if (extension.toLowerCase() == 'zip') {
      const zipFile = await JSZip.loadAsync(file)

      const zipFileName = Object.keys(zipFile.files)[0]

      const zipTokens = zipFileName.split('.')

      zipTokens.pop()

      saveName = zipTokens.join('.') + ".sav"

      data = await zipFile?.file(zipFileName)?.async('arraybuffer')
    } else if (['gb', 'gbc'].includes(extension.toLowerCase())) {
      data = await this.readFile(file) as ArrayBuffer
    }

    if (data != null) {
      this.saveName = saveName

      // check if save exists in localStorage
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

      if (this.emulator.has_timer()) {
        this.rtcName = this.saveName.replace(/\.sav$/, '.rtc')

        this.fetchRtc()
      }

      const saveArr = JSON.parse(localStorage.getItem(this.saveName) || '[]')

      if (saveArr.length > 0) {
        const saveBuffer = new Uint8Array(saveArr)
        this.emulator!.load_save(saveBuffer)
      }


      this.audio = new AudioInterface()

      this.video.setEmulator(this.emulator)
      this.video.setMemory(this.wasm)

      this.audio.setEmulator(this.emulator)
      this.audio.setMemory(this.wasm)

      this.joypad.setEmulator(this.emulator)

      setInterval(() => {
        this.updateRtc()
      }, 5 * 60 * 1000)

      requestAnimationFrame((time) => this.runFrame(time))
    }
  }

  checkSaveGame() {
    if (this.emulator!.has_saved()) {
      clearTimeout(this.timeoutIndex)
      this.timeoutIndex = setTimeout(() => this.saveGame(), 1000)
    }
  }

  saveGame() {
    if (this.saveName != "") {
      const dataPointer = this.emulator!.save_game()
      const saveLength = this.emulator!.get_save_length()

      if (saveLength > 0) {
        const data = new Uint8Array(this.wasm!.memory.buffer, dataPointer, saveLength)

        const saveArr = Array.from(data)

        localStorage.setItem(this.saveName, JSON.stringify(saveArr))
      }
    }
  }

  runFrame(time: number) {
    const diff = time - this.previousTime

    this.audio!.pushSamples()

    if (diff >= FPS_INTERVAL || this.previousTime == 0) {
      this.emulator!.step_frame()
      this.video.updateCanvas()

      this.joypad.handleInput()
      this.checkSaveGame()

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

  loadRtc(json: string) {
    this.emulator!.load_rtc(json)
  }

  fetchRtc() {
    if (this.rtcName != "") {
      let json = localStorage.getItem(this.rtcName) || ""

      if (json === "") {
        this.updateRtc()
      } else  {
        this.loadRtc(json)
      }
    }
  }

  updateRtc() {
    const json = this.emulator!.fetch_rtc()

    if (json !== "" && this.rtcName != null) {
      localStorage.setItem(this.rtcName, json)
    }
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