import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PhoneListIcon` instead.
 *
 * @example
 * ```svelte
 * <PhoneList color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PhoneList: Component<IconComponentProps, {}, "">;
type PhoneList = ReturnType<typeof PhoneList>;
export default PhoneList;
