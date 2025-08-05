import { WebEmulator } from "../../../pkg/gb_plus_web"
import { GBC } from "../gbc"
import { StateManager } from "../saves/state_manager"

enum GamepadButtons {
  Cross = 0,
  Circle = 1,
  Square = 2,
  Triangle = 3,
  L1 = 4,
  R1 = 5,
  L2 = 6,
  R2 = 7,
  Select = 8,
  Start = 9,
  LeftStick = 10,
  RightStick = 11,
  Up = 12,
  Down = 13,
  Left = 14,
  Right = 15
}

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

const JOYPAD_ELEMENTS = new Map([
  ["a", document.getElementById("joy-a")],
  ["b", document.getElementById("joy-b")],
  ["select", document.getElementById("joy-select")],
  ["start", document.getElementById("joy-start")]
])

const BUTTON_IDS_TO_STRINGS = new Map([
  [GamepadButtons.Cross, "cross"],
  [GamepadButtons.Square, "square"],
  [GamepadButtons.Select, "select"],
  [GamepadButtons.Start, "start"],
  [GamepadButtons.Circle, "circle"],
  [GamepadButtons.Triangle, "triangle"],
  [GamepadButtons.L1, "l1"],
  [GamepadButtons.R1, "r1"],
  [GamepadButtons.L2, "l2"],
  [GamepadButtons.R2, "r2"],
  [GamepadButtons.LeftStick, "left stick"],
  [GamepadButtons.RightStick, "right stick"]
])

const BUTTON_STRINGS_TO_IDS = new Map([
  ["cross", GamepadButtons.Cross],
  ["square", GamepadButtons.Square],
  ["select", GamepadButtons.Select],
  ["start", GamepadButtons.Start],
  ["circle", GamepadButtons.Circle],
  ["triangle", GamepadButtons.Triangle],
  ["l1", GamepadButtons.L1],
  ["r1", GamepadButtons.R1],
  ["l2", GamepadButtons.L2],
  ["r2", GamepadButtons.R2],
  ["left stick", GamepadButtons.LeftStick],
  ["right stick", GamepadButtons.RightStick]
])

export class Joypad {
  pressedKeys = new Map<number, number>()
  gbc: GBC
  keyMap = new Map<string, boolean>()
  stateManager: StateManager|null = null

  private keyMappings = new Map<string, string>([
    ["k", "a"],
    ["j", "b"],
    ["s", "down"],
    ["a", "left"],
    ["d", "right"],
    ["w", "up"],
    ["Tab", "select"],
    ["Enter", "start"]
  ])

  private buttonToKeys = new Map<string, string>([
    ["up", "w"],
    ["down", "s"],
    ["left", "a"],
    ["right", "d"],
    ["select", "tab"],
    ["enter", "start"],
    ["b", "j"],
    ["a", "k"]
  ])

  private buttonToJoypad = new Map([
    ["a", GamepadButtons.Cross],
    ["b", GamepadButtons.Square],
    ["select", GamepadButtons.Select],
    ["start", GamepadButtons.Start]
  ])

  private joypadMappings = new Map([
    [GamepadButtons.Cross, "a"],
    [GamepadButtons.Square, "b"],
    [GamepadButtons.Select, "select"],
    [GamepadButtons.Start, "start"]
  ])

  private currentKeyInput: HTMLInputElement|null = null
  private currentJoyInput: HTMLInputElement|null = null

  private currentFrame = 0

  private changingJoypadMapping = false

  setStateManager(stateManager: StateManager) {
    this.stateManager = stateManager
  }

