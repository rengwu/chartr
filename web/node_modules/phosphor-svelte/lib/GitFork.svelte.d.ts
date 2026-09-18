import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GitForkIcon` instead.
 *
 * @example
 * ```svelte
 * <GitFork color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GitFork: Component<IconComponentProps, {}, "">;
type GitFork = ReturnType<typeof GitFork>;
export default GitFork;
