import { WebEmulator } from "../../../pkg/gb_plus_web"
import { GBC } from "../gbc"
import { StateManager } from "../saves/state_manager"

const BUTTON_A = 0
const BUTTON_B = 2
// const L2 = 6
// const R2 = 7
const SELECT = 8
const START = 9
const LEFT_STICK = 10
const RIGHT_STICK = 11
const UP = 12
const DOWN = 13
const LEFT = 14
const RIGHT = 15


export class Joypad {
  pressedKeys = new Map<number, number>()
  gbc: GBC
  keyMap = new Map<number, boolean>()
  stateManager: StateManager|null = null

  keyMappings = new Map<string, number>([
    ["k", BUTTON_A],
    ["j", BUTTON_B],
    ["s", DOWN],
    ["a", LEFT],
    ["d", RIGHT],
    ["w", UP],
    ["Tab", SELECT],
    ["Enter", START]
  ])

  buttonToKeys = new Map<string, string>([
    ["up", "w"],
    ["down", "s"],
    ["left", "a"],
    ["right", "d"],
    ["select", "tab"],
    ["enter", "start"]
  ])

  joypadMappings = new Map<number, number>([
    [BUTTON_A, BUTTON_A],
    [BUTTON_B, BUTTON_B],
    [DOWN, DOWN],
    [LEFT, LEFT],
    [RIGHT, RIGHT],
    [UP, UP],
    [SELECT, SELECT],
    [START, START]
  ])

  private currentKeyInput: HTMLInputElement|null = null

  setStateManager(stateManager: StateManager) {
    this.stateManager = stateManager
  }

  constructor(gbc: GBC) {
    this.gbc = gbc

    const keyMappings = localStorage.getItem("gbc-key-mappings")

    if (keyMappings != null) {
      this.keyMappings = new Map(JSON.parse(keyMappings))
    }

    const buttonToKeys = localStorage.getItem("gbc-button-to-keys")

    if (buttonToKeys != null) {
      this.buttonToKeys = new Map(JSON.parse(buttonToKeys))
    }

    const keyInputs = document.getElementsByClassName("key-input")

    for (const keyInput of keyInputs) {
      const keyEl = keyInput as HTMLInputElement

      const sibling = keyEl.previousElementSibling as HTMLElement

      const gbButton = sibling.innerText.toLowerCase()

      const key = this.buttonToKeys.get(gbButton)

      if (key != null) {
        keyEl.value = key
      }

      keyInput.addEventListener("focus", (e) => {
        this.currentKeyInput = keyEl
        this.currentKeyInput.className += " is-warning"

        this.currentKeyInput.placeholder = "Enter key..."
      })
    }

    document.addEventListener('keydown', async (e) => {
      if (this.currentKeyInput != null) {
        if (["Meta", "Escape", "F2", "F4", "F5", "F7", "Alt", "Ctrl"].includes(e.key)) {
          if (e.key == "Escape") {
            this.cancelKeyboardMappings()
          }
          return
        }
        e.preventDefault()

        this.currentKeyInput.value = e.key.toLowerCase()
        this.currentKeyInput.className = "input is-success key-input"
        this.currentKeyInput.readOnly = true

        const nextDiv = this.currentKeyInput.parentElement!.nextElementSibling

        if (nextDiv != null) {
          const child = nextDiv.children[1] as HTMLInputElement

          if (child != null) {
            child.focus()
            this.currentKeyInput = child
          } else {
            this.currentKeyInput = null
          }
        } else {
           this.currentKeyInput = null
        }
      } else {
        const button = this.keyMappings.get(e.key.toLowerCase())
        if (button != null) {
          e.preventDefault()
          this.keyMap.set(button, true)
        }
      }
    })

    document.addEventListener('keyup', (e) => {
      const button = this.keyMappings.get(e.key.toLowerCase())
        if (button != null) {
          e.preventDefault()
          this.keyMap.set(button, false)
        }
    })
  }

