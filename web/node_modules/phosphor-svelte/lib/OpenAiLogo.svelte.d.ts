import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `OpenAiLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <OpenAiLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const OpenAiLogo: Component<IconComponentProps, {}, "">;
type OpenAiLogo = ReturnType<typeof OpenAiLogo>;
export default OpenAiLogo;
