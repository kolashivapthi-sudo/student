import { useRef, useEffect, useState } from 'react'
import { ArrowUp, Square, Paperclip, Lightbulb } from 'lucide-react'

export default function FloatingInputBar({ onSend, isStreaming, onStop }) {
  const [value, setValue] = useState('')
  const textareaRef = useRef(null)

  // Auto-resize textarea
  useEffect(() => {
    const el = textareaRef.current
    if (!el) return
    el.style.height = 'auto'
    el.style.height = Math.min(el.scrollHeight, 200) + 'px'
  }, [value])

  const handleSubmit = () => {
    const trimmed = value.trim()
    if (!trimmed || isStreaming) return
    onSend(trimmed)
    setValue('')
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto'
    }
  }

  const handleKeyDown = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSubmit()
    }
  }

  const isEmpty = value.trim().length === 0

  return (
    <div className="absolute bottom-0 left-0 right-0 px-4 pb-5 pt-8"
      style={{
        background: 'linear-gradient(to top, #0f0f11 60%, transparent)',
      }}
    >
      <div className="max-w-3xl mx-auto">
        {/* Quick suggestion pills */}
        {!isStreaming && (
          <div className="flex gap-2 mb-3 overflow-x-auto pb-1 scrollbar-none">
            {[
              'x is 5 more than 3',
              'John has twice Mary\'s apples. Mary has 6.',
              'Find x: x plus 4 equals 10',
            ].map((s, i) => (
              <button
                key={i}
                onClick={() => setValue(s)}
                className="shrink-0 flex items-center gap-1.5 px-3 py-1.5 rounded-full
                  border border-white/[0.08] bg-white/[0.03] text-xs text-zinc-500
                  hover:text-zinc-300 hover:border-white/[0.14] hover:bg-white/[0.06]"
              >
                <Lightbulb size={10} className="text-indigo-400" />
                {s}
              </button>
            ))}
          </div>
        )}

        {/* Main input container */}
        <div
          className="flex flex-col rounded-2xl border border-white/[0.10] bg-zinc-900/80"
          style={{ backdropFilter: 'blur(12px)' }}
        >
          {/* Textarea row */}
          <div className="flex items-end gap-3 px-4 py-3">
            {/* Attachment button */}
            <button className="text-zinc-600 hover:text-zinc-400 mb-0.5 shrink-0">
              <Paperclip size={16} />
            </button>

            {/* Textarea */}
            <textarea
              ref={textareaRef}
              value={value}
              onChange={e => setValue(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Ask a word problem… e.g. John has 3 more apples than Mary. Mary has 5."
              rows={1}
              className="flex-1 bg-transparent text-zinc-100 placeholder-zinc-600
                text-sm leading-relaxed resize-none outline-none
                max-h-48 py-0.5"
            />

            {/* Send / Stop button */}
            <div className="shrink-0 mb-0.5">
              {isStreaming ? (
                <button
                  onClick={onStop}
                  className="w-8 h-8 rounded-xl bg-zinc-700 hover:bg-zinc-600
                    flex items-center justify-center text-zinc-200"
                >
                  <Square size={13} fill="currentColor" />
                </button>
              ) : (
                <button
                  onClick={handleSubmit}
                  disabled={isEmpty}
                  className={`w-8 h-8 rounded-xl flex items-center justify-center
                    transition-all duration-150
                    ${isEmpty
                      ? 'bg-white/[0.05] text-zinc-600 cursor-not-allowed'
                      : 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-lg shadow-indigo-900/30'
                    }`}
                >
                  <ArrowUp size={15} strokeWidth={2.5} />
                </button>
              )}
            </div>
          </div>

          {/* Footer hint */}
          <div className="flex items-center justify-between px-4 py-2 border-t border-white/[0.05]">
            <p className="text-[10px] text-zinc-700">
              Supports: <span className="text-zinc-600">more than · less than · times · divided by · sum · product</span>
            </p>
            <p className="text-[10px] text-zinc-700">
              <kbd className="bg-white/[0.05] px-1.5 py-0.5 rounded text-[9px]">↵</kbd> to send
              &nbsp;·&nbsp;
              <kbd className="bg-white/[0.05] px-1.5 py-0.5 rounded text-[9px]">⇧↵</kbd> newline
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}