  cancelKeyboardMappings() {
    const keyInputs = document.getElementsByClassName("key-input")

    for (const keyInput of keyInputs) {
      const keyEl = keyInput as HTMLInputElement

      keyEl.className = "input is-link key-input"
      keyEl.readOnly = false

      const sibling = keyEl.previousElementSibling as HTMLElement

      const gbButton = sibling.innerText.toLowerCase()

      keyEl.value = this.buttonToKeys.get(gbButton)!
    }

    const modal = document.getElementById("controller-mappings-modal")!

    modal.style.display = "none"
    modal.className = "modal hide"
  }

  updateKeyboardMappings(e: Event) {
    e.preventDefault()

    const keyInputs = document.getElementsByClassName("key-input")

    for (const keyInput of keyInputs) {
      const keyEl = keyInput as HTMLInputElement

      keyEl.readOnly = false
      keyEl.className = "input is-link key-input"

      const sibling = keyEl.previousElementSibling as HTMLElement

      const gbButton = sibling.innerText

      this.buttonToKeys.set(gbButton.toLowerCase(), keyEl.value)

      switch (gbButton.toLowerCase()) {
        case 'up':
          this.keyMappings.set(keyEl.value, UP)
          break
        case 'down':
          this.keyMappings.set(keyEl.value, DOWN)
          break
        case 'left':
          this.keyMappings.set(keyEl.value, LEFT)
          break
        case 'right':
          this.keyMappings.set(keyEl.value, RIGHT)
          break
        case 'select':
          this.keyMappings.set(keyEl.value, SELECT)
          break
        case 'start':
          this.keyMappings.set(keyEl.value, START)
          break
        case 'a':
          this.keyMappings.set(keyEl.value, BUTTON_A)
          break
        case 'b':
          this.keyMappings.set(keyEl.value, BUTTON_B)
          break
      }
    }

    const el = document.getElementById("key-mapping-notification")!

    el.style.display = "block"

    let opacity = 1.0

    const interval = setInterval(() => {
      opacity -= .1

      el.style.opacity = `${opacity}`

      if (opacity <= 0) {
        el.style.display = "none"
        clearInterval(interval)
      }
    }, 150)

    localStorage.setItem("gbc-key-mappings", JSON.stringify(Array.from(this.keyMappings.entries())))
    localStorage.setItem("gbc-button-to-keys", JSON.stringify(Array.from(this.buttonToKeys.entries())))

    const modal = document.getElementById("controller-mappings-modal")!

    modal.style.display = "none"
    modal.className = "modal hide"
  }

  async handleInput() {
    const gamepad = navigator.getGamepads()[0]

    if (this.gbc.emulator != null) {
      this.gbc.emulator.update_input(BUTTON_A, gamepad?.buttons[BUTTON_A].pressed == true || this.keyMap.get(BUTTON_A) == true)
      this.gbc.emulator.update_input(BUTTON_B, gamepad?.buttons[BUTTON_B].pressed == true || this.keyMap.get(BUTTON_B) == true)
      this.gbc.emulator.update_input(SELECT, gamepad?.buttons[SELECT].pressed == true || this.keyMap.get(SELECT) == true)
      this.gbc.emulator.update_input(START, gamepad?.buttons[START].pressed == true || this.keyMap.get(START) == true)
      this.gbc.emulator.update_input(UP, gamepad?.buttons[UP].pressed == true || this.keyMap.get(UP) == true)
      this.gbc.emulator.update_input(DOWN, gamepad?.buttons[DOWN].pressed == true || this.keyMap.get(DOWN) == true)
      this.gbc.emulator.update_input(LEFT, gamepad?.buttons[LEFT].pressed == true || this.keyMap.get(LEFT) == true)
      this.gbc.emulator.update_input(RIGHT, gamepad?.buttons[RIGHT].pressed == true || this.keyMap.get(RIGHT) == true)

      if (gamepad?.buttons[LEFT_STICK].pressed) {
        this.gbc.createSaveState(true)
      }

      if (gamepad?.buttons[RIGHT_STICK].pressed) {
        const compressed = await this.gbc.db.loadSaveState(this.gbc.gameName)

        if (compressed != null) {
          this.gbc.loadSaveState(compressed)
        }
      }
    }
  }
}