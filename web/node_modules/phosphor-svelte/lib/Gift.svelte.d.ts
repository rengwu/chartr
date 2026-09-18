import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GiftIcon` instead.
 *
 * @example
 * ```svelte
 * <Gift color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Gift: Component<IconComponentProps, {}, "">;
type Gift = ReturnType<typeof Gift>;
export default Gift;
