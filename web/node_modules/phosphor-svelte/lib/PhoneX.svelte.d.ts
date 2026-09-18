import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PhoneXIcon` instead.
 *
 * @example
 * ```svelte
 * <PhoneX color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PhoneX: Component<IconComponentProps, {}, "">;
type PhoneX = ReturnType<typeof PhoneX>;
export default PhoneX;
