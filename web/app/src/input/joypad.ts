import { WebEmulator } from "../../../pkg/gb_plus_web"
import { GBC } from "../gbc"
import { StateManager } from "../saves/state_manager"

const BUTTON_CROSS = 0
const BUTTON_SQUARE = 2
const L2 = 6
const R2 = 7
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

  setStateManager(stateManager: StateManager) {
    this.stateManager = stateManager
  }

  constructor(gbc: GBC) {
    this.gbc = gbc

    document.addEventListener('keydown', async (e) => {
       switch (e.key) {
        case "w":
          this.keyMap.set(UP, true)
          break
        case "a":
          this.keyMap.set(LEFT, true)
          break
        case "s":
          this.keyMap.set(DOWN, true)
          break
        case "d":
          this.keyMap.set(RIGHT, true)
          break
        case "j":
          this.keyMap.set(BUTTON_SQUARE, true)
          break
        case "k":
          this.keyMap.set(BUTTON_CROSS, true)
          break
        case "Enter":
          e.preventDefault()
          this.keyMap.set(START, true)
          break
        case "Tab":
          e.preventDefault()
          this.keyMap.set(SELECT, true)
          break
        case "F5":
          e.preventDefault()
          this.gbc.createSaveState(true)
          break
        case "F7":
          e.preventDefault()
          const compressed = await this.gbc.db.loadSaveState(this.gbc.gameName)

          if (compressed != null) {
            this.gbc.loadSaveState(compressed)
          }
          break
      }
    })

    document.addEventListener('keyup', (e) => {
       switch (e.key) {
        case "w":
          this.keyMap.set(UP, false)
          break
        case "a":
          this.keyMap.set(LEFT, false)
          break
        case "s":
          this.keyMap.set(DOWN, false)
          break
        case "d":
          this.keyMap.set(RIGHT, false)
          break
        case "j":
          this.keyMap.set(BUTTON_SQUARE, false)
          break
        case "k":
          this.keyMap.set(BUTTON_CROSS, false)
          break
        case "Enter":
          e.preventDefault()
          this.keyMap.set(START, false)
          break
        case "Tab":
          e.preventDefault()
          this.keyMap.set(SELECT, false)
          break
      }
    })
  }

  async handleInput() {
    const gamepad = navigator.getGamepads()[0]

    if (this.gbc.emulator != null) {
      this.gbc.emulator.update_input(BUTTON_CROSS, gamepad?.buttons[BUTTON_CROSS].pressed == true || this.keyMap.get(BUTTON_CROSS) == true)
      this.gbc.emulator.update_input(BUTTON_SQUARE, gamepad?.buttons[BUTTON_SQUARE].pressed == true || this.keyMap.get(BUTTON_SQUARE) == true)
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