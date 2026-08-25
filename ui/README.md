# STUDENT — High-School Algebra Word Problem Solver

A React-based dark-themed chat UI for the STUDENT algebra solver project.

## Features

- **Dark, polished UI** — Deep slate background (#0f0f11) with subtle borders and indigo accents
- **Collapsible sidebar** — Chat history grouped by Today / Yesterday / Previous 7 Days
- **Markdown rendering** — Code blocks with syntax highlighting, tables, blockquotes, lists
- **Floating input bar** — Auto-expanding textarea with backdrop blur
- **Real-time streaming** — Character-by-character AI responses with loading states
- **Responsive design** — Max-width constraints for comfortable reading
- **Mock solver** — Pre-built responses demonstrating STUDENT's constraint propagation

## Tech Stack

- **React 19** + Vite
- **Tailwind CSS v4** (via @tailwindcss/vite plugin)
- **Lucide React** icons
- **Zero markdown dependencies** — Custom lightweight parser

## Getting Started

```bash
cd ui
npm install
npm run dev
```

The app will open at http://localhost:5173/

## Project Structure

```
ui/
├── src/
│   ├── components/
│   │   ├── Sidebar.jsx           # Collapsible sidebar with history
│   │   ├── ChatMessage.jsx       # User/AI message bubbles + markdown renderer
│   │   └── FloatingInputBar.jsx  # Auto-resize textarea with send/stop
│   ├── App.jsx                   # Main layout and state orchestration
│   ├── main.jsx                  # React entry point
│   └── index.css                 # Tailwind + custom styles
├── index.html
├── vite.config.js
└── package.json
```

## Design Specs

- Background: `#0f0f11` (deep zinc)
- Primary text: `text-zinc-100`
- Secondary text: `text-zinc-400`
- Borders: `border-white/10`
- Accents: Indigo/violet (`bg-indigo-600`, `text-indigo-400`)
- User messages: Right-aligned, `bg-zinc-800` bubbles
- AI messages: Left-aligned with avatar, no heavy background, markdown prose
- Code blocks: `#1a1a1f` background with copy button
- Floating input: `backdrop-blur-md bg-zinc-900/80` with `rounded-2xl`

## Mock Solver Behavior

The UI includes pre-programmed responses for demo:
- **"John has 5 more apples than Mary"** → Constraint solving with step-by-step
- **"Sum of two numbers is 20"** → Table output
- **Insufficient info** → Error message with tip
- **Division by zero** → Graceful error

To connect to the real Rust backend, replace `pickResponse()` in `App.jsx` with an HTTP call to the solver API.

## License

This UI matches the parent STUDENT project (Rust reimplementation of the 1980s AI system).
