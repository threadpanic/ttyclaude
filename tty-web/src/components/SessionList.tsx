import type { Session } from '../types'

interface Props {
  sessions: Session[]
  currentSession: string | null
  onSelectSession: (id: string) => void
  onClose: () => void
}

export default function SessionList({ sessions, currentSession, onSelectSession, onClose }: Props) {
  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString()
  }

  return (
    <div className="w-80 bg-gray-800 border-r border-gray-700 flex flex-col">
      <div className="p-4 border-b border-gray-700 flex justify-between items-center">
        <h2 className="text-lg font-bold">Sessions</h2>
        <button
          onClick={onClose}
          className="text-gray-400 hover:text-white"
        >
          ✕
        </button>
      </div>

      <div className="flex-1 overflow-y-auto">
        {sessions.length === 0 && (
          <div className="p-4 text-gray-500 text-center">
            No sessions yet
          </div>
        )}

        {sessions.map((session) => (
          <button
            key={session.id}
            onClick={() => onSelectSession(session.id)}
            className={`w-full text-left p-4 border-b border-gray-700 hover:bg-gray-700 transition ${
              currentSession === session.id ? 'bg-gray-700' : ''
            }`}
          >
            <div className="font-semibold truncate">
              {session.id.substring(0, 8)}...
            </div>
            <div className="text-sm text-gray-400">
              {session.provider} / {session.model}
            </div>
            <div className="text-xs text-gray-500 mt-1">
              {formatDate(session.updated_at)}
            </div>
          </button>
        ))}
      </div>
    </div>
  )
}
