import { reactive } from "../util/reactive"

export interface FileEntry {
  filename: string,
  data?: Uint8Array|RtcJson
}

export interface RtcJson {
  timestamp: number
  carry_bit: boolean
  halted: boolean
  num_wraps: number
}

const BASE_URL = "https://accounts.google.com/o/oauth2/v2/auth"
const CLIENT_ID = "353451169812-e707dk5s0qkjq400mrcpjndn6e1smpiv.apps.googleusercontent.com"

export class CloudService {
  private accessToken: string = ""
  private gbcFolderId: string|null = null

  loggedIn = reactive(false)

  constructor() {
    const queryParams = new URL(document.location.toString()).searchParams

    if (queryParams.has("oauth")) {
      const params = this.getLoginParams()

      location.href = `${BASE_URL}?${params.toString()}`
    }

    this.loggedIn.subscribe(() => {
      if (this.loggedIn.value) {
        const signIn = document.getElementById("cloud-button")

        if (signIn != null) {
          signIn.style.display = "none"
          const signOut = document.getElementById("cloud-logged-in")

          if (signOut != null) {
            signOut.style.display = "block"
            signOut.addEventListener("click", () => this.logout())
          }
        }
      } else {
        const signIn = document.getElementById("cloud-button")

        if (signIn != null) {
          signIn.style.display = "block"

          signIn.addEventListener("click", () => this.oauthSignIn())

          const signOut = document.getElementById("cloud-logged-in")

          if (signOut != null) {
            signOut.style.display = "none"
          }
        }
      }
    })

    window.addEventListener("message", (e) => {
      if (e.data == "authFinished") {
        this.getTokenFromStorage()
      }
    })

    const accessToken = localStorage.getItem("gbc_access_token")
    const expiresIn = parseInt(localStorage.getItem("gbc_access_expires") || "null")
    const gbcFolderId = localStorage.getItem("gbc_folder_id")

    if (gbcFolderId != null) {
      this.gbcFolderId = gbcFolderId
    }

    if (accessToken == null) {
      this.loggedIn.value = false

      if (localStorage.getItem("gbc_user_email") != null) {
        this.silentSignIn()
      }
    } else if (expiresIn != null && (Date.now() < expiresIn)) {
      this.accessToken = accessToken

      this.loggedIn.value = true
    } else {
      localStorage.removeItem("gbc_access_token")
      localStorage.removeItem("gbc_access_expires")
      localStorage.removeItem("gbc_folder_id")

      this.silentSignIn()
    }

  }

  async createGbcSavesFolder() {
    if (this.gbcFolderId == null) {
      const params = new URLSearchParams({
        q: `mimeType = "application/vnd.google-apps.folder" and name="gbc-saves"`
      })
      const url = `https://www.googleapis.com/drive/v3/files?${params.toString()}`

      const json = await this.cloudRequest(() => fetch(url, {
        headers: {
          Authorization: `Bearer ${this.accessToken}`
        },
      }))

      if (json != null && json.files != null && json.files[0] != null) {
        this.gbcFolderId = json.files[0].id
        localStorage.setItem("gbc_folder_id", this.gbcFolderId!!)
      } else {
        // create the folder
        const url = `https://www.googleapis.com/drive/v3/files?uploadType=media`

        const json = await this.cloudRequest(() => fetch(url, {
          method: "POST",
          headers: {
            Authorization: `Bearer ${this.accessToken}`,
            "Content-Type": "application/vnd.google-apps.folder"
          },
          body: JSON.stringify({
            name: "gbc-saves",
            mimeType: "application/vnd.google-apps.folder"
          })
        }))


        if (json != null && json.files != null && json.files[0] != null) {
          this.gbcFolderId = json.files[0].id
        }
      }
    }
  }

  getTokenFromStorage() {
    const accessToken = localStorage.getItem("gbc_access_token")

    if (accessToken != null) {
      this.accessToken = accessToken

      this.loggedIn.value = true
    }
  }

  async oauthSignIn() {
    window.open(`${location.href}?oauth=true`, "popup", "popup=true,width=650,height=650,resizable=true")
  }

  logout() {
    localStorage.removeItem("gbc_access_token")
    localStorage.removeItem("gbc_access_expires")
    localStorage.removeItem("gbc_user_email")
    localStorage.removeItem("gbc_folder_id")

    this.loggedIn.value = false
    this.accessToken = ""
  }

