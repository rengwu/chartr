import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <OpenAiLogoIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const OpenAiLogoIcon: Component<IconComponentProps, {}, "">;
type OpenAiLogoIcon = ReturnType<typeof OpenAiLogoIcon>;
export default OpenAiLogoIcon;
