import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GithubLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <GithubLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GithubLogo: Component<IconComponentProps, {}, "">;
type GithubLogo = ReturnType<typeof GithubLogo>;
export default GithubLogo;
