import { useState } from 'react'
import {
  AlertTriangle,
  Check,
  ChevronDown,
  Circle,
  Cloud,
  Code2,
  Info,
  MoreHorizontal,
  Plus,
  Search,
  SlidersHorizontal,
  SquareTerminal,
  X,
} from 'lucide-react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

type PaneTabsProps = { tabs: string[]; active?: number }

function PaneTabs({ tabs, active = 0 }: PaneTabsProps) {
  return (
    <div className="agent-pane-tabs">
      {tabs.map((tab, index) => (
        <button className={`agent-pane-tab ${index === active ? 'active' : ''}`} key={tab}>
          <i /> <span>{tab}</span> <X />
        </button>
      ))}
      <button className="agent-pane-add" aria-label="New tab"><Plus /></button>
    </div>
  )
}

function ClaudePane() {
  return (
    <section className="agent-pane claude-pane">
      <PaneTabs tabs={['claude', '7']} active={1} />
      <div className="shell-terminal">
        <p>rengwu@JGRs-MacBook-Pro bb-chartr % <i className="shell-cursor outline" /></p>
      </div>
    </section>
  )
}

function OpenCodePane() {
  return (
    <section className="agent-pane terminal-pane opencode-pane">
      <PaneTabs tabs={['opencode', '4']} active={1} />
      <div className="shell-terminal opencode-shell">
        <p>rengwu@JGRs-MacBook-Pro bb-chartr % herdr<br />error: nested herdr is disabled by default.<br />see configuration if you want to enable it.</p>
        <p>“recursion is a pathway to many abilities some consider to be... unn<br />atural.”<br />rengwu@JGRs-MacBook-Pro bb-chartr % <i className="shell-cursor outline" /></p>
      </div>
    </section>
  )
}

function GrokPane() {
  return (
    <section className="agent-pane terminal-pane grok-pane">
      <PaneTabs tabs={['grok', '8']} active={1} />
      <div className="shell-terminal">
        <p>rengwu@JGRs-MacBook-Pro bb-chartr % <i className="shell-cursor solid" /></p>
      </div>
    </section>
  )
}

function WorkspacePreview() {
  const [activeSession, setActiveSession] = useState('group')
  return (
    <div className="native-window workspace-window">
      <div className="native-titlebar workspace-titlebar">
        <div className="workspace-title"><div className="traffic-lights"><i /><i /><i /></div><strong>bb-chartr</strong><span>⌃</span></div>
        <div />
        <ChevronDown />
      </div>
      <div className="workspace-native-body">
        <aside className="workspace-native-sidebar">
          <section className={`workspace-space-card free-space-card ${activeSession === 'free' ? 'active' : 'inactive'}`}>
            <div className="free-session-head"><span>Free sessions</span><Plus /></div>
            <button className={`free-session ${activeSession === 'free' ? 'active' : ''}`} onClick={() => setActiveSession('free')}><i />opencode</button>
            <button className="free-session"><i />2</button>
          </section>
          <button className={`workspace-space-card ${activeSession === 'group' ? 'active' : ''}`} onClick={() => setActiveSession('group')}>
            <div><strong>bb-chartr</strong><span><Plus /><MoreHorizontal /></span></div>
            <p className="sidebar-group-active"><SquareTerminal />6 tabs</p>
            <p className="sidebar-session-hover"><i />5 <X /></p>
            <p className="sidebar-session-rest"><i />6</p>
          </button>
        </aside>
        <main className="agent-workspace">
          <ClaudePane />
          <OpenCodePane />
          <GrokPane />
        </main>
      </div>
    </div>
  )
}

function SettingsPreview() {
  return (
    <div className="native-window settings-window">
      <div className="native-titlebar">
        <div className="traffic-lights"><i /><i /><i /></div>
        <div className="window-title">Settings</div>
        <div />
      </div>
      <div className="settings-body">
        <nav className="settings-nav">
          <div className="settings-search"><Search /> Search settings</div>
          {['General', 'Appearance', 'Terminal', 'Hotkeys', 'Plugins'].map((item) => (
            <button key={item} className={item === 'Appearance' ? 'active' : ''}>{item === 'Appearance' ? <SlidersHorizontal /> : <Circle />}{item}</button>
          ))}
        </nav>
        <main className="settings-main">
          <div className="settings-header"><div><span>SETTINGS</span><h2>Appearance</h2></div><span>Changes save automatically</span></div>
          <div className="setting-group">
            <div className="setting-row"><div><strong>Theme mode</strong><span>Choose one theme or match your system.</span></div><div className="segmented"><button className="active">Fixed</button><button>System</button></div></div>
            <div className="setting-row"><div><strong>Fixed theme</strong><span>Applied to every chartr window.</span></div><button className="native-select">Current playground theme <ChevronDown /></button></div>
          </div>
          <div className="setting-group">
            <div className="setting-row"><div><strong>UI font</strong><span>Used throughout chartr's interface.</span></div><button className="native-select">IBM Plex Sans <ChevronDown /></button></div>
            <div className="setting-row"><div><strong>UI font size</strong><span>Scales controls and interface text.</span></div><div className="stepper"><button>−</button><span>14</span><button>+</button></div></div>
            <div className="setting-row"><div><strong>Reduce motion</strong><span>Minimize non-essential animation.</span></div><button className="native-switch" aria-label="Reduce motion"><i /></button></div>
          </div>
        </main>
      </div>
    </div>
  )
}

function TokenLabel({ children }: { children: string }) {
  return <code className="specimen-token">{children}</code>
}

