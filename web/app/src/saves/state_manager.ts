
import { InitOutput, WebEmulator } from '../../../pkg/gb_plus_web'
import { GbcDatabase } from './gbc_database'
import { zlib, unzlib  } from 'fflate'
import { StateEntry } from '../interface//game_state_entry'

export class StateManager {
  emulator: WebEmulator
  wasm: InitOutput|null = null
  db: GbcDatabase
  gameName: string

  constructor(
    emulator: WebEmulator,
    wasm: InitOutput|null,
    gameName: string,
    db: GbcDatabase
  ) {
    this.emulator = emulator
    this.wasm = wasm
    this.gameName = gameName
    this.db = db
  }

  async createSaveState(imageUrl: string, stateName = "quick_save.state", isUpdate: boolean = false): Promise<StateEntry|null> {
    if (this.wasm != null) {
      const data = new Uint8Array(this.wasm.memory.buffer, this.emulator.create_save_state(), this.emulator.save_state_length())

      return new Promise((resolve, reject) => {
        zlib(data, { level: 2 }, async (err, compressed) => {
          if (err) {
            console.log(err)
            resolve(null)
          } else {
            const entry = await this.db.createSaveState(this.gameName, compressed, imageUrl, stateName, isUpdate)
            resolve(entry)
          }
        })
      })
    }

    return null
  }


  async decompress(compressed: Uint8Array): Promise<Uint8Array|null> {
    return new Promise((resolve, reject) => {
      unzlib(compressed, (err, data) => {
        if (err) {
          console.log(err)
          resolve(null)
        } else  {
          resolve(data)
        }
      })
    })
  }

  async getSaveStateData(stateName: string = "quick_save.state") {
    return await this.db.loadSaveState(this.gameName, stateName)
  }

  async loadSaveState(stateName = "quick_save.state"): Promise<Uint8Array|null> {
    const compressed = await this.db.loadSaveState(this.gameName, stateName)

    return new Promise(async (resolve, reject) => {
      if (compressed != null) {
        const data = await this.decompress(compressed)
        resolve(data)
      } else {
        resolve(null)
      }
    })
  }
}