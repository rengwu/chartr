import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GitlabLogoSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <GitlabLogoSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GitlabLogoSimple: Component<IconComponentProps, {}, "">;
type GitlabLogoSimple = ReturnType<typeof GitlabLogoSimple>;
export default GitlabLogoSimple;