  initKeyboardMappings() {
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

      const gbButton = this.getGbButton(keyInput as HTMLInputElement)

      if (gbButton != null) {
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
    }

    document.addEventListener('keydown', async (e) => {
      if (this.currentKeyInput != null) {

        if (e.key == "Escape") {
          this.cancelMappings()
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

        const gbButton = this.getGbButton(this.currentKeyInput)

        if (gbButton == null) {
          return
        }

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

        const checkmark = this.currentKeyInput.nextElementSibling as HTMLElement

        checkmark.style.display = "inline"

        this.focusNextInput(this.currentKeyInput, (child) => {
          child.focus()
          this.currentKeyInput = child
        })
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

  constructor(gbc: GBC) {
    this.gbc = gbc

    this.initKeyboardMappings()
    this.initControllerMappings()
  }

  initControllerMappings() {
    const joyMappings = localStorage.getItem("gbc-joy-mappings")

    if (joyMappings != null) {
      this.joypadMappings = new Map(JSON.parse(joyMappings))
    }

    const buttonToJoypad = localStorage.getItem("gbc-button-to-joypad")

    if (buttonToJoypad != null) {
      this.buttonToJoypad = new Map(JSON.parse(buttonToJoypad))
    }

    const joyInputs = document.getElementsByClassName('joy-input')

    for (const joyInput of joyInputs) {
      const joyEl = joyInput as HTMLInputElement

      const gbButton = this.getGbButton(joyEl)

      if (gbButton == null) {
        return
      }

      const buttonStr = BUTTON_IDS_TO_STRINGS.get(this.buttonToJoypad.get(gbButton)!)

      if (buttonStr != null) {
        joyEl.value = buttonStr
      }

      joyInput.addEventListener("focus", () => {
        this.currentJoyInput = joyInput as HTMLInputElement

        this.currentJoyInput.className += " is-warning"
        this.currentJoyInput.placeholder = "Press button...."

        this.currentFrame = requestAnimationFrame((time) => this.pollInput())
      })
    }
  }

  getGbButton(keyInput: HTMLInputElement) {
    const sibling =
      keyInput.parentElement?.parentElement?.previousElementSibling as HTMLElement|undefined

    if (sibling != null) {
      return sibling.innerText.toLowerCase()
    }

    return null
  }

  pollInput() {
    const gamepad = navigator.getGamepads()[0]

    if (gamepad != null && this.currentJoyInput != null) {
      for (let buttonIdx = 0; buttonIdx < gamepad.buttons.length; buttonIdx++) {
        const button = gamepad.buttons[buttonIdx]

        if (button.pressed) {
          const oldJoyButton = this.currentJoyInput.value

          const oldButtonId = BUTTON_STRINGS_TO_IDS.get(oldJoyButton)

          if (oldButtonId != null) {
            this.joypadMappings.delete(oldButtonId)

            const existingButton = this.joypadMappings.get(buttonIdx)

            if (existingButton != null) {
              const oldMapping = this.buttonToJoypad.get(existingButton)!

              this.joypadMappings.delete(oldMapping)

              this.joypadMappings.set(oldButtonId, existingButton)
              this.buttonToJoypad.set(existingButton, oldButtonId)

              const element = JOYPAD_ELEMENTS.get(existingButton) as HTMLInputElement

              if (element != null) {
                element.value = oldJoyButton
              }
            }
          }

          const buttonStr = BUTTON_IDS_TO_STRINGS.get(buttonIdx)

          if (buttonStr != null) {
            const gbButton = this.getGbButton(this.currentJoyInput)

            if (gbButton == null) {
              return
            }

            this.joypadMappings.set(buttonIdx, gbButton)
            this.buttonToJoypad.set(gbButton, buttonIdx)

            this.currentJoyInput.value = buttonStr
            this.currentJoyInput.className = "input is-success joy-input"

            const checkmark = this.currentJoyInput.nextElementSibling as HTMLElement

            checkmark.style.display = "inline"

            localStorage.setItem("gbc-joy-mappings", JSON.stringify(Array.from(this.joypadMappings.entries())))
            localStorage.setItem("gbc-button-to-joypad", JSON.stringify(Array.from(this.buttonToJoypad.entries())))

            this.focusNextInput(this.currentJoyInput, (child) => {
                setTimeout(() => {
                  child.focus()
                  this.currentJoyInput = child
                }, 200)
            })

            cancelAnimationFrame(this.currentFrame)

            return
          }
        }
      }
    }
    this.currentFrame = requestAnimationFrame((time) => this.pollInput())
  }

  focusNextInput(input: HTMLInputElement|null, callback: (child: HTMLInputElement) => void) {
    const nextDiv =
      input?.parentElement?.parentElement?.parentElement?.nextElementSibling

    if (nextDiv != null) {
      const child = nextDiv.children[1]?.children[0]?.children[0] as HTMLInputElement

      if (child != null) {
        callback(child)
      } else {
        input = null
      }
    } else {
      input = null
    }
  }

  cancelMappings() {
    this.revertInputs('key-input')
    this.revertInputs('joy-input')

    const modal = document.getElementById("controller-mappings-modal")

    if (modal != null) {
      modal.style.display = "none"
      modal.className = "modal hide"
    }

    this.gbc.emulator!.set_pause(false)

    cancelAnimationFrame(this.currentFrame)
  }

  revertInputs(className: string) {
    const inputs = document.getElementsByClassName(className)

    for (const input of inputs) {
      const inputEl = input as HTMLInputElement
      inputEl.className = `input is-link ${className}`
    }
  }

  async handleInput() {
    const gamepad = navigator.getGamepads()[0]

    if (this.gbc.emulator != null) {
      const isGoingLeft = (gamepad?.axes[0] ?? 0) <= -0.5
      const isGoingRight = (gamepad?.axes[0] ?? 0) >= 0.5

      const isGoingUp = (gamepad?.axes[1] ?? 0) <= -0.5
      const isGoingDown = (gamepad?.axes[1] ?? 0) >= 0.5

      this.gbc.emulator.update_input("a", gamepad?.buttons[this.buttonToJoypad.get("a") ?? GamepadButtons.Cross].pressed == true || this.keyMap.get("a") == true)
      this.gbc.emulator.update_input("b", gamepad?.buttons[this.buttonToJoypad.get("b") ?? GamepadButtons.Square].pressed == true || this.keyMap.get("b") == true)
      this.gbc.emulator.update_input("select", gamepad?.buttons[this.buttonToJoypad.get("select") ?? GamepadButtons.Select].pressed == true || this.keyMap.get("select") == true)
      this.gbc.emulator.update_input("start", gamepad?.buttons[this.buttonToJoypad.get("start") ?? GamepadButtons.Start].pressed == true || this.keyMap.get("start") == true)
      this.gbc.emulator.update_input("up", gamepad?.buttons[GamepadButtons.Up].pressed == true || isGoingUp || this.keyMap.get("up") == true)
      this.gbc.emulator.update_input("down", gamepad?.buttons[GamepadButtons.Down].pressed == true || isGoingDown || this.keyMap.get("down") == true)
      this.gbc.emulator.update_input("left", gamepad?.buttons[GamepadButtons.Left].pressed == true || isGoingLeft || this.keyMap.get("left") == true)
      this.gbc.emulator.update_input("right", gamepad?.buttons[GamepadButtons.Right].pressed == true || isGoingRight || this.keyMap.get("right") == true)

      if (gamepad?.buttons[GamepadButtons.LeftStick].pressed) {
        this.gbc.createSaveState(true)
      }

      if (gamepad?.buttons[GamepadButtons.RightStick].pressed) {
        const compressed = await this.gbc.db.loadSaveState(this.gbc.gameName)

        if (compressed != null) {
          this.gbc.loadSaveState(compressed)
        }
      }
    }
  }
}