  silentSignIn() {
    const silentEl = document.getElementById("silent-sign-in") as HTMLIFrameElement

    if (silentEl != null && silentEl.contentWindow != null) {
      const params = this.getLoginParams(true)

      silentEl.contentWindow.window.location.href = `${BASE_URL}?${params.toString()}`
    }
  }

  private refreshTokensIfNeeded() {

    const userEmail = localStorage.getItem("gbc_user_email")

    if (userEmail == null) {
      return
    }

    return new Promise((resolve, reject) => {
      const gbcExpires = parseInt(localStorage.getItem("gbc_access_expires") || "-1")
      if (gbcExpires != null && (Date.now() >= gbcExpires || gbcExpires == -1)) {
        // refresh tokens as they're expired
        window.addEventListener("message", async (e) => {
          if (e.data == "authFinished") {
            this.getTokenFromStorage()

            resolve(null)
          }
        })

        this.silentSignIn()
      } else {
        resolve(null)
      }
    })
  }

  async cloudRequest(request: () => Promise<Response>, returnBuffer: boolean = false): Promise<any> {
    return new Promise(async (resolve, reject) => {
      await this.refreshTokensIfNeeded()

      const response = await request()

      if (response.status == 200) {
        const data = returnBuffer ? await response.arrayBuffer() : await response.json()

        resolve(data)
      } else if (response.status == 401) {

        this.logout()

        const notification = document.getElementById("request-failure-notification")!

        notification.style.display = "block"

        let opacity = 1.0

        let interval = setInterval(() => {
          opacity -= 0.05
          notification.style.opacity = `${opacity}`

          if (opacity <= 0) {
            clearInterval(interval)
          }
        }, 100)

        resolve(null)
      }
    })
  }

  getLoginParams(noPrompt: boolean = false) {
    // since it always redirects back to the root, location.href should be fine (hopefully!)
    const params = new URLSearchParams({
      client_id: CLIENT_ID,
      redirect_uri: location.href.split('?')[0].replace(/\/$/, ''), // remove the trailing slash
      response_type: "token",
      scope: "https://www.googleapis.com/auth/drive.file https://www.googleapis.com/auth/userinfo.email",
    })

    if (noPrompt) {
      const email = localStorage.getItem("gbc_user_email")

      if (email != null) {
        params.append("prompt", "none")
        params.append("login_hint", email)
      }

    }

    return params
  }

  async getFileInfo(name: string, searchRoot: boolean = false) {
    await this.createGbcSavesFolder()

    const fileName = name.match(/\.sav$/) || name.match(/\.rtc$/) ? name : `${name}.sav`

    const query = searchRoot ? `name = "${fileName}"` : `name = "${fileName}" and parents in "${this.gbcFolderId}"`

    const params = new URLSearchParams({
      q: query,
      fields: "files/id,files/parents,files/name"
    })

    const url = `https://www.googleapis.com/drive/v3/files?${params.toString()}`

    return await this.cloudRequest(() => fetch(url, {
      headers: {
        Authorization: `Bearer ${this.accessToken}`
      }
    }))
  }

  async getFile(filename: string, fetchBytes = true): Promise<FileEntry> {
    const json = await this.getFileInfo(filename)

    if (json != null && json.files != null) {
      const file = json.files[0]

      if (file != null) {

        // retrieve the file data from the cloud
        const url = `https://www.googleapis.com/drive/v3/files/${file.id}?alt=media`

        const body = await this.cloudRequest(() => fetch(url, {
          headers: {
            Authorization: `Bearer ${this.accessToken}`
          }
        }), fetchBytes)

        const returnVal = fetchBytes ? {
          filename,
          data: new Uint8Array((body as ArrayBuffer))
        } : {
          filename,
          data: body as RtcJson
        }

        return returnVal
      }

    }

    return {
      filename,
      data: undefined
    }
  }

  async deleteSave(gameName: string): Promise<boolean> {
    const json = await this.getFileInfo(gameName)

    if (json != null && json.files != null) {
      const url = `https://www.googleapis.com/drive/v3/files/${json.files[0].id}`

      await this.cloudRequest(() => fetch(url, {
        headers: {
          Authorization: `Bearer ${this.accessToken}`
        },
        method: "DELETE"
      }))

      return true
    }

    return false
  }

