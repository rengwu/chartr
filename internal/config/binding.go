// Package config owns chartr's local configuration: the operator's registered
// agent library (agents.go) and the closed set of roles a session is spawned to
// do. Execution is chosen per spawn from the library — there are no role→agent
// bindings, no committed execution layer, and nothing about how an agent runs can
// arrive by `git pull` (ADR 0009 as superseded by the agent-selection effort). A
// role still picks a skill, derives a default from a ticket's type, and drives the
// AFK/HITL quiet hint; it simply no longer resolves to an agent.
package config

import (
	"os/exec"

	"github.com/rengwu/chartr/internal/wayfinder"
)

// Role is one of the closed set of things a session is spawned to do (ADR
// 0002). The set is fixed here; anything outside it is one caller's mistake with
// one answer rather than each entry point's own.
type Role string

const (
	RoleGrill     Role = "grill"
	RolePrototype Role = "prototype"
	RoleResearch  Role = "research"
	RoleImplement Role = "implement"
)

// Roles is the closed role set in a stable display order. It is the set every
// ticket offers a session in: what a ticket *is* picks the default role, and the
// operator picks from all four at the gate.
var Roles = []Role{RoleGrill, RolePrototype, RoleResearch, RoleImplement}

// IsRole reports whether a string names one of the four roles, exactly (the set
// is case-sensitive, as every producer of it is). Every surface that takes a
// role from outside — the spawn action's request body, the payload preview's —
// checks it here, so an unknown role is one caller's mistake with one answer
// rather than each entry point's own.
func IsRole(role string) bool {
	for _, r := range Roles {
		if string(r) == role {
			return true
		}
	}
	return false
}

// RoleForTicketType returns the role a ticket of this type spawns as. The
// method's four ticket types map one-to-one onto the four roles, which is the
// per-ticket fact a map's kind used to approximate uniformly; an unrecognised
// type falls to implement, the same default the frontend has always used.
//
// This takes wayfinder's own types rather than restating the strings: wayfinder
// imports nothing of ours, so there is no cycle to dodge and no second copy of
// the mapping to drift.
func RoleForTicketType(t wayfinder.Type) Role {
	switch t {
	case wayfinder.TypeGrilling:
		return RoleGrill
	case wayfinder.TypePrototype:
		return RolePrototype
	case wayfinder.TypeResearch:
		return RoleResearch
	default:
		return RoleImplement
	}
}

// RoleIsAFK reports whether a session in this role runs unattended — the operator
// kicks it off and walks away — as opposed to a human-in-the-loop role that is
// *supposed* to sit idle waiting on its human. Only this split earns a session the
// "quiet" hint: an AFK session silent past a threshold may be stuck, while an idle
// HITL session is simply waiting and must show nothing (spec, Sessions and
// adapters; stories 34–35).
//
// `grill` is the human-in-the-loop role — a grilling session is a dialogue, and
// story 35 names it as the one that must never wear a quiet badge. Every other
// role (prototype, research, implement) runs to completion on its own, so
// silence from one is a signal worth surfacing. An unrecognised name is treated as
// AFK: a stray session shows the hint rather than swallowing a possible stall.
func RoleIsAFK(role string) bool {
	return role != string(RoleGrill)
}

// LookPath reports whether a binary is resolvable on the current PATH.
func LookPath(binary string) bool {
	_, err := exec.LookPath(binary)
	return err == nil
}