function ComponentsPreview() {
  return (
    <div className="component-showcase">
      <header className="showcase-titlebar">
        <div className="traffic-lights"><i /><i /><i /></div>
        <div><strong>chartr component states</strong><span>Theme token specimen</span></div>
        <b>20 TOKENS</b>
      </header>

      <div className="showcase-grid">
        <section className="specimen-section sidebar-specimen">
          <div className="specimen-heading"><div><span>01</span><h2>Sidebar hierarchy</h2></div><TokenLabel>sidebar</TokenLabel></div>
          <div className="mini-sidebar">
            <div className="mini-sidebar-heading"><span>Spaces</span><Plus /></div>
            <div className="mini-space inactive"><div><i /><strong>personal</strong><MoreHorizontal /></div><TokenLabel>sidebar_card_inactive</TokenLabel></div>
            <div className="mini-space active">
              <div><i /><strong>chartr</strong><MoreHorizontal /></div>
              <p className="rest"><SquareTerminal />3 tabs <TokenLabel>sidebar_card_active</TokenLabel></p>
              <p className="hover"><i />review changes <TokenLabel>sidebar_session_hover</TokenLabel></p>
              <p className="active"><i />theme playground <TokenLabel>sidebar_session_active</TokenLabel></p>
            </div>
          </div>
        </section>

        <section className="specimen-section chrome-specimen">
          <div className="specimen-heading"><div><span>02</span><h2>Pane chrome</h2></div><TokenLabel>card</TokenLabel></div>
          <div className="mini-pane">
            <div className="mini-tabs">
              <button><i />claude <X /></button>
              <button><i />opencode <X /></button>
              <button className="open"><i />grok <X /></button>
              <button className="add"><Plus /></button>
            </div>
            <div className="mini-terminal">
              <p><span>~/Projects/chartr</span> <b>main</b></p>
              <p>Theme preview ready. <em>Waiting for input…</em></p>
              <p><strong>❯</strong><i /></p>
            </div>
          </div>
          <div className="token-key"><TokenLabel>surface</TokenLabel><TokenLabel>card</TokenLabel><TokenLabel>border</TokenLabel><TokenLabel>terminal_foreground</TokenLabel></div>
        </section>

        <section className="specimen-section interaction-specimen">
          <div className="specimen-heading"><div><span>03</span><h2>Interaction states</h2></div><TokenLabel>ring</TokenLabel></div>
          <div className="interaction-list">
            <button><span><i />Resting row</span><TokenLabel>card</TokenLabel></button>
            <button className="hover"><span><i />Hovered row</span><TokenLabel>hover</TokenLabel></button>
            <button className="pressed"><span><i />Pressed row</span><TokenLabel>card_open</TokenLabel></button>
            <button className="selected"><span><Check />Selected row</span><TokenLabel>selected</TokenLabel></button>
            <button className="focused"><span><Code2 />Focused field</span><TokenLabel>ring</TokenLabel></button>
          </div>
        </section>

        <section className="specimen-section content-specimen">
          <div className="specimen-heading"><div><span>04</span><h2>Content contrast</h2></div><TokenLabel>text</TokenLabel></div>
          <div className="type-stack">
            <p className="primary-copy">Primary interface text <TokenLabel>text</TokenLabel></p>
            <p className="muted-copy">Supporting metadata and labels <TokenLabel>muted</TokenLabel></p>
            <p className="quiet-copy">Disabled or de-emphasized content <TokenLabel>quiet</TokenLabel></p>
            <a>Open documentation <TokenLabel>accent</TokenLabel></a>
          </div>
        </section>

        <section className="specimen-section status-specimen">
          <div className="specimen-heading"><div><span>05</span><h2>Semantic status</h2></div><span className="status-summary"><i /><i /><i /></span></div>
          <div className="semantic-grid">
            <div className="semantic notice"><AlertTriangle /><div><strong>Build failed</strong><span>3 compiler errors</span></div><TokenLabel>notice</TokenLabel></div>
            <div className="semantic done"><Check /><div><strong>Checks passed</strong><span>128 complete</span></div><TokenLabel>done</TokenLabel></div>
            <div className="semantic idle"><Info /><div><strong>Runner idle</strong><span>Waiting for work</span></div><TokenLabel>idle</TokenLabel></div>
            <div className="semantic accent"><Cloud /><div><strong>Sync available</strong><span>Review update</span></div><TokenLabel>accent</TokenLabel></div>
          </div>
        </section>

        <section className="specimen-section layer-specimen">
          <div className="specimen-heading"><div><span>06</span><h2>Surface stack</h2></div><span /></div>
          <div className="layer-stack">
            <div><TokenLabel>surface</TokenLabel><div><TokenLabel>sidebar</TokenLabel><div><TokenLabel>card</TokenLabel><div><TokenLabel>card_open</TokenLabel></div></div></div></div>
          </div>
          <p className="layer-note">Boundaries use <TokenLabel>border</TokenLabel> throughout.</p>
        </section>
      </div>
    </div>
  )
}

export function ThemePreview() {
  return (
    <Tabs defaultValue="components" className="preview-tabs">
      <div className="preview-bar">
        <TabsList>
          <TabsTrigger value="components">Components</TabsTrigger>
          <TabsTrigger value="workspace">Workspace</TabsTrigger>
          <TabsTrigger value="settings">Settings</TabsTrigger>
        </TabsList>
        <div className="preview-label"><span>LIVE PREVIEW</span><i /></div>
      </div>
      <TabsContent value="components" className="preview-stage component-stage"><ComponentsPreview /></TabsContent>
      <TabsContent value="workspace" className="preview-stage"><WorkspacePreview /></TabsContent>
      <TabsContent value="settings" className="preview-stage"><SettingsPreview /></TabsContent>
    </Tabs>
  )
}
