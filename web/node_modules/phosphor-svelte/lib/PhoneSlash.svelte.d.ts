import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PhoneSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <PhoneSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PhoneSlash: Component<IconComponentProps, {}, "">;
type PhoneSlash = ReturnType<typeof PhoneSlash>;
export default PhoneSlash;
