
import { GBC } from "./src/gbc"
import 'bulma/css/bulma'
import './styles/app'

const gbc = new GBC()

gbc.addEventListeners()
gbc.initWasm()