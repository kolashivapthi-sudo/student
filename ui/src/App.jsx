import { useState, useRef, useEffect, useCallback } from 'react'
import Sidebar from './components/Sidebar'
import ChatMessage from './components/ChatMessage'
import FloatingInputBar from './components/FloatingInputBar'
import { Sigma, Sparkles } from 'lucide-react'

/* ──────────────────────────────────────────────
   Mock STUDENT solver responses
────────────────────────────────────────────── */
const STUDENT_RESPONSES = {
  default: (problem) => `I analyzed your problem: **"${problem}"**

Here's how I solved it step by step:

**Expressions:**
\`\`\`
x + 3 = 10
x = 7
\`\`\`

**Answer:** \`x = 7\`

---

The equation was extracted, flattened into a single-variable constraint, and solved via constraint propagation.`,

  'john': `I analyzed your problem: **"John has 5 more apples than Mary. Mary has 3 apples."**

**Expressions:**
\`\`\`
mary = 3
john = mary + 5
\`\`\`

Solving step by step:
1. \`mary\` is known → **3**
2. Substitute into second equation: \`john = 3 + 5\`

**Answer:** \`john = 8\`

> John has **8 apples**, Mary has **3 apples**.`,

  'sum': `I analyzed your problem: **"The sum of two numbers is 20. One is 4 times the other."**

**Expressions:**
\`\`\`
_t0 = smaller * 4
larger = _t0
_t1 = smaller + larger
_t1 = 20
\`\`\`

Solving via constraint propagation:
- Let \`smaller = x\` and \`larger = 4x\`
- \`x + 4x = 20\` → \`5x = 20\` → \`x = 4\`

**Answer:**
| Variable | Value |
|----------|-------|
| smaller  | 4     |
| larger   | 16    |

✓ Verified: \`4 + 16 = 20\``,

  'error': `⚠️ **Could not understand the problem.**

\`Error: Not enough information to solve the problem.\`

**Tip:** Make sure your problem has enough constraints. For two unknowns, you need two equations.

**Example that works:**
\`\`\`
John has 3 more apples than Mary. Mary has 5 apples.
\`\`\``,

  'divide': `⚠️ **Division by zero detected.**

\`Error: Division by zero is not possible.\`

The solver caught this before evaluation. Please check your problem — one of the divisors evaluates to zero.`,
}

function pickResponse(input) {
  const lower = input.toLowerCase()
  if (lower.includes('john') || lower.includes('mary')) return STUDENT_RESPONSES.john
  if (lower.includes('sum') || lower.includes('two number')) return STUDENT_RESPONSES.sum
  if (lower.includes('divide by zero') || lower.includes('÷ 0')) return STUDENT_RESPONSES.divide
  if (lower.includes('???') || lower.length < 5) return STUDENT_RESPONSES.error
  return STUDENT_RESPONSES.default(input)
}

