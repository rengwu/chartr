import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GitCommitIcon` instead.
 *
 * @example
 * ```svelte
 * <GitCommit color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GitCommit: Component<IconComponentProps, {}, "">;
type GitCommit = ReturnType<typeof GitCommit>;
export default GitCommit;
