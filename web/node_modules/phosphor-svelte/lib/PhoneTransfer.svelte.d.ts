import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PhoneTransferIcon` instead.
 *
 * @example
 * ```svelte
 * <PhoneTransfer color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PhoneTransfer: Component<IconComponentProps, {}, "">;
type PhoneTransfer = ReturnType<typeof PhoneTransfer>;
export default PhoneTransfer;
