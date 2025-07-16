import JSZip from 'jszip'

export class GBC {


  onGameChange(file: File) {
    console.log(file)
  }

  addEventListeners() {
    const loadGame = document.getElementById("game-button")
    const gameInput = document.getElementById("game-input")

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
        case "Escape":
          const savesModal = document.getElementById("saves-modal")

          if (savesModal != null) {
            savesModal.className = "modal hide"
            savesModal.style.display = "none"
          }

          const helpModal = document.getElementById("help-modal")

          if (helpModal != null) {
            helpModal.className = "modal hide"
            helpModal.style.display = "none"
          }
          break
      }
    }
  }
}