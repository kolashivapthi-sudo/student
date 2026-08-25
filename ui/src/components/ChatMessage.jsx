import { Sigma, Copy, Check, User } from 'lucide-react'
import { useState } from 'react'

/* ──────────────────────────────────────────────
   Ultra-light markdown renderer (no deps)
   Handles: code blocks, inline code, bold, italic,
   headers, lists, blockquotes, tables, hr
────────────────────────────────────────────── */
function renderMarkdown(text) {
  const lines = text.split('\n')
  const elements = []
  let i = 0
  let key = 0

  while (i < lines.length) {
    const line = lines[i]

    // Fenced code block
    if (line.startsWith('```')) {
      const lang = line.slice(3).trim()
      const codeLines = []
      i++
      while (i < lines.length && !lines[i].startsWith('```')) {
        codeLines.push(lines[i])
        i++
      }
      elements.push(
        <CodeBlock key={key++} code={codeLines.join('\n')} lang={lang} />
      )
      i++
      continue
    }

    // Heading
    const hMatch = line.match(/^(#{1,3})\s+(.+)/)
    if (hMatch) {
      const level = hMatch[1].length
      const Tag = `h${level}`
      elements.push(
        <Tag key={key++} className={`ai-heading-${level}`}>
          {parseInline(hMatch[2])}
        </Tag>
      )
      i++
      continue
    }

    // Horizontal rule
    if (/^---+$/.test(line.trim())) {
      elements.push(<hr key={key++} />)
      i++
      continue
    }

    // Blockquote
    if (line.startsWith('> ')) {
      const bqLines = []
      while (i < lines.length && lines[i].startsWith('> ')) {
        bqLines.push(lines[i].slice(2))
        i++
      }
      elements.push(
        <blockquote key={key++}>{parseInline(bqLines.join(' '))}</blockquote>
      )
      continue
    }

    // Unordered list
    if (/^[-*+]\s/.test(line)) {
      const items = []
      while (i < lines.length && /^[-*+]\s/.test(lines[i])) {
        items.push(<li key={i}>{parseInline(lines[i].slice(2))}</li>)
        i++
      }
      elements.push(<ul key={key++}>{items}</ul>)
      continue
    }

    // Ordered list
    if (/^\d+\.\s/.test(line)) {
      const items = []
      while (i < lines.length && /^\d+\.\s/.test(lines[i])) {
        items.push(
          <li key={i}>{parseInline(lines[i].replace(/^\d+\.\s/, ''))}</li>
        )
        i++
      }
      elements.push(<ol key={key++}>{items}</ol>)
      continue
    }

    // Table
    if (line.includes('|') && lines[i + 1]?.includes('---')) {
      const tableLines = []
      while (i < lines.length && lines[i].includes('|')) {
        tableLines.push(lines[i])
        i++
      }
      elements.push(<MarkdownTable key={key++} lines={tableLines} />)
      continue
    }

    // Empty line → spacer
    if (line.trim() === '') {
      elements.push(<div key={key++} className="h-2" />)
      i++
      continue
    }

    // Paragraph
    elements.push(
      <p key={key++}>{parseInline(line)}</p>
    )
    i++
  }

  return elements
}

/* Parse inline markdown: bold, italic, inline-code, links */
function parseInline(text) {
  const parts = []
  // Split on **, *, `backtick`, [link](url)
  const regex = /(\*\*(.+?)\*\*|\*(.+?)\*|`([^`]+)`|\[([^\]]+)\]\(([^)]+)\))/g
  let last = 0
  let match

  while ((match = regex.exec(text)) !== null) {
    if (match.index > last) {
      parts.push(text.slice(last, match.index))
    }
    if (match[0].startsWith('**')) {
      parts.push(<strong key={match.index}>{match[2]}</strong>)
    } else if (match[0].startsWith('*')) {
      parts.push(<em key={match.index}>{match[3]}</em>)
    } else if (match[0].startsWith('`')) {
      parts.push(<code key={match.index}>{match[4]}</code>)
    } else if (match[0].startsWith('[')) {
      parts.push(
        <a
          key={match.index}
          href={match[6]}
          target="_blank"
          rel="noreferrer"
        >
          {match[5]}
        </a>
      )
    }
    last = match.index + match[0].length
  }

  if (last < text.length) {
    parts.push(text.slice(last))
  }

  return parts.length === 1 && typeof parts[0] === 'string' ? text : parts
}

