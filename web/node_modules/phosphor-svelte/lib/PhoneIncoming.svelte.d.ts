import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PhoneIncomingIcon` instead.
 *
 * @example
 * ```svelte
 * <PhoneIncoming color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PhoneIncoming: Component<IconComponentProps, {}, "">;
type PhoneIncoming = ReturnType<typeof PhoneIncoming>;
export default PhoneIncoming;
