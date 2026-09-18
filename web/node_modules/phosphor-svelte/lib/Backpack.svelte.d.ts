import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BackpackIcon` instead.
 *
 * @example
 * ```svelte
 * <Backpack color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Backpack: Component<IconComponentProps, {}, "">;
type Backpack = ReturnType<typeof Backpack>;
export default Backpack;
