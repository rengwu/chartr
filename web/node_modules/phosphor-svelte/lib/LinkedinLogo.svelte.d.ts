import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LinkedinLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <LinkedinLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LinkedinLogo: Component<IconComponentProps, {}, "">;
type LinkedinLogo = ReturnType<typeof LinkedinLogo>;
export default LinkedinLogo;
