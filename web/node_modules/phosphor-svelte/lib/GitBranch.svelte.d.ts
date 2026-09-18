import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GitBranchIcon` instead.
 *
 * @example
 * ```svelte
 * <GitBranch color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GitBranch: Component<IconComponentProps, {}, "">;
type GitBranch = ReturnType<typeof GitBranch>;
export default GitBranch;
