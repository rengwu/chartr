package terminal

import (
	"github.com/rengwu/chartr/internal/adapter"
	"github.com/rengwu/chartr/internal/proc"
	"github.com/rengwu/chartr/internal/transcript"
)

// bindTranscript is the production half of the title seam: it turns a live agent
// tab into the persisted session behind it. It is the manager's default binder;
// a test injects its own normalized source in its place.
//
// Every step of it can decline, and declining is ordinary: a platform with no
// process reader, an adapter chartr has measured no transcript store for, an
// unreadable process, a state root that is not there, a candidate session that
// is not unique. None of them is an error the cockpit surfaces — the tab simply
// stays untitled, which costs nothing, while a wrong binding would spend the
// operator's money on someone else's conversation.
func bindTranscript(t *Terminal) (TitleBinding, bool) {
	agent, ok := t.transcriptAgent()
	if !ok {
		return TitleBinding{}, false
	}
	w, ok := transcript.Watch(agent)
	if !ok {
		return TitleBinding{}, false
	}
	return TitleBinding{
		Adapter: agent.Adapter,
		// The profile a generation must run under, rendered from the root the
		// live agent's own environment selected — never a copy of what was read
		// (internal/proc, adapter.StateRootEnv).
		Env:    adapter.StateRootEnv(agent.Adapter, agent.StateRoot),
		Source: w,
	}, true
}

// transcriptAgent resolves the process identity behind the tab's foreground
// agent — the pid, working directory and validated state root a transcript is
// matched from.
//
// How it is resolved is the one place a launched tab and an ad-hoc shell differ,
// for the reason the sampler has always differed: chartr started the binary in a
// launched tab, so its root process *is* the agent, while an ad-hoc shell's
// agent is whatever currently holds the PTY's foreground group. Everything past
// the resolution is identical.
//
// An adapter chartr can watch no transcripts for is refused before any process
// is read, so an unwatchable tab costs no lookups at all.
func (t *Terminal) transcriptAgent() (proc.Agent, bool) {
	t.mu.Lock()
	agent, launched, pid, pgrp := t.titleAgentLocked(), t.launchedAgent, t.shellPID, t.lastPgrp
	t.mu.Unlock()

	if agent == "" || !transcript.Supported(agent) {
		return proc.Agent{}, false
	}
	if launched != "" {
		return proc.Launched(agent, pid)
	}
	if pgrp <= 0 {
		return proc.Agent{}, false
	}
	return proc.Foreground(pgrp, agentEngine.Identify)
}
