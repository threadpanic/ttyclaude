import { useState, useEffect, useCallback, useRef } from 'react'
import type { WsMessage } from '../types'

export function useWebSocket(url: string) {
  const [isConnected, setIsConnected] = useState(false)
  const [lastMessage, setLastMessage] = useState<WsMessage | null>(null)
  const wsRef = useRef<WebSocket | null>(null)

  useEffect(() => {
    const ws = new WebSocket(url)

    ws.onopen = () => {
      console.log('WebSocket connected')
      setIsConnected(true)

      // Authenticate
      ws.send(JSON.stringify({
        type: 'authenticate',
        token: 'default_user'
      }))
    }

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as WsMessage
        setLastMessage(message)
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e)
      }
    }

    ws.onclose = () => {
      console.log('WebSocket disconnected')
      setIsConnected(false)
    }

    ws.onerror = (error) => {
      console.error('WebSocket error:', error)
    }

    wsRef.current = ws

    return () => {
      ws.close()
    }
  }, [url])

  const sendMessage = useCallback((message: WsMessage) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message))
    }
  }, [])

  return { isConnected, lastMessage, sendMessage }
}