/* ──────────────────────────────────────────────
   Welcome screen shown before first message
────────────────────────────────────────────── */
function WelcomeScreen({ onExample }) {
  const examples = [
    {
      label: 'Simple assignment',
      text: 'x is 5 more than 3. Find x.',
    },
    {
      label: 'Two-variable system',
      text: 'John has 3 more apples than Mary. Mary has 5 apples. Find John.',
    },
    {
      label: 'Sum & ratio',
      text: 'The sum of two numbers is 20. One is 4 times the other.',
    },
    {
      label: 'Division',
      text: '25 apples divided by 5 baskets. How many per basket?',
    },
  ]

  return (
    <div className="flex flex-col items-center justify-center h-full px-6 text-center">
      {/* Logo */}
      <div className="w-16 h-16 rounded-2xl bg-indigo-600/20 border border-indigo-500/20
        flex items-center justify-center mb-6">
        <Sigma size={28} className="text-indigo-400" />
      </div>

      <h1 className="text-2xl font-semibold text-zinc-100 mb-2 tracking-tight">
        STUDENT Solver
      </h1>
      <p className="text-zinc-500 text-sm max-w-sm leading-relaxed mb-8">
        Type an algebra word problem in plain English.
        I'll parse it, build equations, and solve step by step.
      </p>

      {/* Example cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 w-full max-w-xl">
        {examples.map((ex, i) => (
          <button
            key={i}
            onClick={() => onExample(ex.text)}
            className="text-left p-4 rounded-xl border border-white/[0.07]
              bg-white/[0.02] hover:bg-white/[0.05] hover:border-white/[0.12]
              group transition-all"
          >
            <div className="flex items-center gap-2 mb-1.5">
              <Sparkles size={11} className="text-indigo-400" />
              <p className="text-[11px] text-indigo-400 font-semibold uppercase tracking-wider">
                {ex.label}
              </p>
            </div>
            <p className="text-xs text-zinc-400 group-hover:text-zinc-300 leading-relaxed">
              {ex.text}
            </p>
          </button>
        ))}
      </div>
    </div>
  )
}

/* ──────────────────────────────────────────────
   Main App
────────────────────────────────────────────── */
let msgIdCounter = 0
const newId = () => ++msgIdCounter

export default function App() {
  const [messages, setMessages] = useState([])
  const [isStreaming, setIsStreaming] = useState(false)
  const [activeChat, setActiveChat] = useState(null)
  const bottomRef = useRef(null)

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const handleSend = useCallback((text) => {
    if (isStreaming) return

    const userMsg = { id: newId(), role: 'user', content: text }
    const aiMsg = { id: newId(), role: 'assistant', content: '', loading: true }

    setMessages(prev => [...prev, userMsg, aiMsg])
    setIsStreaming(true)

    // Simulate streaming response
    const fullResponse = pickResponse(text)
    const words = fullResponse.split('')
    let accumulated = ''
    let charIdx = 0

    const interval = setInterval(() => {
      if (charIdx >= words.length) {
        clearInterval(interval)
        setMessages(prev =>
          prev.map(m =>
            m.id === aiMsg.id ? { ...m, content: fullResponse, loading: false } : m
          )
        )
        setIsStreaming(false)
        return
      }

      // Stream 3 chars at a time for realistic feel
      accumulated += words.slice(charIdx, charIdx + 3).join('')
      charIdx += 3
      const snap = accumulated
      setMessages(prev =>
        prev.map(m =>
          m.id === aiMsg.id ? { ...m, content: snap, loading: false } : m
        )
      )
    }, 18)
  }, [isStreaming])

  const handleStop = () => {
    setIsStreaming(false)
  }

  const handleNew = () => {
    setMessages([])
    setActiveChat(null)
  }

  const handleExample = (text) => {
    handleSend(text)
  }

  const isEmpty = messages.length === 0

  return (
    <div className="flex h-screen overflow-hidden bg-[#0f0f11]">
      {/* Sidebar */}
      <Sidebar
        activeId={activeChat}
        onSelect={id => setActiveChat(id)}
        onNew={handleNew}
      />

      {/* Main area */}
      <div className="flex-1 flex flex-col relative overflow-hidden">
        {/* Top bar */}
        <header className="flex items-center justify-between px-6 py-3.5 border-b border-white/[0.05] shrink-0">
          <div className="flex items-center gap-3">
            <h2 className="text-sm font-medium text-zinc-300">
              {isEmpty ? 'New conversation' : 'Algebra Solver'}
            </h2>
            {!isEmpty && (
              <span className="text-[10px] px-2 py-0.5 rounded-full bg-indigo-500/15 text-indigo-400 border border-indigo-500/20">
                {messages.filter(m => m.role === 'user').length} problem{messages.filter(m => m.role === 'user').length !== 1 ? 's' : ''}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2 text-[11px] text-zinc-600">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 inline-block" />
            Ready
          </div>
        </header>

        {/* Message thread */}
        <div className="flex-1 overflow-y-auto">
          {isEmpty ? (
            <WelcomeScreen onExample={handleExample} />
          ) : (
            <div className="max-w-3xl mx-auto px-6 py-8 pb-48">
              {messages.map(msg => (
                <ChatMessage key={msg.id} message={msg} />
              ))}
              <div ref={bottomRef} />
            </div>
          )}
        </div>

        {/* Floating input bar */}
        <FloatingInputBar
          onSend={handleSend}
          isStreaming={isStreaming}
          onStop={handleStop}
        />
      </div>
    </div>
  )
}
