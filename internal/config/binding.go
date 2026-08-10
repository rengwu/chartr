// Package config owns chartr's local configuration: the operator's
// registered agent library (agents.go) and the closed set of roles a
// session is spawned to do. Execution is chosen per spawn from the
// library — no role→agent bindings, no committed execution layer, so
// nothing about how an agent runs can arrive by `git pull`. A role still
// picks a skill, derives a default from a ticket's type, and drives the
// AFK/HITL quiet hint; it just no longer resolves to an agent.
package config

import (
	"os/exec"

	"github.com/rengwu/chartr/internal/wayfinder"
)

// Role is one of the closed set of things a session is spawned to do (ADR
// 0002). Fixed here; anything outside it is one caller's mistake with one
// answer, not each entry point's own.
type Role string

const (
	RoleGrill     Role = "grill"
	RolePrototype Role = "prototype"
	RoleResearch  Role = "research"
	RoleImplement Role = "implement"
)

// Roles is the closed role set in stable display order — the set every
// ticket offers a session in: what a ticket is picks the default role, the
// operator picks from all four at the gate.
var Roles = []Role{RoleGrill, RolePrototype, RoleResearch, RoleImplement}

// RoleBindingAuto is the sentinel a `[roles]` row carries to mean "resolve by
// precedence": chartr walks the enabled sources in order and takes the first
// that ships a skill matching the role's accepted names (SkillAliases). It is
// the "no preference" the settings picker writes — distinct from an absent row,
// which leaves the role unbound and refuses the spawn, and from a `Source/skill`
// ref, which pins one source's skill against precedence. A bare word rather than
// a qualified ref on purpose: it names no source, because following the source
// order is exactly what it asks for.
const RoleBindingAuto = "auto"

// IsRole reports whether a string names one of the four roles exactly
// (case-sensitive). Every surface taking a role from outside — the spawn
// action's request body, the payload preview's — checks it here, so an
// unknown role is one caller's mistake with one answer.
func IsRole(role string) bool {
	for _, r := range Roles {
		if string(r) == role {
			return true
		}
	}
	return false
}

// RoleForTicketType returns the role a ticket of this type spawns as. The
// four ticket types map one-to-one onto the four roles; an unrecognised
// type falls to implement, the frontend's longstanding default.
//
// Takes wayfinder's own types rather than restating the strings: wayfinder
// imports nothing of ours, so no cycle to dodge and no second copy to drift.
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

// SkillAliases names the skill names each role is willing to resolve from any
// enabled source when its explicit binding does not resolve. The operator's
// binding is tried first; if it fails, these names are searched in source order
// so a repo that ships the right skill names works without hand-editing the
// binding.
func SkillAliases(role Role) []string {
	switch role {
	case RoleGrill:
		return []string{"grill", "grill-me", "grilling", "grill-with-docs"}
	case RolePrototype:
		return []string{"prototype"}
	case RoleResearch:
		return []string{"research"}
	case RoleImplement:
		return []string{"implement"}
	default:
		return nil
	}
}

// LookPath reports whether a binary is resolvable on the current PATH.
func LookPath(binary string) bool {
	_, err := exec.LookPath(binary)
	return err == nil
}
