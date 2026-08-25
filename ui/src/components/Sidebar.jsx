import { useState } from 'react'
import {
  MessageSquarePlus,
  ChevronLeft,
  ChevronRight,
  Search,
  Settings,
  User,
  Hash,
  Sigma,
  Trash2,
} from 'lucide-react'

const HISTORY = {
  Today: [
    { id: 1, title: 'Sum of two numbers = 20' },
    { id: 2, title: 'John has 5 more apples than Mary' },
  ],
  Yesterday: [
    { id: 3, title: 'Train speed word problem' },
    { id: 4, title: 'Divided by zero test' },
  ],
  'Previous 7 Days': [
    { id: 5, title: 'Age problem: father and son' },
    { id: 6, title: 'Quadratic — unsupported degree' },
    { id: 7, title: 'Two-variable system' },
  ],
}

export default function Sidebar({ activeId, onSelect, onNew }) {
  const [collapsed, setCollapsed] = useState(false)
  const [hoveredId, setHoveredId] = useState(null)

  return (
    <aside
      style={{ width: collapsed ? 64 : 260, minWidth: collapsed ? 64 : 260 }}
      className="relative flex flex-col h-full bg-[#111113] border-r border-white/[0.06] overflow-hidden"
    >
      {/* ── Top bar ── */}
      <div className="flex items-center justify-between px-3 pt-4 pb-3">
        {!collapsed && (
          <div className="flex items-center gap-2 pl-1">
            <div className="w-6 h-6 rounded-md bg-indigo-500/20 flex items-center justify-center">
              <Sigma size={13} className="text-indigo-400" />
            </div>
            <span className="text-sm font-semibold text-zinc-100 tracking-tight">
              STUDENT
            </span>
          </div>
        )}
        <button
          onClick={() => setCollapsed(c => !c)}
          className="ml-auto w-7 h-7 flex items-center justify-center rounded-lg text-zinc-500 hover:text-zinc-300 hover:bg-white/[0.06]"
        >
          {collapsed ? <ChevronRight size={15} /> : <ChevronLeft size={15} />}
        </button>
      </div>

      {/* ── New Chat button ── */}
      <div className="px-3 mb-3">
        <button
          onClick={onNew}
          className={`w-full flex items-center gap-2.5 px-3 py-2 rounded-xl
            bg-indigo-600/20 hover:bg-indigo-600/30 border border-indigo-500/20
            text-indigo-300 hover:text-indigo-200 text-sm font-medium
            ${collapsed ? 'justify-center' : ''}`}
        >
          <MessageSquarePlus size={15} />
          {!collapsed && <span>New Chat</span>}
        </button>
      </div>

      {/* ── Search ── */}
      {!collapsed && (
        <div className="px-3 mb-4">
          <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/[0.04] border border-white/[0.06]">
            <Search size={13} className="text-zinc-500 shrink-0" />
            <input
              type="text"
              placeholder="Search chats..."
              className="bg-transparent text-xs text-zinc-400 placeholder-zinc-600 w-full outline-none"
            />
          </div>
        </div>
      )}

      {/* ── History ── */}
      <div className="flex-1 overflow-y-auto px-2 space-y-5">
        {!collapsed &&
          Object.entries(HISTORY).map(([group, chats]) => (
            <div key={group}>
              <p className="text-[10px] font-semibold uppercase tracking-widest text-zinc-600 px-2 mb-1.5">
                {group}
              </p>
              <ul className="space-y-0.5">
                {chats.map(chat => (
                  <li key={chat.id}>
                    <button
                      onMouseEnter={() => setHoveredId(chat.id)}
                      onMouseLeave={() => setHoveredId(null)}
                      onClick={() => onSelect(chat.id)}
                      className={`w-full flex items-center justify-between gap-2 px-2 py-1.5 rounded-lg text-left text-xs
                        ${activeId === chat.id
                          ? 'bg-white/[0.07] text-zinc-100'
                          : 'text-zinc-400 hover:bg-white/[0.04] hover:text-zinc-200'
                        }`}
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        <Hash size={11} className="text-zinc-600 shrink-0" />
                        <span className="truncate">{chat.title}</span>
                      </div>
                      {hoveredId === chat.id && (
                        <Trash2
                          size={12}
                          className="text-zinc-600 hover:text-red-400 shrink-0"
                        />
                      )}
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          ))}

        {/* Collapsed: just icons */}
        {collapsed && (
          <div className="flex flex-col items-center gap-1 pt-1">
            {Object.values(HISTORY)
              .flat()
              .slice(0, 5)
              .map(chat => (
                <button
                  key={chat.id}
                  onClick={() => onSelect(chat.id)}
                  title={chat.title}
                  className={`w-9 h-9 flex items-center justify-center rounded-lg
                    ${activeId === chat.id
                      ? 'bg-white/[0.08] text-zinc-200'
                      : 'text-zinc-600 hover:bg-white/[0.05] hover:text-zinc-400'
                    }`}
                >
                  <Hash size={13} />
                </button>
              ))}
          </div>
        )}
      </div>

      {/* ── Bottom: profile / settings ── */}
      <div className="px-3 pb-4 pt-3 border-t border-white/[0.05] space-y-1">
        <button
          className={`w-full flex items-center gap-2.5 px-2 py-2 rounded-lg text-zinc-500
            hover:text-zinc-300 hover:bg-white/[0.05] text-xs
            ${collapsed ? 'justify-center' : ''}`}
        >
          <Settings size={14} />
          {!collapsed && <span>Settings</span>}
        </button>
        <button
          className={`w-full flex items-center gap-2.5 px-2 py-2 rounded-lg
            text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.05]
            ${collapsed ? 'justify-center' : ''}`}
        >
          <div className="w-5 h-5 rounded-full bg-indigo-500/30 flex items-center justify-center shrink-0">
            <User size={11} className="text-indigo-300" />
          </div>
          {!collapsed && (
            <div className="min-w-0">
              <p className="text-xs font-medium text-zinc-200 truncate">
                Shivapthi
              </p>
              <p className="text-[10px] text-zinc-600 truncate">
                Free plan
              </p>
            </div>
          )}
        </button>
      </div>
    </aside>
  )
}
