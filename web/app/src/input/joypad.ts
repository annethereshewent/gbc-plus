import { WebEmulator } from "../../../pkg/gb_plus_web"

const BUTTON_CROSS = 0
const BUTTON_SQUARE = 2
const SELECT = 8
const START = 9
const UP = 12
const DOWN = 13
const LEFT = 14
const RIGHT = 15



export class Joypad {
  pressedKeys = new Map<number, number>()
  emulator: WebEmulator|null = null
  keyMap = new Map<number, boolean>()

  setEmulator(emulator: WebEmulator) {
    this.emulator = emulator
  }

  constructor() {
    document.addEventListener('keydown', (e) => {
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
          this.keyMap.set(BUTTON_CROSS, true)
          break
        case "k":
          this.keyMap.set(BUTTON_SQUARE, true)
          break
        case "Enter":
          e.preventDefault()
          this.keyMap.set(START, true)
          break
        case "Tab":
          e.preventDefault()
          this.keyMap.set(SELECT, true)
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
          this.keyMap.set(BUTTON_CROSS, false)
          break
        case "k":
          this.keyMap.set(BUTTON_SQUARE, false)
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

  handleInput() {
    const gamepad = navigator.getGamepads()[0]

    if (this.emulator != null) {
      this.emulator.update_input(BUTTON_CROSS, gamepad?.buttons[BUTTON_CROSS].pressed == true || this.keyMap.get(BUTTON_CROSS) == true)
      this.emulator.update_input(BUTTON_SQUARE, gamepad?.buttons[BUTTON_SQUARE].pressed == true || this.keyMap.get(BUTTON_SQUARE) == true)
      this.emulator.update_input(SELECT, gamepad?.buttons[SELECT].pressed == true || this.keyMap.get(SELECT) == true)
      this.emulator.update_input(START, gamepad?.buttons[START].pressed == true || this.keyMap.get(START) == true)
      this.emulator.update_input(UP, gamepad?.buttons[UP].pressed == true || this.keyMap.get(UP) == true)
      this.emulator.update_input(DOWN, gamepad?.buttons[DOWN].pressed == true || this.keyMap.get(DOWN) == true)
      this.emulator.update_input(LEFT, gamepad?.buttons[LEFT].pressed == true || this.keyMap.get(LEFT) == true)
      this.emulator.update_input(RIGHT, gamepad?.buttons[RIGHT].pressed == true || this.keyMap.get(RIGHT) == true)
    }
  }
}