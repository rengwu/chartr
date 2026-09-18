import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SwordIcon` instead.
 *
 * @example
 * ```svelte
 * <Sword color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Sword: Component<IconComponentProps, {}, "">;
type Sword = ReturnType<typeof Sword>;
export default Sword;
