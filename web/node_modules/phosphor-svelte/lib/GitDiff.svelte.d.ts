import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GitDiffIcon` instead.
 *
 * @example
 * ```svelte
 * <GitDiff color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GitDiff: Component<IconComponentProps, {}, "">;
type GitDiff = ReturnType<typeof GitDiff>;
export default GitDiff;
