import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MegaphoneIcon` instead.
 *
 * @example
 * ```svelte
 * <Megaphone color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Megaphone: Component<IconComponentProps, {}, "">;
type Megaphone = ReturnType<typeof Megaphone>;
export default Megaphone;
