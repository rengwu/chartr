import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PhoneIcon` instead.
 *
 * @example
 * ```svelte
 * <Phone color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Phone: Component<IconComponentProps, {}, "">;
type Phone = ReturnType<typeof Phone>;
export default Phone;
