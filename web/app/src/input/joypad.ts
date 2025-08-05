import { WebEmulator } from "../../../pkg/gb_plus_web"
import { GBC } from "../gbc"
import { StateManager } from "../saves/state_manager"

const BUTTON_CROSS = 0
const BUTTON_SQUARE = 2
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

const BUTTONS_TO_ELEMENTS = new Map([
  ["a", document.getElementById("a-button")],
  ["b", document.getElementById("b-button")],
  ["start", document.getElementById("start")],
  ["select", document.getElementById("select")],
  ["up", document.getElementById("up")],
  ["down", document.getElementById("down")],
  ["left", document.getElementById("left")],
  ["right", document.getElementById("right")]
])

export class Joypad {
  pressedKeys = new Map<number, number>()
  gbc: GBC
  keyMap = new Map<string, boolean>()
  stateManager: StateManager|null = null

  keyMappings = new Map<string, string>([
    ["k", "a"],
    ["j", "b"],
    ["s", "down"],
    ["a", "left"],
    ["d", "right"],
    ["w", "up"],
    ["Tab", "select"],
    ["Enter", "start"]
  ])

  buttonToKeys = new Map<string, string>([
    ["up", "w"],
    ["down", "s"],
    ["left", "a"],
    ["right", "d"],
    ["select", "tab"],
    ["enter", "start"],
    ["b", "j"],
    ["a", "k"]
  ])

  joypadMappings = new Map<number, string>([
    [BUTTON_CROSS, "a"],
    [BUTTON_SQUARE, "b"],
    [DOWN, "down"],
    [LEFT, "left"],
    [RIGHT, "right"],
    [UP, "up"],
    [SELECT, "select"],
    [START, "start"]
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

        if (e.key == "Escape") {
          this.cancelKeyboardMappings()
          return
        }

        if (!["Meta", "F2", "F4", "F5", "F7", "Alt"].includes(e.key)) {
          e.preventDefault()
        } else {
          this.currentKeyInput.className = "input is-link key-input"
          this.currentKeyInput.blur()
          this.currentKeyInput = null
          return
        }

        const sibling = this.currentKeyInput.previousElementSibling as HTMLElement

        const gbButton = sibling.innerText.toLowerCase()

        const oldKeyboardKey = this.currentKeyInput.value.toLowerCase()

        this.keyMappings.delete(oldKeyboardKey)

        const existingButton = this.keyMappings.get(e.key.toLowerCase())

        if (existingButton != null) {
          const oldMapping = this.buttonToKeys.get(existingButton)!

          this.keyMappings.delete(oldMapping)

          this.keyMappings.set(oldKeyboardKey, existingButton)
          this.buttonToKeys.set(existingButton, oldKeyboardKey)

          const element = BUTTONS_TO_ELEMENTS.get(existingButton) as HTMLInputElement

          if (element != null) {
            element.value = oldKeyboardKey
          }
        }

        this.keyMappings.set(e.key.toLowerCase(), gbButton.toLowerCase())
        this.buttonToKeys.set(gbButton.toLowerCase(), e.key.toLowerCase())

        localStorage.setItem('gbc-key-mappings', JSON.stringify(Array.from(this.keyMappings.entries())))
        localStorage.setItem("gbc-button-to-keys", JSON.stringify(Array.from(this.buttonToKeys.entries())))

        this.currentKeyInput.value = e.key.toLowerCase()
        this.currentKeyInput.className = "input is-success key-input"

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
    }

    const modal = document.getElementById("controller-mappings-modal")!

    modal.style.display = "none"
    modal.className = "modal hide"
  }

  async handleInput() {
    const gamepad = navigator.getGamepads()[0]

    if (this.gbc.emulator != null) {
      this.gbc.emulator.update_input("a", gamepad?.buttons[BUTTON_CROSS].pressed == true || this.keyMap.get("a") == true)
      this.gbc.emulator.update_input("b", gamepad?.buttons[BUTTON_SQUARE].pressed == true || this.keyMap.get("b") == true)
      this.gbc.emulator.update_input("select", gamepad?.buttons[SELECT].pressed == true || this.keyMap.get("select") == true)
      this.gbc.emulator.update_input("start", gamepad?.buttons[START].pressed == true || this.keyMap.get("start") == true)
      this.gbc.emulator.update_input("up", gamepad?.buttons[UP].pressed == true || this.keyMap.get("up") == true)
      this.gbc.emulator.update_input("down", gamepad?.buttons[DOWN].pressed == true || this.keyMap.get("down") == true)
      this.gbc.emulator.update_input("left", gamepad?.buttons[LEFT].pressed == true || this.keyMap.get("left") == true)
      this.gbc.emulator.update_input("right", gamepad?.buttons[RIGHT].pressed == true || this.keyMap.get("right") == true)

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