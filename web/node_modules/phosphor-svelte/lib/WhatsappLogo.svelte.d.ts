import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WhatsappLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <WhatsappLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WhatsappLogo: Component<IconComponentProps, {}, "">;
type WhatsappLogo = ReturnType<typeof WhatsappLogo>;
export default WhatsappLogo;