  async getSaves(): Promise<FileEntry[]> {
    await this.createGbcSavesFolder()

    const params = new URLSearchParams({
      q: `parents in "${this.gbcFolderId}"`
    })
    const url = `https://www.googleapis.com/drive/v3/files?${params.toString()}`

    const json = await this.cloudRequest(() => fetch(url, {
      headers: {
        Authorization: `Bearer ${this.accessToken}`
      }
    }))

    const saveEntries: FileEntry[] = []
    if (json != null && json.files != null) {
      for (const file of json.files) {
        saveEntries.push({
          filename: file.name
        })
      }
    }

    return saveEntries
  }

  async uploadFile(filename: string, bytes: Uint8Array|null, jsonStr: string|null = null) {
    const json = await this.getFileInfo(filename)

    // this is a hack to get it to change the underlying array buffer
    // (so it doesn't save a bunch of junk from memory unrelated to save)

    const payload: Uint8Array|string = bytes == null ? jsonStr! : new Uint8Array(Array.from(bytes))

    const buffer = bytes == null ? payload : (payload as Uint8Array).buffer

    let resultFile: any
    if (json != null && json.files != null) {
      const file = json.files[0]

      if (file != null) {
        const url = `https://www.googleapis.com/upload/drive/v3/files/${file.id}?uploadType=media`
        await this.cloudRequest(() => fetch(url, {
          method: "PATCH",
          headers: {
            Authorization: `Bearer ${this.accessToken}`,
            "Content-Type": "application/octet-stream",
            "Content-Length": `${payload.length}`
          },
          body: buffer
        }))
        // there's no need for renaming the file since it's already been uploaded
        return
      } else {
        const url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=media&fields=id,name,parents"
        resultFile = await this.cloudRequest(() => fetch(url, {
          method: "POST",
          headers: {
            Authorization: `Bearer ${this.accessToken}`,
            "Content-Type": "application/octet-stream",
            "Content-Length": `${payload.length}`
          },
          body: buffer
        }))
      }
    }

    if (resultFile != null) {
      let fileName = !filename.match(/\.sav$/) && !filename.match(/\.rtc$/) ? `${filename}.sav` : filename

      const params = new URLSearchParams({
        uploadType: "media",
        addParents: this.gbcFolderId || "",
        removeParents: resultFile.parents.join(",")
      })

      const url = `https://www.googleapis.com/drive/v3/files/${resultFile.id}?${params.toString()}`

      await this.cloudRequest(() => fetch(url, {
        method: "PATCH",
        headers: {
          Authorization: `Bearer ${this.accessToken}`,
          "Content-Type": "application/octet-stream"
        },
        body: JSON.stringify({
          name: fileName,
          mimeType: "application/octet-stream"
        })
      }))
    }
  }

  async getLoggedInEmail() {
    const url = "https://www.googleapis.com/oauth2/v2/userinfo"

    const json = await this.cloudRequest(() => fetch(url, {
      headers: {
        Authorization: `Bearer ${this.accessToken}`
      }
    }))

    if (json != null && json.email != null) {
      localStorage.setItem("gbc_user_email", json.email)
    }
  }

  async checkAuthentication() {
    if (window.location.href.indexOf("#") != -1) {
      const tokenParams = window.location.href.split("#")[1].split("&")

      let accessToken = tokenParams.filter((param) => param.indexOf('access_token') != -1)[0]
      let expires = tokenParams.filter((param) => param.indexOf('expires_in') != -1)[0]

      if (accessToken != null) {
        accessToken = accessToken.split("=")[1]

        if (expires != null) {
          expires = expires.split("=")[1]

          const timestamp = parseInt(expires) * 1000 + Date.now()

          localStorage.setItem("gbc_access_expires", timestamp.toString())
        }

        localStorage.setItem("gbc_access_token", accessToken)

        this.accessToken = accessToken
        this.loggedIn.value = true

        // finally get logged in user email
        await this.getLoggedInEmail()

        parent.postMessage("authFinished", "*")

        window.opener?.postMessage("authFinished", "*")

        window.close()
      }
    }

  }
}