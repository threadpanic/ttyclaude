export interface Message {
  id: string
  role: 'user' | 'assistant'
  content: string
  created_at: number
}

export interface Session {
  id: string
  provider: string
  model: string
  created_at: number
  updated_at: number
}

export interface WsMessage {
  type: string
  [key: string]: any
}
