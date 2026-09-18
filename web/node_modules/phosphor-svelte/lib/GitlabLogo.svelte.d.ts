import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GitlabLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <GitlabLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GitlabLogo: Component<IconComponentProps, {}, "">;
type GitlabLogo = ReturnType<typeof GitlabLogo>;
export default GitlabLogo;
