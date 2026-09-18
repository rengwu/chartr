import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GitPullRequestIcon` instead.
 *
 * @example
 * ```svelte
 * <GitPullRequest color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GitPullRequest: Component<IconComponentProps, {}, "">;
type GitPullRequest = ReturnType<typeof GitPullRequest>;
export default GitPullRequest;
