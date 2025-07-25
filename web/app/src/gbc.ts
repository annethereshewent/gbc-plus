import JSZip from 'jszip'
import init, { WebEmulator, InitOutput } from "../../pkg/gb_plus_web"
import wasmData from '../../pkg/gb_plus_web_bg.wasm'
import { SCREEN_HEIGHT, SCREEN_WIDTH, VideoInterface } from './output/video_interface'
import { AudioInterface } from './output/audio_interface'
import { Joypad } from './input/joypad'
import { WaveformVisualizer } from './util/waveform_visualizer'
import { CloudService } from './saves/cloud_service'
import moment from 'moment'
import { StateEntry } from './interface/game_state_entry'
import { GbcDatabase } from './saves/gbc_database'
import { StateManager } from './saves/state_manager'

const FPS_INTERVAL = 1000 / 60

const PALETTES = [
  {
    name: 'Classic green',
    class: 'classic-green',
    color: '#489848'
  },
  {
    name: 'Grayscale',
    class: 'grayscale',
    color: '#555555'
  },
  {
    name: 'Solarized',
    class: 'solarized',
    color: '#586e75'
  },
  {
    name: 'Maverick',
    class: 'maverick',
    color: '#306850'
  },
  {
    name: 'Oceanic',
    class: 'oceanic',
    color: '#0074d9'
  },
  {
    name: 'Burnt peach',
    class: 'burnt-peach',
    color: '#803d26'
  },
  {
    name: 'Grape soda',
    class: 'grape-soda',
    color: '#5c258d'
  },
  {
    name: 'Strawberry milk',
    class: 'strawberry-milk',
    color: '#e175a4'
  },
  {
    name: 'Witching hour',
    class: 'witching-hour',
    color: '#4b0082'
  },
  {
    name: 'Void dream',
    class: 'void-dream',
    color: '#4f86f7'
  }
]

export class GBC {
  emulator: WebEmulator|null = null
  private wasm: InitOutput|null = null
  private canvas: HTMLCanvasElement = document.getElementById('game-canvas')! as HTMLCanvasElement
  private plotCanvas: HTMLCanvasElement = document.getElementById('waveform-visualizer')! as HTMLCanvasElement
  private context = this.canvas.getContext("2d")
  private video: VideoInterface = new VideoInterface(this.canvas, this.context!)
  private audio: AudioInterface|null = null
  private previousTime = 0
  private joypad: Joypad = new Joypad(this)
  private waveVisualizer = new WaveformVisualizer(this.plotCanvas)
  private showWaveform = false
  private updateSaveGame = ""
  private fullScreen = false

  private cloudService = new CloudService()

  private saveName = ""
  private rtcName = ""
  gameName = ""
  private isPaused = false

  db = new GbcDatabase()
  private stateManager: StateManager|null = null

  private timeoutIndex: any|null = null

  private frameNumber: number = -1

  private romData = new Uint8Array()

  private palette = 1

  constructor() {
    const palette = localStorage.getItem('dmg-palette')

    if (palette != null) {
      this.palette = parseInt(palette)
    }
  }

  checkOauth() {
    this.cloudService.checkAuthentication()
  }

  async onGameChange(file: File) {
    const fileName = file.name

    const tokens = fileName.split('/')

    let gameName = tokens.pop()!

    this.gameName = gameName

    const gameNameTokens = gameName.split('.')

    const extension = gameNameTokens.pop()!

    gameName = gameNameTokens.join('.')

    let saveName = gameName + ".sav"
    this.rtcName = gameName + ".rtc"

    let data = null
    if (extension.toLowerCase() == 'zip') {
      const zipFile = await JSZip.loadAsync(file)

      const zipFileName = Object.keys(zipFile.files)[0]

      this.gameName = zipFileName

      const zipTokens = zipFileName.split('.')

      zipTokens.pop()

      saveName = zipTokens.join('.') + ".sav"
      this.rtcName = zipTokens.join('.') + ".rtc"

      data = await zipFile?.file(zipFileName)?.async('arraybuffer')
    } else if (['gb', 'gbc'].includes(extension.toLowerCase())) {
      data = await this.readFile(file) as ArrayBuffer
    }

    if (data != null) {
      this.saveName = saveName

      this.startGame(data)
    }
  }

  async initWasm() {
    this.wasm = await init(wasmData)
    this.emulator = new WebEmulator()
  }

  toggleFullscreen() {
    if (!this.fullScreen) {
      document.documentElement.requestFullscreen()
    } else {
      document.exitFullscreen()
    }

    this.fullScreen = !this.fullScreen
  }