function MarkdownTable({ lines }) {
  const rows = lines.map(l =>
    l
      .split('|')
      .map(c => c.trim())
      .filter((_, i, arr) => i > 0 && i < arr.length - 1)
  )
  const [header, , ...body] = rows
  return (
    <table>
      <thead>
        <tr>
          {header?.map((cell, i) => <th key={i}>{parseInline(cell)}</th>)}
        </tr>
      </thead>
      <tbody>
        {body.map((row, r) => (
          <tr key={r}>
            {row.map((cell, c) => <td key={c}>{parseInline(cell)}</td>)}
          </tr>
        ))}
      </tbody>
    </table>
  )
}

function CodeBlock({ code, lang }) {
  const [copied, setCopied] = useState(false)

  const handleCopy = () => {
    navigator.clipboard.writeText(code)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="my-3 rounded-xl overflow-hidden border border-white/[0.08]">
      {/* Header bar */}
      <div className="flex items-center justify-between px-4 py-2 bg-white/[0.04] border-b border-white/[0.06]">
        <span className="text-[11px] font-mono text-zinc-500 uppercase tracking-wider">
          {lang || 'plaintext'}
        </span>
        <button
          onClick={handleCopy}
          className="flex items-center gap-1.5 text-[11px] text-zinc-500 hover:text-zinc-200 px-2 py-0.5 rounded-md hover:bg-white/[0.06]"
        >
          {copied ? (
            <>
              <Check size={11} className="text-emerald-400" />
              <span className="text-emerald-400">Copied</span>
            </>
          ) : (
            <>
              <Copy size={11} />
              <span>Copy</span>
            </>
          )}
        </button>
      </div>
      <pre className="px-4 py-3 text-zinc-300 text-[0.82rem] leading-relaxed overflow-x-auto bg-[#13131a]">
        <code>{code}</code>
      </pre>
    </div>
  )
}

/* ──────────────────────────────────────────────
   Main ChatMessage component
────────────────────────────────────────────── */
export default function ChatMessage({ message }) {
  const isUser = message.role === 'user'

  if (isUser) {
    return (
      <div className="flex justify-end mb-6 group">
        <div className="flex items-end gap-2.5 max-w-[75%]">
          <div className="px-4 py-3 rounded-2xl rounded-br-sm bg-zinc-800 border border-white/[0.07] text-zinc-100 text-sm leading-relaxed">
            {message.content}
          </div>
          <div className="w-7 h-7 rounded-full bg-indigo-500/20 flex items-center justify-center shrink-0 mb-0.5">
            <User size={13} className="text-indigo-300" />
          </div>
        </div>
      </div>
    )
  }

  // AI response
  return (
    <div className="flex gap-4 mb-8 group">
      {/* Avatar */}
      <div className="w-8 h-8 rounded-xl bg-indigo-600/20 border border-indigo-500/20 flex items-center justify-center shrink-0 mt-0.5">
        <Sigma size={14} className="text-indigo-400" />
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <p className="text-[11px] font-semibold text-indigo-400 mb-2 uppercase tracking-widest">
          STUDENT
        </p>

        {message.loading ? (
          <div className="flex items-center gap-1.5 py-2">
            {[0, 1, 2].map(i => (
              <span
                key={i}
                className="w-1.5 h-1.5 rounded-full bg-indigo-400 opacity-70"
                style={{
                  animation: 'pulse 1.2s ease-in-out infinite',
                  animationDelay: `${i * 0.2}s`,
                }}
              />
            ))}
          </div>
        ) : (
          <div className="ai-prose text-sm text-zinc-300 leading-relaxed">
            {renderMarkdown(message.content)}
          </div>
        )}
      </div>
    </div>
  )
}
