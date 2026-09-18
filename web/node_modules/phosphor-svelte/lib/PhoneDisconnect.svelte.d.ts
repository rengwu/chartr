import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PhoneDisconnectIcon` instead.
 *
 * @example
 * ```svelte
 * <PhoneDisconnect color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PhoneDisconnect: Component<IconComponentProps, {}, "">;
type PhoneDisconnect = ReturnType<typeof PhoneDisconnect>;
export default PhoneDisconnect;
