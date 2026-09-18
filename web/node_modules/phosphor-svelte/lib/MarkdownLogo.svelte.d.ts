import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MarkdownLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <MarkdownLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MarkdownLogo: Component<IconComponentProps, {}, "">;
type MarkdownLogo = ReturnType<typeof MarkdownLogo>;
export default MarkdownLogo;
