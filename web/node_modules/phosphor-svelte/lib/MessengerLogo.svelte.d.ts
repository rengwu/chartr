import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MessengerLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <MessengerLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MessengerLogo: Component<IconComponentProps, {}, "">;
type MessengerLogo = ReturnType<typeof MessengerLogo>;
export default MessengerLogo;
