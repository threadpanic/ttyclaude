# tty-web - Web Client

Modern browser-based interface for LLM conversations. Connects to tty-server via HTTP REST + WebSocket.

## Features

- **Real-Time Streaming**
  - WebSocket-based token streaming
  - Instant message updates
  - Low latency

- **Markdown Rendering**
  - react-markdown for content
  - Syntax highlighting (react-syntax-highlighter)
  - Code blocks with language detection

- **Session Management**
  - Create/load/delete sessions
  - Session list sidebar
  - Persistent conversations

- **Modern UI**
  - Tailwind CSS styling
  - Dark mode (default)
  - Responsive design
  - Mobile-friendly

## Installation

```bash
npm install
```

## Development

```bash
# Start dev server (with hot reload)
npm run dev

# Open http://localhost:5173
```

The dev server proxies API requests to tty-server:
- `/api/*` → `http://localhost:7332/api/*`
- `/ws` → `ws://localhost:7332/ws`

## Production Build

```bash
npm run build
npm run preview
```

## Configuration

### Vite Config (`vite.config.ts`)

```typescript
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:7332',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://localhost:7332',
        ws: true,
      },
    },
  },
})
```

### Environment Variables (planned)

```bash
# .env.local
VITE_API_URL=http://localhost:7332
VITE_WS_URL=ws://localhost:7332
```

## Architecture

```
tty-web/
├── src/
│   ├── components/       - React components
│   │   ├── ChatView.tsx  - Main chat interface
│   │   └── SessionList.tsx - Session sidebar
│   ├── hooks/
│   │   └── useWebSocket.ts - WebSocket hook
│   ├── types/
│   │   └── index.ts      - TypeScript types
│   ├── App.tsx           - Main app component
│   └── main.tsx          - Entry point
├── public/               - Static assets
└── index.html            - HTML template
```

## Components

### App.tsx

Main application container. Manages:
- WebSocket connection
- Session state
- Message state
- API calls

### ChatView.tsx

Chat interface component:
- Message display (scrollable)
- Markdown rendering
- Input form
- Send button

### SessionList.tsx

Session management sidebar:
- List all sessions
- Select session to load
- Show session metadata
- Delete sessions (planned)

### useWebSocket.ts

Custom hook for WebSocket:
- Connection management
- Message parsing
- Send helper
- Connection status

## API Integration

### REST API

```typescript
// List sessions
const response = await fetch('/api/v1/sessions')
const data = await response.json()

// Create session
const response = await fetch('/api/v1/sessions', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    provider: 'anthropic',
    model: null
  })
})

// Load messages
const response = await fetch(`/api/v1/sessions/${id}/messages`)
```

### WebSocket

```typescript
const ws = new WebSocket('ws://localhost:7332/ws')

// Authenticate
ws.send(JSON.stringify({
  type: 'authenticate',
  token: 'user_token'
}))

// Send message
ws.send(JSON.stringify({
  type: 'send_message',
  session_id: '...',
  content: 'Hello!'
}))

// Receive tokens
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data)
  if (msg.type === 'token') {
    // Append to current message
  }
}
```

## Styling

Uses Tailwind CSS with custom configuration:

```css
/* Dark theme colors */
bg-gray-900  /* Background */
bg-gray-800  /* Card/sidebar */
bg-gray-700  /* Hover states */
text-gray-100 /* Primary text */
text-gray-400 /* Secondary text */

/* Accent colors */
bg-blue-600   /* Primary actions */
bg-green-600  /* Success/create */
bg-red-600    /* Destructive/error */
```

## Deployment

### Static Hosting (Netlify, Vercel)

```bash
npm run build
# Deploy dist/ folder
```

Add `_redirects` for SPA routing:
```
/*    /index.html   200
```

### Nginx

```nginx
server {
    listen 80;
    server_name chat.example.com;

    root /var/www/tty-web;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api/ {
        proxy_pass http://localhost:7332;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    location /ws {
        proxy_pass http://localhost:7332;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "Upgrade";
        proxy_set_header Host $host;
    }
}
```

### Docker (Planned)

```dockerfile
FROM node:20 as builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
```

## Development Tips

### Hot Reload

Vite provides instant HMR. Changes to `.tsx` files reload immediately.

### Type Checking

```bash
npm run build  # Runs tsc for type checking
```

### Debugging

```typescript
// Enable WebSocket logging
const { isConnected, lastMessage, sendMessage } = useWebSocket(wsUrl)

useEffect(() => {
  console.log('WS Message:', lastMessage)
}, [lastMessage])
```

## Roadmap

- [ ] User authentication (JWT)
- [ ] Session search/filter
- [ ] Export conversation to markdown
- [ ] Conversation branching
- [ ] Multi-session tabs
- [ ] Drag-and-drop file upload
- [ ] Voice input (planned)
- [ ] Offline support (PWA)

## Browser Compatibility

- Chrome/Edge: 90+
- Firefox: 88+
- Safari: 14+
- Mobile: iOS Safari 14+, Chrome Android 90+

Requires:
- WebSocket support
- ES2020 features
- CSS Grid/Flexbox

## Performance

- Initial bundle: ~200KB (gzipped)
- React: 45KB
- Syntax highlighter: 80KB (lazy loaded)
- Total: ~350KB including all dependencies

Optimizations:
- Code splitting (Vite automatic)
- Lazy loading for syntax highlighter
- Virtualized message list (planned for 1000+ messages)

## License

See [LICENSE](../LICENSE)
