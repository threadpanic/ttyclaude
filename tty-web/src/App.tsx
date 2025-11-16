import { useState, useEffect } from 'react'
import { useWebSocket } from './hooks/useWebSocket'
import ChatView from './components/ChatView'
import SessionList from './components/SessionList'
import type { Message, Session } from './types'

function App() {
  const [sessions, setSessions] = useState<Session[]>([])
  const [currentSession, setCurrentSession] = useState<string | null>(null)
  const [messages, setMessages] = useState<Message[]>([])
  const [showSessions, setShowSessions] = useState(false)

  const wsUrl = import.meta.env.DEV
    ? 'ws://localhost:7332/ws'
    : `ws://${window.location.host}/ws`

  const { isConnected, lastMessage, sendMessage } = useWebSocket(wsUrl)

  // Handle WebSocket messages
  useEffect(() => {
    if (!lastMessage) return

    switch (lastMessage.type) {
      case 'authenticated':
        console.log('Authenticated as:', lastMessage.user_id)
        loadSessions()
        break

      case 'session_created':
        setCurrentSession(lastMessage.session_id)
        setMessages([])
        loadSessions()
        break

      case 'token':
        // Append token to last assistant message
        setMessages(prev => {
          const newMessages = [...prev]
          const lastMsg = newMessages[newMessages.length - 1]
          if (lastMsg && lastMsg.role === 'assistant') {
            lastMsg.content += lastMessage.content
          }
          return newMessages
        })
        break

      case 'message_complete':
        console.log('Message complete')
        break

      case 'error':
        console.error('Error:', lastMessage.message)
        break
    }
  }, [lastMessage])

  const loadSessions = async () => {
    try {
      const response = await fetch('/api/v1/sessions')
      const data = await response.json()
      setSessions(data.sessions || [])
    } catch (e) {
      console.error('Failed to load sessions:', e)
    }
  }

  const createSession = async () => {
    try {
      const response = await fetch('/api/v1/sessions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          provider: 'anthropic',
          model: null
        })
      })
      const data = await response.json()
      setCurrentSession(data.session_id)
      setMessages([])
      loadSessions()
    } catch (e) {
      console.error('Failed to create session:', e)
    }
  }

  const loadSession = async (sessionId: string) => {
    try {
      const response = await fetch(`/api/v1/sessions/${sessionId}/messages`)
      const data = await response.json()
      setMessages(data.messages || [])
      setCurrentSession(sessionId)
      setShowSessions(false)
    } catch (e) {
      console.error('Failed to load session:', e)
    }
  }

  const sendUserMessage = (content: string) => {
    if (!currentSession) {
      alert('Please create a session first')
      return
    }

    // Add user message to UI
    const userMsg: Message = {
      id: Date.now().toString(),
      role: 'user',
      content,
      created_at: Date.now()
    }

    const assistantMsg: Message = {
      id: (Date.now() + 1).toString(),
      role: 'assistant',
      content: '',
      created_at: Date.now()
    }

    setMessages(prev => [...prev, userMsg, assistantMsg])

    // Send via WebSocket
    sendMessage({
      type: 'send_message',
      session_id: currentSession,
      content
    })
  }

  return (
    <div className="h-screen flex flex-col bg-gray-900 text-gray-100">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 p-4 flex justify-between items-center">
        <h1 className="text-xl font-bold">tty-web</h1>
        <div className="flex gap-2">
          <button
            onClick={() => setShowSessions(!showSessions)}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded"
          >
            Sessions
          </button>
          <button
            onClick={createSession}
            className="px-4 py-2 bg-green-600 hover:bg-green-700 rounded"
          >
            New Session
          </button>
          <div className={`px-3 py-2 rounded ${isConnected ? 'bg-green-600' : 'bg-red-600'}`}>
            {isConnected ? '● Connected' : '○ Disconnected'}
          </div>
        </div>
      </header>

      {/* Main content */}
      <div className="flex-1 flex overflow-hidden">
        {showSessions && (
          <SessionList
            sessions={sessions}
            currentSession={currentSession}
            onSelectSession={loadSession}
            onClose={() => setShowSessions(false)}
          />
        )}

        <ChatView
          messages={messages}
          onSendMessage={sendUserMessage}
          sessionId={currentSession}
        />
      </div>
    </div>
  )
}

export default App