  async startGame(data: ArrayBuffer) {
    if (this.frameNumber != -1) {
      this.emulator = new WebEmulator()
      cancelAnimationFrame(this.frameNumber)
    }
    if (this.emulator != null) {
      const byteArr = new Uint8Array(data)

      this.romData = byteArr

      this.emulator.load_rom(byteArr)

      if (this.emulator.has_timer()) {
        this.rtcName = this.saveName.replace(/\.sav$/, '.rtc')

        this.fetchRtc()
      }

      // check if save exists and whether it's on the cloud
      const saveBuffer = this.cloudService.usingCloud ?
        (await this.cloudService.getSave(this.saveName)).data : new Uint8Array(JSON.parse(localStorage.getItem(this.saveName) || '[]'))

      if (saveBuffer != null && saveBuffer.length > 0) {
        this.emulator!.load_save(saveBuffer)
      }

      this.audio = new AudioInterface()

      this.stateManager = new StateManager(this.emulator, this.wasm, this.gameName, this.db)

      this.video.setEmulator(this.emulator)
      this.video.setMemory(this.wasm)

      this.audio.setEmulator(this.emulator)
      this.audio.setMemory(this.wasm)

      this.joypad.setStateManager(this.stateManager)

      setInterval(() => {
        this.updateRtc()
      }, 5 * 60 * 1000)

      this.emulator!.change_palette(this.palette)

      this.frameNumber = requestAnimationFrame((time) => this.runFrame(time))
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
        // need to do this for uploading to the cloud, otherwise it will try to upload the emulator's
        // entire memory
        const uint8Clone = new Uint8Array(saveArr)


        if (!this.cloudService.usingCloud) {
          localStorage.setItem(this.saveName, JSON.stringify(saveArr))
        } else {
          this.cloudService.uploadSave(this.saveName, uint8Clone)
        }
      }
    }
  }

  runFrame(time: number) {
    const diff = time - this.previousTime

    if (!this.isPaused) {
      if (diff >= FPS_INTERVAL || this.previousTime == 0) {
        const samples = this.audio!.pushSamples()
        if (this.showWaveform) {
          this.waveVisualizer.plot(samples)
        }
        this.emulator!.step_frame()
        this.video.updateCanvas()

        this.joypad.handleInput()
        this.checkSaveGame()

        this.previousTime = time - (diff % FPS_INTERVAL)
      }

      this.frameNumber = requestAnimationFrame((time) => this.runFrame(time))
    }
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

  toggleWavePlot() {
    this.showWaveform = !this.showWaveform

    let currentOpacity = 0
    let initialOpacity: number
    let delta: number

    if (this.showWaveform) {
      currentOpacity = 0.0
      initialOpacity = currentOpacity
      delta = 0.25

      this.plotCanvas.style.display = "block"
    } else {
      currentOpacity = 1.0
      initialOpacity = currentOpacity
      delta = -0.25
    }

    this.plotCanvas.style.opacity = `${initialOpacity}`

    const interval = setInterval(() => {
      currentOpacity += delta

      this.plotCanvas.style.opacity = `${currentOpacity}`

      if ((currentOpacity <= 0.0 && initialOpacity != 0.0) || (currentOpacity >= 1.0 && initialOpacity != 1.0)) {
        if (!this.showWaveform) {
          this.plotCanvas.style.display = "none"
        }
        clearInterval(interval)
      }
    }, 150)
  }

  closeStatesModal() {
    this.emulator?.set_pause(false)
    this.isPaused = false
    const statesModal = document.getElementById("states-modal")

    if (statesModal != null) {
      statesModal.className = "modal hide"
      statesModal.style.display = "none"
    }
  }

  async displaySaveStatesModal() {
    if (this.gameName != "") {
      const modal = document.getElementById("states-modal")
      const statesList = document.getElementById("states-list")

      if (modal != null && statesList != null) {
        this.emulator?.set_pause(true)
        modal.style.display = "block"

        statesList.innerHTML = ""

        const entry = await this.db.getSaveStates(this.gameName)

        if (entry != null) {
          for (const key in entry.states) {
            const stateEntry = entry.states[key]

            this.addStateElement(statesList, stateEntry)
          }
        }
      }
    }
  }

  displayMenu(stateName: string) {
    const menus = document.getElementsByClassName("state-menu") as HTMLCollectionOf<HTMLElement>

    for (const menu of menus) {
      if (menu.id.indexOf(stateName) == -1) {
        menu.style.display = "none"
      }
    }

    const menu = document.getElementById(`menu-${stateName}`)

    if (menu != null) {
      if (menu.style.display == "block") {
        menu.style.display = "none"
      } else {
        menu.style.display = "block"
      }
    }
  }

  async updateState(entry: StateEntry) {
    const imageUrl = this.getImageUrl()
    if (imageUrl != null && this.stateManager != null) {
      const oldStateName = entry.stateName

      const updateEntry = await this.stateManager.createSaveState(imageUrl, entry.stateName, true)

      if (updateEntry != null) {
        this.updateStateElement(updateEntry, oldStateName)
      }
    }
  }

  updateStateElement(entry: StateEntry, oldStateName: string) {
    const image = document.getElementById(`image-${oldStateName}`) as HTMLImageElement
    const title = document.getElementById(`title-${oldStateName}`)

    if (image != null && title != null) {
      image.src = entry.imageUrl

      if (entry.stateName != "quick_save.state") {
        const timestamp = parseInt(entry.stateName.replace(".state", ""))

        title.innerText = `Save on ${moment.unix(timestamp).format("lll")}`
      }
    }
  }

  getImageUrl() {
    if (this.emulator != null && this.wasm != null) {
      let screen = new Uint8Array(SCREEN_WIDTH * SCREEN_HEIGHT * 4)
      screen = new Uint8Array(this.wasm.memory.buffer, this.emulator.get_screen(), SCREEN_WIDTH * SCREEN_HEIGHT * 4)
      const canvas = document.getElementById("save-state-canvas") as HTMLCanvasElement

      const context = canvas.getContext("2d")

      if (context != null) {
        const imageData = context.getImageData(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)

        let screenIndex = 0
        for (let i = 0; i < screen.length; i += 4) {
          imageData.data[i] = screen[screenIndex]
          imageData.data[i + 1] = screen[screenIndex + 1]
          imageData.data[i + 2] = screen[screenIndex + 2]
          imageData.data[i + 3] = screen[screenIndex + 3]

          screenIndex += 4
        }

        context.putImageData(imageData, 0, 0)

        return canvas.toDataURL()
      }
    }

    return null
  }

  addStateElement(statesList: HTMLElement, entry: StateEntry) {
    const divEl = document.createElement("div")

    divEl.className = "state-element"
    divEl.id = entry.stateName

    divEl.addEventListener("click", () => this.displayMenu(entry.stateName))

    const imgEl = document.createElement("img")

    imgEl.className = "state-image"
    imgEl.id = `image-${entry.stateName}`

    const pEl = document.createElement("p")
    pEl.id = `title-${entry.stateName}`

    if (entry.stateName != "quick_save.state") {

      const timestamp = parseInt(entry.stateName.replace(".state", ""))

      pEl.innerText = `Save on ${moment.unix(timestamp).format("lll")}`
    } else {
      pEl.innerText = "Quick save"
    }

    const menu = document.createElement("aside")

    menu.className = "state-menu hide"
    menu.id = `menu-${entry.stateName}`
    menu.style.display = "none"

    menu.innerHTML = `
      <ul class="state-menu-list">
        <li><a id="update-${entry.stateName}">Update State</a></li>
        <li><a id="load-${entry.stateName}">Load state</a></li>
        <li><a id="delete-${entry.stateName}">Delete state</a></li>
      </ul>
    `
    imgEl.src = entry.imageUrl


    divEl.append(imgEl)
    divEl.append(pEl)
    divEl.append(menu)

    statesList.append(divEl)

    // finally add event listeners for loading and deleting states
    document.getElementById(`update-${entry.stateName}`)?.addEventListener("click", () => this.updateState(entry))
    document.getElementById(`load-${entry.stateName}`)?.addEventListener("click", () => this.loadSaveState(entry.state))
    document.getElementById(`delete-${entry.stateName}`)?.addEventListener("click", () => this.deleteState(entry.stateName))
  }

  async loadSaveState(compressed: Uint8Array) {
    if (this.romData != null) {
      cancelAnimationFrame(this.frameNumber)

      if (this.stateManager != null) {
        const data = await this.stateManager.decompress(compressed)

        if (data != null) {
          this.emulator!.load_save_state(data)

          this.emulator!.reload_rom(this.romData)

          this.frameNumber = requestAnimationFrame((time) => this.runFrame(time))
        }

        this.closeStatesModal()
      }
    }
  }

  async deleteState(stateName: string) {
    if (confirm("Are you sure you want to delete this save state?")) {
      await this.db.deleteState(this.gameName, stateName)

      const el = document.getElementById(stateName)

      el?.remove()
    }
  }

  async displaySavesModal() {
    if (!this.cloudService.usingCloud) {
      return
    }
    const saves = await this.cloudService.getSaves()
    const savesModal = document.getElementById("saves-modal")
    const savesList = document.getElementById("saves-list")

    if (saves != null && savesModal != null && savesList != null) {
      savesModal.className = "modal show"
      savesModal.style.display = "block"

      this.emulator?.set_pause(true)

      savesList.innerHTML = ''
      for (const save of saves) {
        const divEl = document.createElement("div")

        divEl.className = "save-entry"

        const spanEl = document.createElement("span")

        spanEl.innerText = save.gameName.length > 50 ? save.gameName.substring(0, 50) + "..." : save.gameName

        const deleteSaveEl = document.createElement('i')

        deleteSaveEl.className = "fa-solid fa-x save-icon delete-save"

        deleteSaveEl.addEventListener('click', () => this.deleteSave(save.gameName))

        const updateSaveEl = document.createElement('i')

        updateSaveEl.className = "fa-solid fa-file-pen save-icon update"

        updateSaveEl.addEventListener("click", () => this.updateSave(save.gameName))

        const downloadSaveEl = document.createElement("div")

        downloadSaveEl.className = "fa-solid fa-download save-icon download"

        downloadSaveEl.addEventListener("click", () => this.downloadSave(save.gameName))

        divEl.append(spanEl)
        divEl.append(downloadSaveEl)
        divEl.append(deleteSaveEl)
        divEl.append(updateSaveEl)

        savesList.append(divEl)
      }
    }
  }

  generateFile(data: Uint8Array, gameName: string) {
    const blob = new Blob([data], {
      type: "application/octet-stream"
    })

    const objectUrl = URL.createObjectURL(blob)

    const a = document.createElement('a')

    a.href = objectUrl
    a.download = gameName.match(/\.sav$/) ? gameName : `${gameName}.sav`
    document.body.append(a)
    a.style.display = "none"

    a.click()
    a.remove()

    setTimeout(() => URL.revokeObjectURL(objectUrl), 1000)
  }

  async handleSaveChange(e: Event) {
    if (!this.cloudService.usingCloud) {
      return
    }
    let saveName = (e.target as HTMLInputElement)?.files?.[0].name?.split('/')?.pop()

    if (saveName != this.updateSaveGame) {
      if (!confirm("Warning! Save file doesn't match selected game name. are you sure you want to continue?")) {
        return
      }
    }


    const data = await this.readFile((e.target as HTMLInputElement).files![0]) as ArrayBuffer

    if (data != null) {
      const bytes = new Uint8Array(data as ArrayBuffer)

      if (this.updateSaveGame != "") {
        this.cloudService.uploadSave(this.updateSaveGame, bytes)
      }

      const notification = document.getElementById("save-notification")

      if (notification != null) {
        notification.style.display = "block"

        let opacity = 1.0

        let interval = setInterval(() => {
          opacity -= 0.1
          notification.style.opacity = `${opacity}`

          if (opacity <= 0) {
            clearInterval(interval)
          }
        }, 100)
      }

      const savesModal = document.getElementById("saves-modal")

      if (savesModal != null) {
        savesModal.style.display = "none"
        savesModal.className = "modal hide"
      }
    }
  }

  async downloadSave(gameName: string) {
    if (!this.cloudService.usingCloud) {
      return
    }
    const entry = await this.cloudService.getSave(gameName)

    if (entry != null) {
      this.generateFile(entry.data!!, gameName)
    }
  }

  updateSave(gameName: string) {
    this.updateSaveGame = gameName

    document.getElementById("save-input")?.click()
  }

  async deleteSave(gameName: string) {
    if (this.cloudService.usingCloud && confirm("are you sure you want to delete this save?")) {
      const result = await this.cloudService.deleteSave(gameName)

      if (result) {
        const savesList = document.getElementById("saves-list")

        if (savesList != null) {
          for (const child of savesList.children) {
            const children = [...child.children]
            const spanElement = (children.filter((childEl) => childEl.tagName.toLowerCase() == 'span')[0] as HTMLSpanElement)

            if (spanElement?.innerText == gameName) {
              child.remove()
              break
            }
          }
        }
      }
    }
  }

  changePalette(index: number) {
    this.emulator?.change_palette(index)

    localStorage.setItem('dmg-palette', index.toString())

    this.palette = index

    this.hidePalettesModal()

    this.emulator?.set_pause(false)
  }

  showColorPalettes() {
    this.emulator?.set_pause(true)

    const paletteModal = document.getElementById("color-palettes-modal")

    if (paletteModal != null) {
      const palettesDiv = document.getElementById("color-palettes")

      const ulEl = document.createElement('ul')

      ulEl.className = "palettes-list"

      if (palettesDiv != null) {
        palettesDiv.innerHTML = ''
        for (let i = 0; i < PALETTES.length; i++) {
          const palette = PALETTES[i]
          const liEl = document.createElement('li')

          const divEl = document.createElement('div')

          divEl.className = 'color-palette'

          divEl.innerText = palette.name

          const spanEl = document.createElement("span")

          spanEl.className = "palette-circle"
          spanEl.style.background = palette.color

          spanEl.addEventListener("click", () => this.changePalette(i))

          divEl.appendChild(spanEl)

          liEl.appendChild(divEl)

          const divEl2 = document.createElement('div')

          divEl2.style.clear = 'both'

          liEl.append(divEl2)

          ulEl.appendChild(liEl)
        }

        palettesDiv.appendChild(ulEl)
      }

      paletteModal.style.display = "block"
      paletteModal.className = "modal show"
    }
  }

  async createSaveState(isQuickSave = false) {
    const now = moment()

    const stateName = `${now.unix()}.state`

    if (this.gameName != "") {
      const imageUrl = this.getImageUrl()
      if (imageUrl != null) {
        const entry = isQuickSave ?
          await this.stateManager?.createSaveState(imageUrl) :
          await this.stateManager?.createSaveState(imageUrl, stateName)

        const statesList = document.getElementById("states-list")

        if (entry != null && statesList != null) {
          this.addStateElement(statesList, entry)
        }
      }
    }
  }

  hideSavesModal() {
    const savesModal = document.getElementById('saves-modal')

    if (savesModal != null) {
      savesModal.className = 'modal hide'
      savesModal.style.display = 'none'
    }
  }

  hidePalettesModal() {
    const palettesModal = document.getElementById('color-palettes-modal')

    if (palettesModal != null) {
      palettesModal.className = 'modal hide'
      palettesModal.style.display = 'none'
    }
  }

  addEventListeners() {
    const loadGame = document.getElementById('game-button')
    const gameInput = document.getElementById('game-input')

    document.getElementById("states-modal-close")?.addEventListener("click", () => this.closeStatesModal())
    document.getElementById("hide-saves-modal")?.addEventListener("click", () => this.hideSavesModal())
    document.getElementById("save-states")?.addEventListener("click", () => this.displaySaveStatesModal())
    document.getElementById("create-save-state")?.addEventListener("click", () => this.createSaveState())
    document.getElementById("save-management")?.addEventListener("click", () => this.displaySavesModal())
    document.getElementById("fullscreen")?.addEventListener("click", () => this.toggleFullscreen())
    document.getElementById("dmg-color-palettes-item")?.addEventListener("click", () => this.showColorPalettes())
    document.getElementById('hide-palettes-modal')?.addEventListener("click", () => this.hidePalettesModal())

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

    const waveformButton = document.getElementById("waveform-visualizer")!

    waveformButton.addEventListener('click', () => {
      this.toggleWavePlot()
    })

    document.onkeydown = (ev) => {
      switch (ev.key) {
        case 'Escape':
          this.emulator?.set_pause(false)
          const savesModal = document.getElementById('saves-modal')

          if (savesModal != null) {
            savesModal.className = 'modal hide'
            savesModal.style.display = 'none'
          }

          const helpModal = document.getElementById('help-modal')!

          if (helpModal != null) {
            helpModal.className = 'modal hide'
            helpModal.style.display = 'none'
          }

          const statesModal = document.getElementById('states-modal')

          if (statesModal != null) {
            statesModal.className = 'modal hide'
            statesModal.style.display = 'none'
          }

          const palettesModal = document.getElementById('color-palettes-modal')

          if (palettesModal != null) {
            palettesModal.className = 'modal hide'
            palettesModal.style.display = 'none'
          }

          break
        case 'F4':
          this.toggleWavePlot()
          break
        case 'F2':
          this.palette = (this.palette + 1) % PALETTES.length

          this.emulator?.change_palette(this.palette)

          localStorage.setItem('dmg-palette', this.palette.toString())
      }
    }
  }
}