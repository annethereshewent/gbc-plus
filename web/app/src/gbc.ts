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
import { reactive } from './util/reactive'

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
  private realPreviousTime = 0
  private joypad: Joypad = new Joypad(this)
  private waveVisualizer = new WaveformVisualizer(this.plotCanvas)
  private showWaveform = false
  private updateSaveGame = ""

  private fps = 0

  private cloudService = new CloudService()

  private saveName = reactive("")

  private rtcName = ""
  gameName = ""
  private isPaused = false

  db = new GbcDatabase()
  private stateManager: StateManager|null = null

  private timeoutIndex: any|null = null

  private frameNumber: number = -1

  private romData = new Uint8Array()

  private palette = 1

  private frames = 0

  private rtcInterval: any|null = null

  constructor() {
    const palette = localStorage.getItem('dmg-palette')

    if (palette != null) {
      this.palette = parseInt(palette)
    }

    (document.getElementById("upload-save") as HTMLInputElement).style.display = "none"

    this.saveName.subscribe(() => {
      if (this.saveName.value != "" && this.cloudService.loggedIn.value) {
        const data = JSON.parse(localStorage.getItem(this.saveName.value) || '[]')

        if (data.length == 0) {
          return
        }

        (document.getElementById("upload-save") as HTMLInputElement).style.display = "block"
      } else {
        (document.getElementById("upload-save") as HTMLInputElement).style.display = "none"
      }
    })
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

    let data = null
    if (extension.toLowerCase() == 'zip') {
      const zipFile = await JSZip.loadAsync(file)

      const zipFileName = Object.keys(zipFile.files)[0]

      this.gameName = zipFileName

      const zipTokens = zipFileName.split('.')

      zipTokens.pop()

      saveName = zipTokens.join('.') + ".sav"

      data = await zipFile?.file(zipFileName)?.async('arraybuffer')
    } else if (['gb', 'gbc'].includes(extension.toLowerCase())) {
      data = await this.readFile(file) as ArrayBuffer
    }

    if (data != null) {
      this.saveName.value = saveName

      this.startGame(data)
    }
  }

  async initWasm() {
    this.wasm = await init(wasmData)
    this.emulator = new WebEmulator()
  }

  toggleFullscreen() {
    if (document.fullscreenElement == null) {
      document.documentElement.requestFullscreen()
    } else {
      document.exitFullscreen()
    }
  }

  async startGame(data: ArrayBuffer) {
    clearInterval(this.rtcInterval)
    if (this.frameNumber != -1) {
      this.emulator = new WebEmulator()
      cancelAnimationFrame(this.frameNumber)
    }
    if (this.emulator != null) {
      const byteArr = new Uint8Array(data)

      this.romData = byteArr

      this.emulator.load_rom(byteArr)

      if (this.emulator.has_timer()) {
        this.rtcName = this.saveName.value.replace(/\.sav$/, '.rtc')

        this.fetchRtc()
      }

      const saveBuffer =
        this.cloudService.loggedIn.value ?
        (await this.cloudService.getFile(this.saveName.value)).data :
        new Uint8Array(JSON.parse(localStorage.getItem(this.saveName.value) || "null"))

      if (saveBuffer != null) {
        this.emulator.load_save(saveBuffer as Uint8Array)
      }

      this.audio = new AudioInterface()

      this.stateManager = new StateManager(this.emulator, this.wasm, this.gameName, this.db)

      this.video.setEmulator(this.emulator)
      this.video.setMemory(this.wasm)

      this.audio.setEmulator(this.emulator)
      this.audio.setMemory(this.wasm)

      this.joypad.setStateManager(this.stateManager)

      this.rtcInterval = setInterval(() => {
        this.updateRtc()
      }, 30 * 60 * 1000)

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
    if (this.saveName.value != "") {
      const dataPointer = this.emulator!.save_game()
      const saveLength = this.emulator!.get_save_length()

      if (saveLength > 0) {
        const data = new Uint8Array(this.wasm!.memory.buffer, dataPointer, saveLength)

        const saveArr = Array.from(data)
        // need to do this for uploading to the cloud, otherwise it will try to upload the emulator's
        // entire memory
        const uint8Clone = new Uint8Array(saveArr)

        if (!this.cloudService.loggedIn.value) {
          localStorage.setItem(this.saveName.value, JSON.stringify(saveArr))
        } else {
          this.cloudService.uploadFile(this.saveName.value, uint8Clone)
        }
      }
    }
  }

  updateFps() {
    document.getElementById("fps-counter")!.innerText = `${this.fps} FPS`
  }

  runFrame(time: number) {
    const diff = time - this.previousTime
    if (!this.isPaused) {
      const realDiff = time - this.realPreviousTime
      this.fps = Math.floor(1000 / realDiff)

      if (this.frames >= 60) {
        this.frames -= 60

        this.updateFps()
      }

      if (this.emulator!.is_rtc_dirty()) {
        this.emulator!.clear_rtc_dirty()
        this.updateRtc()
      }

      this.realPreviousTime = time
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

      this.frames++
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

  async fetchRtc() {
    if (this.rtcName != "") {
      if (this.cloudService.loggedIn.value) {
        const rtc = await this.cloudService.getFile(this.rtcName, false)

        if (rtc.data != null) {
          this.loadRtc(JSON.stringify(rtc.data))
        } else {
          this.updateRtc()
        }
      } else {
        let json = localStorage.getItem(this.rtcName) || ""

        if (json === "") {
          this.updateRtc()
        } else  {
          this.loadRtc(json)
        }
      }
    }
  }

  updateRtc() {
    const json = this.emulator!.fetch_rtc()

    if (json !== "" && this.rtcName != null) {
      this.cloudService.loggedIn ? this.cloudService.uploadFile(this.rtcName, null, json) : localStorage.setItem(this.rtcName, json)
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
    if (!this.cloudService.loggedIn.value) {
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

        spanEl.innerText = save.filename.length > 50 ? save.filename.substring(0, 50) + "..." : save.filename

        const deleteSaveEl = document.createElement('i')

        deleteSaveEl.className = "fa-solid fa-x save-icon delete-save"

        deleteSaveEl.addEventListener('click', () => this.deleteSave(save.filename))

        const updateSaveEl = document.createElement('i')

        updateSaveEl.className = "fa-solid fa-file-pen save-icon update"

        updateSaveEl.addEventListener("click", () => this.updateSave(save.filename))

        const downloadSaveEl = document.createElement("div")

        downloadSaveEl.className = "fa-solid fa-download save-icon download"

        downloadSaveEl.addEventListener("click", () => this.downloadSave(save.filename))

        divEl.append(spanEl)
        divEl.append(downloadSaveEl)
        divEl.append(deleteSaveEl)
        divEl.append(updateSaveEl)

        savesList.append(divEl)
      }
    }
  }

  showSaveNotification() {
    const notification = document.getElementById("save-notification")

    if (notification != null) {
      notification.style.display = "block"

      let opacity = 1.0

      let interval = setInterval(() => {
        opacity -= 0.05
        notification.style.opacity = `${opacity}`

        if (opacity <= 0) {
          clearInterval(interval)

          notification.style.display = "none"
        }
      }, 100)
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
    if (!this.cloudService.loggedIn.value) {
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
        this.cloudService.uploadFile(this.updateSaveGame, bytes)
      }

      this.showSaveNotification()

      const savesModal = document.getElementById("saves-modal")

      if (savesModal != null) {
        savesModal.style.display = "none"
        savesModal.className = "modal hide"
      }
    }
  }

  async downloadSave(gameName: string) {
    if (!this.cloudService.loggedIn.value) {
      return
    }
    const entry = await this.cloudService.getFile(gameName)

    if (entry != null && entry.data != null) {
      this.generateFile(entry.data as Uint8Array, gameName)
    }
  }

  updateSave(gameName: string) {
    this.updateSaveGame = gameName

    document.getElementById("save-input")?.click()
  }

  async deleteSave(gameName: string) {
    if (this.cloudService.loggedIn.value && confirm("are you sure you want to delete this save?")) {
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

        const notification = document.getElementById("state-notification")

        if (notification != null) {
          notification.style.display = "block"

          let opacity = 1.0

          let interval = setInterval(() => {
            opacity -= 0.05
            notification.style.opacity = `${opacity}`

            if (opacity <= 0) {
              notification.style.display = "none"
              clearInterval(interval)
            }
          }, 100)
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

  async uploadSave() {
    if (this.saveName.value != "" && this.cloudService.loggedIn.value) {
      if (confirm(
        `Are you sure you want to upload your local save? This will overwrite your existing data
        and delete your local save.
        `
      )) {
        const saveArr = JSON.parse(localStorage.getItem(this.saveName.value) ?? "[]")

        if (saveArr.length > 0) {
          const saveData = new Uint8Array(saveArr)

          await this.cloudService.uploadFile(this.saveName.value, saveData)

          this.showSaveNotification()

          localStorage.removeItem(this.saveName.value)
        }
      }
    }
  }

  showControllerMappingsModal() {
    this.emulator!.set_pause(true)
    const el = document.getElementById("controller-mappings-modal")!

    el.style.display = 'block'
    el.className = 'modal show'
  }

  showKeyboardTab() {
    this.showTab("keyboard-mappings-form", "controller-mappings-form", "keyboard-tab", "controller-tab")
  }

  showControllerTab() {
    this.showTab("controller-mappings-form", "keyboard-mappings-form", "controller-tab", "keyboard-tab")
  }

  showTab(formElId: string, hiddenElId: string, activeTab: string, hiddenTab: string) {
    const formEl = document.getElementById(formElId)!

    formEl.style.display = "block"

    const hiddenEl = document.getElementById(hiddenElId)!

    hiddenEl.style.display = "none"

    document.getElementById(activeTab)!.className += "is-active"

    document.getElementById(hiddenTab)!.className = document.getElementById(hiddenTab)!.className.replace("is-active", "").trim()
  }

  addEventListeners() {
    const loadGame = document.getElementById('game-button')
    const gameInput = document.getElementById('game-input')

    const closeButtons = document.getElementsByClassName("modal-close")

    if (closeButtons != null) {
      for (const closeButton of closeButtons) {
        closeButton.addEventListener("click", () => {
          this.emulator!.set_pause(false)
          const modals = document.getElementsByClassName("modal")

          this.joypad.cancelMappings()

          if (modals != null) {
            for (const modal of modals) {
              // need that semi-colon here because javascript thinks the next line is a function call otherwise.
              // good old javascript!
              (modal as HTMLElement).style.display = "none";
              (modal as HTMLElement).className = "modal hide"
            }
          }
        })
      }
    }

    document.getElementById("hide-saves-modal")?.addEventListener("click", () => this.hideSavesModal())
    document.getElementById("save-states")?.addEventListener("click", () => this.displaySaveStatesModal())
    document.getElementById("create-save-state")?.addEventListener("click", () => this.createSaveState())
    document.getElementById("save-management")?.addEventListener("click", () => this.displaySavesModal())
    document.getElementById("fullscreen")?.addEventListener("click", () => this.toggleFullscreen())
    document.getElementById("dmg-color-palettes-item")?.addEventListener("click", () => this.showColorPalettes())
    document.getElementById('hide-palettes-modal')?.addEventListener("click", () => this.hidePalettesModal())
    document.getElementById("save-input")?.addEventListener("change", (e) => this.handleSaveChange(e))
    document.getElementById("upload-save")?.addEventListener("click", () => this.uploadSave())
    document.getElementById("controller-mappings")?.addEventListener("click", () => this.showControllerMappingsModal())
    document.getElementById("mappings-cancel-button")?.addEventListener("click", () => this.joypad.cancelMappings())
    document.getElementById("keyboard-tab")?.addEventListener("click", () => this.showKeyboardTab())
    document.getElementById("controller-tab")?.addEventListener("click", () => this.showControllerTab())

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

    const waveformButton = document.getElementById("waveform-visualizer-button")!

    waveformButton.addEventListener('click', () => {
      this.toggleWavePlot()
    })

    document.onkeydown = (ev) => {
      switch (ev.key) {
        case 'Escape':
          ev.preventDefault()

          this.emulator?.set_pause(false)

          const modals = document.getElementsByClassName('modal')

          for (const modal of modals) {
            const modalEl = modal as HTMLElement
            modalEl.className = 'modal hide'
            modalEl.style.display = 'none'
          }

          break
        case 'F4':
          this.toggleWavePlot()
          break
        case 'F2':
          this.palette = (this.palette + 1) % PALETTES.length

          this.emulator?.change_palette(this.palette)

          localStorage.setItem('dmg-palette', this.palette.toString())
          break
      }
    }
  }
}