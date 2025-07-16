export class GBC {

  addEventListeners() {
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